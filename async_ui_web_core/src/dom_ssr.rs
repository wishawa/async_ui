use std::cell::RefCell;
use std::mem;
use std::ops::Deref;
use std::rc::{Rc, Weak};
pub type Node = SsrNode;
pub type HtmlElement = elements::HtmlElement;
pub type Element = SsrElement;
pub type Text = SsrText;
pub type EventTarget = SsrEventTarget;
pub type DocumentFragment = SsrDocumentFragment;
pub type Comment = SsrComment;

pub struct SsrEventTarget {}

impl SsrEventTarget {
    pub fn event_subscribed(&self, name: &str) {
        // I think it might be good to I.e disable checkboxes etc
        // in SSR when there is an event subscription created, making it impossible
        // to interact with such elements until client hydration is complete?
        println!("event subscribed: {name}")
    }
    pub fn event_unsubscribed(&self, name: &str) {
        println!("event unsubscribed: {name}")
    }
    pub fn to_owned(&self) -> Self {
        Self {}
    }
}

pub mod elements {
    use super::{Element, Node};
    use std::ops::Deref;

    macro_rules! impl_element {
        ($name:ident) => {
            #[repr(transparent)]
            pub struct $name(Element);

            impl AsRef<Node> for $name {
                fn as_ref(&self) -> &Node {
                    &self.0 .0
                }
            }
            impl AsRef<Element> for $name {
                fn as_ref(&self) -> &Element {
                    &self.0
                }
            }
            impl AsRef<$name> for $name {
                fn as_ref(&self) -> &Self {
                    &self
                }
            }
            impl From<Element> for $name {
                fn from(e: Element) -> Self {
                    Self(e)
                }
            }
        };
        ($name:ident, childless) => {
            impl_element!($name);
            impl AsRef<HtmlElement> for $name {
                fn as_ref(&self) -> &HtmlElement {
                    // SAFETY: All node structs are `repr(transparent)` and have
                    // the same contents, also all are only using interior mutability.
                    unsafe { core::mem::transmute(&self.0) }
                }
            }
            impl Deref for $name {
                type Target = HtmlElement;
                fn deref(&self) -> &Self::Target {
                    self.as_ref()
                }
            }
        };
        ($name:ident, childed) => {
            impl_element!($name);
            impl AsRef<HtmlElement> for $name {
                fn as_ref(&self) -> &HtmlElement {
                    // SAFETY: All node structs are `repr(transparent)` and have
                    // the same contents, also all are only using interior mutability.
                    unsafe { core::mem::transmute(&self.0) }
                }
            }
            impl Deref for $name {
                type Target = HtmlElement;
                fn deref(&self) -> &Self::Target {
                    self.as_ref()
                }
            }
        };
        ($name:ident, childed, htmlelement) => {
            impl_element!($name);
            impl Deref for $name {
                type Target = Element;
                fn deref(&self) -> &Self::Target {
                    self.as_ref()
                }
            }
        };
    }
    impl_element!(HtmlElement, childed, htmlelement);
    impl_element!(HtmlAnchorElement, childed);
    impl_element!(HtmlAreaElement, childless);
    impl_element!(HtmlAudioElement, childed);
    // impl_element!(HtmlBElement, childed);
    impl_element!(HtmlBrElement, childless);
    impl_element!(HtmlBaseElement, childless);
    impl_element!(HtmlButtonElement, childed);
    impl_element!(HtmlCanvasElement, childed);
    impl_element!(HtmlDListElement, childed);
    impl_element!(HtmlDataElement, childed);
    impl_element!(HtmlDataListElement, childed);
    impl_element!(HtmlDialogElement, childed);
    impl_element!(HtmlDivElement, childed);
    impl_element!(HtmlEmbedElement, childless);
    impl_element!(HtmlFieldSetElement, childed);
    impl_element!(HtmlFormElement, childed);
    impl_element!(HtmlFrameSetElement, childed);
    impl_element!(HtmlHrElement, childless);
    impl_element!(HtmlHeadingElement, childed);
    impl_element!(HtmlImageElement, childed);
    impl_element!(HtmlIFrameElement, childed);
    impl_element!(HtmlInputElement, childless);
    impl_element!(HtmlLiElement, childed);
    impl_element!(HtmlLabelElement, childed);
    impl_element!(HtmlLegendElement, childed);
    impl_element!(HtmlLinkElement, childless);
    impl_element!(HtmlMapElement, childed);
    impl_element!(HtmlMetaElement, childless);
    impl_element!(HtmlMeterElement, childed);
    impl_element!(HtmlOListElement, childed);
    impl_element!(HtmlObjectElement, childed);
    impl_element!(HtmlOptGroupElement, childed);
    impl_element!(HtmlOptionElement, childed);
    impl_element!(HtmlOutputElement, childed);
    impl_element!(HtmlParagraphElement, childed);
    impl_element!(HtmlPictureElement, childed);
    impl_element!(HtmlPreElement, childed);
    impl_element!(HtmlProgressElement, childed);
    impl_element!(HtmlQuoteElement, childed);
    impl_element!(HtmlSelectElement, childed);
    impl_element!(HtmlSourceElement, childless);
    impl_element!(HtmlSpanElement, childed);
    impl_element!(HtmlStyleElement, childed);
    impl_element!(HtmlTableCellElement, childed);
    impl_element!(HtmlTableColElement, childed);
    impl_element!(HtmlTableElement, childed);
    impl_element!(HtmlTableRowElement, childed);
    impl_element!(HtmlTableSectionElement, childed);
    impl_element!(HtmlTemplateElement, childed);
    impl_element!(HtmlTextAreaElement, childed);
    impl_element!(HtmlTimeElement, childed);
    impl_element!(HtmlTrackElement, childless);
    impl_element!(HtmlUListElement, childed);
    impl_element!(HtmlVideoElement, childed);

    macro_rules! ats {
        ($id:ident, &str, $v:literal) => {
            pub fn $id(&self, v: &str) {
                self.set_attribute($v, v)
            }
        };
        ($id:ident, bool, $v:literal) => {
            pub fn $id(&self, v: bool) {
                if v {
                    self.set_attribute($v, "")
                } else {
                    self.remove_attribute($v);
                }
            }
        };
    }
    impl HtmlButtonElement {
        pub fn set_disabled(&self, disabled: bool) {
            todo!()
        }
    }
    impl HtmlLabelElement {
        ats!(set_html_for, &str, "for");
    }
    impl HtmlInputElement {
        ats!(set_type, &str, "type");
        ats!(set_placeholder, &str, "placeholder");
        ats!(set_autofocus, bool, "autofocus");
        pub fn value(&self) -> String {
            todo!()
        }
        pub fn value_as_number(&self) -> f64 {
            todo!()
        }
        pub fn checked(&self) -> bool {
            todo!()
        }
        pub fn set_checked(&self, flag: bool) {
            todo!()
        }
        pub fn set_min(&self, v: &str) {
            todo!()
        }
        pub fn set_max(&self, v: &str) {
            todo!()
        }
        pub fn set_step(&self, v: &str) {
            todo!()
        }
        pub fn set_value(&self, v: &str) {
            todo!()
        }
    }
    impl HtmlOptionElement {
        pub fn set_selected(&self, flag: bool) {
            todo!()
        }
    }
    impl HtmlSelectElement {
        pub fn set_value(&self, val: &str) {
            todo!()
        }
        pub fn selected_index(&self) -> usize {
            todo!()
        }
        pub fn set_autofocus(&self, auto: bool) {
            todo!()
        }
        pub fn set_placeholder(&self, placeholder: &str) {
            todo!()
        }
    }
    impl HtmlAnchorElement {
        ats!(set_href, &str, "href");
    }
}

// It is possible to make it just a wrapper that calls `set_attribute(class, ...)`...
// Except async-ui already provides api for classList manipulation, and we need to optimize for this case
#[derive(Debug, Default, Clone)]
pub struct SsrClassList(Rc<RefCell<Vec<String>>>);
impl SsrClassList {
    pub fn add_1(&self, class: &str) -> Option<()> {
        let mut v = self.0.borrow_mut();
        v.push(class.to_owned());

        Some(())
    }
    pub fn remove_1(&self, class: &str) -> Option<()> {
        let mut v = self.0.borrow_mut();
        let pos = v.iter().position(|el| el == class);
        if let Some(pos) = pos {
            v.remove(pos);
        }

        Some(())
    }
    // TODO: Delegate add_1 to add, not the other way around
    pub fn add<'s>(&self, c: impl IntoIterator<Item = &'s str>) -> Option<()> {
        for c in c {
            self.add_1(c)?;
        }

        Some(())
    }
    // TODO: Delegate add_1 to add, not the other way around
    pub fn remove<'s>(&self, c: impl IntoIterator<Item = &'s str>) -> Option<()> {
        for c in c {
            self.remove_1(c)?;
        }

        Some(())
    }
    pub fn toggle_with_force(&self, c: &str, force: bool) -> Option<()> {
        if force {
            self.add_1(c)
        } else {
            self.remove_1(c)
        }
    }
}

#[derive(Debug)]
enum SsrNodeKind {
    Text(String),
    Element {
        name: String,
        classes: SsrClassList,
        attrs: Vec<(String, String)>,
        children: Vec<Node>,
    },
    Comment(String),
}
#[derive(Debug)]
struct SsrNodeInner {
    kind: SsrNodeKind,
    parent: Option<WeakSsrNode>,
}

#[derive(Debug)]
struct WeakSsrNode(Weak<RefCell<SsrNodeInner>>);
impl WeakSsrNode {
    fn upgrade(&self) -> Option<SsrNode> {
        self.0.upgrade().map(SsrNode)
    }
}

// TODO: The structure is weird... It would be better to make it the
// other way around... Implement every method for every node kind and then
// make `SsrNode` just an `enum {SsrElement, SsrText, ...}`?
//
// Then `AsRef<Node>` implementations would be weird without unsafe code.
#[derive(Clone)]
pub struct SsrElement(SsrNode);
impl AsRef<SsrNode> for SsrElement {
    fn as_ref(&self) -> &SsrNode {
        &self.0
    }
}
impl AsRef<SsrElement> for SsrElement {
    fn as_ref(&self) -> &SsrElement {
        self
    }
}
impl From<SsrElement> for SsrNode {
    fn from(value: SsrElement) -> Self {
        value.0
    }
}
impl Deref for SsrElement {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct SsrText(SsrNode);
impl SsrText {
    pub fn set_data(&self, text: &str) {
        self.0.set_text_content(Some(text));
    }
}
impl AsRef<SsrNode> for SsrText {
    fn as_ref(&self) -> &SsrNode {
        &self.0
    }
}
impl From<SsrText> for SsrNode {
    fn from(value: SsrText) -> Self {
        value.0
    }
}
#[derive(Clone, Debug)]
pub struct SsrDocumentFragment(SsrNode);
impl SsrDocumentFragment {
    pub fn new() -> Option<Self> {
        Some(Self(create_ssr_element("#fragment").0))
    }
}
impl AsRef<SsrNode> for SsrDocumentFragment {
    fn as_ref(&self) -> &SsrNode {
        &self.0
    }
}
impl From<SsrDocumentFragment> for SsrNode {
    fn from(value: SsrDocumentFragment) -> Self {
        value.0
    }
}
impl Deref for SsrDocumentFragment {
    type Target = Node;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Clone)]
pub struct SsrComment(SsrNode);
impl SsrComment {
    pub fn new() -> Option<Self> {
        Some(Self(SsrNode(Rc::new(RefCell::new(SsrNodeInner {
            kind: SsrNodeKind::Comment("".to_owned()),
            parent: None,
        })))))
    }
}
impl AsRef<SsrNode> for SsrComment {
    fn as_ref(&self) -> &SsrNode {
        &self.0
    }
}
impl From<SsrComment> for SsrNode {
    fn from(value: SsrComment) -> Self {
        value.0
    }
}
impl Deref for SsrComment {
    type Target = Node;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct SsrNode(Rc<RefCell<SsrNodeInner>>);
impl SsrNode {
    fn take_parent(&self) {
        // borrow_mut to immediately take `node.parent` `Option`, might not be required
        // but doing that to be safe.
        let mut node = self.0.borrow_mut();
        if let Some(parent) = node.parent.take() {
            // unreachable!("experiment: check if reparenting happens this way in async-ui");
            if let Some(parent) = parent.upgrade() {
                drop(node);
                parent.remove_child(&parent);
            }
        }
    }
    // fn take_parent_for_remarent(&self, new_parent: &Self) {
    //     // borrow_mut to immediately take `node.parent` `Option`, might not be required
    //     // but doing that to be safe.
    //     let mut node = self.0.borrow_mut();
    //     if let Some(parent) = node.parent.take() {
    //         unreachable!("experiment: check if reparenting happens this way in async-ui");
    //         if let Some(parent) = parent.upgrade() {
    //             drop(node);
    //             parent.remove_child(&parent);
    //         }
    //     }
    // }
    fn take_known_parent(&self, known: &Self) {
        let mut node = self.0.borrow_mut();
        // parent might not be set if called from `insert_before`.
        if let Some(parent) = node.parent.take() {
            if cfg!(debug_assertions) {
                let parent = parent.upgrade().expect("parent is either not set or known");
                drop(node);
                assert!(
                    Self::ptr_eq(&parent, known),
                    "parent is either not set or known"
                );
            }
        }
    }

    pub fn focus(&self) -> Option<()> {
        // Noop, should it somehow be communicated to CSR/hydration,
        // e.g by anchor url or startup script?
        Some(())
    }
    // TODO: move this method to `Element`
    pub fn set_attribute(&self, name: &str, value: &str) {
        assert_ne!(name, "class");
        let mut node = self.0.borrow_mut();
        match &mut node.kind {
            SsrNodeKind::Element { attrs, .. } => {
                if let Some((_, v)) = attrs.iter_mut().find(|(n, v)| n == name) {
                    *v = value.to_owned();
                } else {
                    attrs.push((name.to_string(), value.to_string()));
                }
            }
            SsrNodeKind::Comment(_) | SsrNodeKind::Text(_) => {
                panic!("text/comment have no attributes")
            }
        }
    }
    // TODO: move this method to `Element`
    pub fn remove_attribute(&self, name: &str) {
        assert_ne!(name, "class");
        let mut node = self.0.borrow_mut();
        match &mut node.kind {
            SsrNodeKind::Element { attrs, .. } => {
                if let Some(pos) = attrs.iter_mut().position(|(n, v)| n == name) {
                    attrs.remove(pos);
                }
            }
            SsrNodeKind::Comment(_) | SsrNodeKind::Text(_) => {
                panic!("text/comment have no attributes")
            }
        }
    }
    // TODO: move this method to `Element`
    pub fn class_list(&self) -> SsrClassList {
        let node = self.0.borrow();
        match &node.kind {
            SsrNodeKind::Element { classes, .. } => classes.clone(),
            _ => unreachable!("non-elements have no classes"),
        }
    }
    // TODO: Move to `Element`
    // TODO: Should conflicts be handled?
    pub fn set_id(&self, id: &str) {
        self.set_attribute("id", id);
    }
    pub fn parent_node(&self) -> Option<Node> {
        let node = self.0.borrow();
        let parent = node.parent.as_ref()?;
        let v = parent
            .upgrade()
            .expect("parent shouldn't be dropped that early");
        Some(v)
    }
    pub fn next_sibling(&self) -> Option<Node> {
        let v = self.parent_node()?;
        let node = v.0.borrow();
        let SsrNodeKind::Element { children, .. } = &node.kind else {
            unreachable!("parent might onjy be element");
        };
        let pos = children
            .iter()
            .position(|el| Self::ptr_eq(el, self))
            .expect("parent should contain child");
        let sibling = children.get(pos+1)?.clone();
        assert!(!sibling.is_same_node(Some(self)));
        Some(sibling)
    }
    pub fn is_same_node(&self, other: Option<&Node>) -> bool {
        let Some(other) = other else {
            return false;
        };
        Self::ptr_eq(self, other)
    }
    pub fn append_child(&self, new_node: &Node) -> Option<()> {
        self.insert_before(new_node, None)
    }
    pub fn insert_before(&self, new_node: &Node, reference_node: Option<&Node>) -> Option<()> {
        assert!(!new_node.is_same_node(Some(self)));
        // insert_before removes node from the previous parent first.
        // Not sure it if matters in async-ui, but matching DOM behavior first.
        new_node.take_parent();

        let mut node = self.0.borrow_mut();
        match &mut node.kind {
            SsrNodeKind::Text(_) | SsrNodeKind::Comment(_) => {
                // TODO: Error: Cannot add children to a Text
                None
            }
            SsrNodeKind::Element { children, .. } => {
                // Find the insert position
                let pos = if let Some(reference_node) = reference_node {
                    // TODO: Error: Child to insert before is not a child of this node
                    let pos = children
                        .iter()
                        .position(|el| Self::ptr_eq(el, reference_node))?;
                    Some(pos)
                } else {
                    None
                };

                // Update node parent
                {
                    let mut node = new_node.0.borrow_mut();
                    node.parent = Some(Self::downgrade(self));
                }

                // Perform insertion
                if let Some(pos) = pos {
                    children.insert(pos, new_node.clone());
                } else {
                    children.push(new_node.clone());
                }
                Some(())
            }
        }
    }

    /// None corresponds to web NotFoundError: the node to be removed is not a child of this node.
    // TODO: Return removed child?
    pub fn remove_child(&self, child: &Node) -> Option<()> {
        let mut node = self.0.borrow_mut();
        match &mut node.kind {
            SsrNodeKind::Text(_) | SsrNodeKind::Comment(_) => None,
            SsrNodeKind::Element { children, .. } => {
                let pos = children.iter().position(|el| Self::ptr_eq(el, child))?;
                children.remove(pos);
                drop(node);
                child.take_known_parent(self);
                Some(())
            }
        }
    }

    // TODO: Move to `Element`
    pub fn set_inner_text(&self, text: &str) {
        self.set_text_content(Some(text));
    }
    pub fn set_text_content(&self, text: Option<&str>) {
        let mut node = self.0.borrow_mut();
        match &mut node.kind {
            SsrNodeKind::Text(v) | SsrNodeKind::Comment(v) => {
                *v = text.unwrap_or_default().to_owned();
            }
            SsrNodeKind::Element { children, .. } => {
                let old_children = mem::take(children);
                children.push(create_ssr_text(text.unwrap_or_default()).0);
                drop(node);

                for child in old_children {
                    child.take_known_parent(self);
                }
            }
        }
    }

    fn downgrade(this: &Self) -> WeakSsrNode {
        WeakSsrNode(Rc::downgrade(&this.0))
    }

    fn ptr_eq(a: &Self, b: &Self) -> bool {
        Rc::ptr_eq(&a.0, &b.0)
    }

    pub fn serialize_html(&self, last_is_text: &mut bool, out: &mut String) {
        fn encode_text(text: &str, out: &mut String) {
            for ele in text.chars() {
                match ele {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    c => out.push(c),
                }
            }
        }
        fn encode_attr_value(text: &str, out: &mut String) {
            for ele in text.chars() {
                match ele {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    '"' => out.push_str("&quot;"),
                    c => out.push(c),
                }
            }
        }

        let node = self.0.borrow();
        match &node.kind {
            SsrNodeKind::Text(t) => {
                if *last_is_text {
                    // For hydration - ensure text nodes are separated, as with real DOM building
                    out.push_str("<!---->");
                }
                encode_text(t, out);
                *last_is_text = true;
            }
            SsrNodeKind::Element {
                name,
                classes,
                attrs,
                children,
            } => {
                // TODO: Ensure there is nothing criminal in element name/attrs?
                out.push('<');
                out.push_str(name);
                for (k, v) in attrs {
                    out.push(' ');
                    out.push_str(k);
                    out.push('=');
                    out.push('"');
                    encode_attr_value(v, out);
                    out.push('"');
                }
                {
                    // TODO: Ensure added classes have no spaces in them?
                    let classes = classes.0.borrow();
                    if !classes.is_empty() {
                        out.push_str(" class=\"");
                        for (i, ele) in classes.iter().enumerate() {
                            if i != 0 {
                                out.push(' ');
                            }
                            encode_attr_value(ele, out);
                        }
                        out.push('"');
                    }
                }
                if children.is_empty() {
                    // Closing self-closing element is not valid in HTML4, ensure that DOCTYPE html is passed for html5 compat
                    out.push_str("/>");
                } else {
                    *last_is_text = false;
                    out.push('>');
                    for child in children {
                        child.serialize_html(last_is_text, out);
                    }
                    out.push('<');
                    out.push('/');
                    out.push_str(name);
                    out.push('>');
                }
                *last_is_text = false;
            }
            SsrNodeKind::Comment(c) => {
                // Is comment content even important? Maybe for some hydration markers?
                // TODO: Ensure proper escaping
                out.push_str("<!--'");
                out.push_str(c);
                out.push_str("-->");
                *last_is_text = false;
            }
        }
    }
}

pub fn create_ssr_element(name: &str) -> SsrElement {
    SsrElement(SsrNode(Rc::new(RefCell::new(SsrNodeInner {
        kind: SsrNodeKind::Element {
            name: name.to_owned(),
            attrs: vec![],
            children: vec![],
            classes: SsrClassList::default(),
        },
        parent: None,
    }))))
}
pub fn create_ssr_text(name: &str) -> SsrText {
    SsrText(SsrNode(Rc::new(RefCell::new(SsrNodeInner {
        kind: SsrNodeKind::Text(name.to_owned()),
        parent: None,
    }))))
}
