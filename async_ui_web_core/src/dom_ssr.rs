use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::mem;
use std::ops::Deref;
use std::rc::{Rc, Weak};
pub type Node = SsrNode;
pub type Element = SsrElement;
pub type HtmlElement = SsrHtmlElement;
pub type Text = SsrText;
pub type EventTarget = SsrEventTarget;
pub type DocumentFragment = SsrDocumentFragment;
pub type Comment = SsrComment;

pub struct SsrEventTarget {}

impl SsrEventTarget {
    pub fn event_subscribed(&self, _name: &str) {
        // I think it might be good to I.e disable checkboxes etc
        // in SSR when there is an event subscription created, making it impossible
        // to interact with such elements until client hydration is complete?
        // println!("event subscribed: {name}")
    }
    pub fn event_unsubscribed(&self, _name: &str) {
        // println!("event unsubscribed: {name}")
    }
    pub fn to_owned(&self) -> Self {
        Self {}
    }
}

pub mod elements {
    use super::{Element, SsrHtmlElement, HtmlElement, Node};
    use std::ops::Deref;

    macro_rules! impl_element {
        ($name:ident) => {
            #[repr(transparent)]
            pub struct $name(HtmlElement);

            impl AsRef<Node> for $name {
                fn as_ref(&self) -> &Node {
                    &self.0 .0
                }
            }
            impl AsRef<Element> for $name {
                fn as_ref(&self) -> &Element {
                    &self.0 .0
                }
            }
            impl AsRef<HtmlElement> for $name {
                fn as_ref(&self) -> &HtmlElement {
                    &self.0
                }
            }
            impl AsRef<$name> for $name {
                fn as_ref(&self) -> &Self {
                    self
                }
            }
            impl Deref for $name {
                type Target = HtmlElement;

                fn deref(&self) -> &Self::Target {
                    self.as_ref()
                }
            }
            impl TryFrom<Element> for $name {
                type Error = ();

                fn try_from(value: Element) -> Result<Self, Self::Error> {
                    // Nothing is done for non-html elements yet, all elements are html elements
                    Ok(Self(SsrHtmlElement(value)))
                }
            }
        };
    }
    impl_element!(HtmlAnchorElement);
    impl_element!(HtmlAreaElement);
    impl_element!(HtmlAudioElement);
    // impl_element!(HtmlBElement);
    impl_element!(HtmlBrElement);
    impl_element!(HtmlBaseElement);
    impl_element!(HtmlButtonElement);
    impl_element!(HtmlCanvasElement);
    impl_element!(HtmlDListElement);
    impl_element!(HtmlDataElement);
    impl_element!(HtmlDataListElement);
    impl_element!(HtmlDialogElement);
    impl_element!(HtmlDivElement);
    impl_element!(HtmlEmbedElement);
    impl_element!(HtmlFieldSetElement);
    impl_element!(HtmlFormElement);
    impl_element!(HtmlFrameSetElement);
    impl_element!(HtmlHrElement);
    impl_element!(HtmlHeadingElement);
    impl_element!(HtmlImageElement);
    impl_element!(HtmlIFrameElement);
    impl_element!(HtmlInputElement);
    impl_element!(HtmlLiElement);
    impl_element!(HtmlLabelElement);
    impl_element!(HtmlLegendElement);
    impl_element!(HtmlLinkElement);
    impl_element!(HtmlMapElement);
    impl_element!(HtmlMetaElement);
    impl_element!(HtmlMeterElement);
    impl_element!(HtmlOListElement);
    impl_element!(HtmlObjectElement);
    impl_element!(HtmlOptGroupElement);
    impl_element!(HtmlOptionElement);
    impl_element!(HtmlOutputElement);
    impl_element!(HtmlParagraphElement);
    impl_element!(HtmlPictureElement);
    impl_element!(HtmlPreElement);
    impl_element!(HtmlProgressElement);
    impl_element!(HtmlQuoteElement);
    impl_element!(HtmlSelectElement);
    impl_element!(HtmlSourceElement);
    impl_element!(HtmlSpanElement);
    impl_element!(HtmlStyleElement);
    impl_element!(HtmlTableCellElement);
    impl_element!(HtmlTableColElement);
    impl_element!(HtmlTableElement);
    impl_element!(HtmlTableRowElement);
    impl_element!(HtmlTableSectionElement);
    impl_element!(HtmlTemplateElement);
    impl_element!(HtmlTextAreaElement);
    impl_element!(HtmlTimeElement);
    impl_element!(HtmlTrackElement);
    impl_element!(HtmlUListElement);
    impl_element!(HtmlVideoElement);

    macro_rules! ats {
        ($id:ident, &str, attr: $v:literal) => {
            pub fn $id(&self, v: &str) {
                self.set_attribute($v, v)
            }
        };
        ($id:ident, bool, attr: $v:literal) => {
            pub fn $id(&self, v: bool) {
                if v {
                    self.set_attribute($v, "")
                } else {
                    self.remove_attribute($v);
                }
            }
        };
        ($id:ident, f64, attr: $v:literal) => {
            pub fn $id(&self, v: f64) {
                self.set_attribute($v, &v.to_string())
            }
        };
        ($id:ident, &str, noop) => {
            pub fn $id(&self, _v: &str) {}
        };
        ($id:ident, bool, noop) => {
            pub fn $id(&self, _v: bool) {}
        };
    }
    macro_rules! atg {
        ($id:ident, $ty:ty, $v:expr) => {
            pub fn $id(&self) -> $ty {
                $v
            }
        };
        // Some values can be get back from the attribute
        ($id:ident, $ty:ty, attr: $name:literal $(.map(|$mi:ident| $me:expr))? = $v:expr) => {
            pub fn $id(&self) -> $ty {
                self.get_attribute($name)
                    $(.map(|$mi| $me))?
                    .unwrap_or_else(|| $v)
            }
        };
    }
    impl HtmlElement {
        ats!(set_hidden, bool, attr: "hidden");
        ats!(set_title, &str, attr: "title");
    }
    impl HtmlButtonElement {
        ats!(set_disabled, bool, attr: "disabled");
    }
    impl HtmlLabelElement {
        ats!(set_html_for, &str, attr: "for");
    }
    impl HtmlInputElement {
        ats!(set_type, &str, attr: "type");
        ats!(set_placeholder, &str, attr: "placeholder");
        ats!(set_autofocus, bool, attr: "autofocus");
        ats!(set_value, &str, attr: "value");
        atg!(value, String, attr: "value" = "".to_owned());
        atg!(value_as_number, f64, attr: "value".map(|v| v.parse().unwrap_or(f64::NAN)) = f64::NAN);
        atg!(checked, bool, attr: "checked".map(|_v| true) = true);
        ats!(set_checked, bool, attr: "checked");
        ats!(set_step, &str, attr: "step");
        ats!(set_min, &str, attr: "min");
        ats!(set_max, &str, attr: "max");
        ats!(set_disabled, bool, attr: "disabled");
    }
    impl HtmlOptionElement {
        // TODO: Other sibling `HtmlSelectElement` options should be reset here, should it be emulated?
        // Can't do that server-side. Without that, server may produce invalid html.
        //
        // Maybe some generic on-load-prop-setting option should be added for not available attributes?..
        // E.g data-oncreate="self.selected = true"... It won't work with noscript, however.
        ats!(set_selected, bool, attr: "selected");
        atg!(value, String, attr: "value" = "".to_owned());
        ats!(set_value, &str, attr: "value");
        ats!(set_disabled, bool, attr: "disabled");
    }
    impl HtmlSelectElement {
        ats!(set_placeholder, &str, attr: "placeholder");
        // Is a property, not an attribute, see comment on set_selected in `HtmlOptionElement`
        ats!(set_value, &str, noop);
        atg!(value, String, "".to_owned());
        // Until set_value is not working, default index is ok...
        atg!(selected_index, i32, 0);
        ats!(set_autofocus, bool, attr: "autofocus");
        ats!(set_multiple, bool, attr: "multiple");
        ats!(set_disabled, bool, attr: "disabled");
    }
    impl HtmlAnchorElement {
        ats!(set_href, &str, attr: "href");
    }
    impl HtmlMeterElement {
        ats!(set_min, f64, attr: "min");
        ats!(set_max, f64, attr: "max");
        ats!(set_value, f64, attr: "value");
    }
}

#[derive(Debug, Clone)]
pub struct SsrStyle(SsrElement);
impl SsrStyle {
    pub fn set_property(&self, property: &str, value: &str) -> Option<()> {
        let mut element = self.0.borrow_element_mut();
        if value.is_empty() {
            if let Some(pos) = element.style.iter().position(|(k, _)| k == property) {
                element.style.remove(pos);
            }
        } else if let Some((_, v)) = element.style.iter_mut().find(|(k, _)| k == property) {
            *v = value.to_owned()
        } else {
            element.style.push((property.to_owned(), value.to_owned()));
        }
        Some(())
    }
}

// It is possible to make it just a wrapper that calls `set_attribute(class, ...)`...
// Except async-ui already provides api for classList manipulation, and we need to optimize for this case
#[derive(Debug, Clone)]
pub struct SsrClassList(SsrElement);
impl SsrClassList {
    pub fn add_1(&self, class: &str) -> Option<()> {
        let mut element = self.0.borrow_element_mut();

        let class = class.to_owned();

        if !element.classes.contains(&class) {
            element.classes.push(class);
        }

        Some(())
    }
    pub fn remove_1(&self, class: &str) -> Option<()> {
        let mut element = self.0.borrow_element_mut();

        let pos = element.classes.iter().position(|el| el == class);
        if let Some(pos) = pos {
            element.classes.remove(pos);
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
struct SsrElementData {
    name: String,
    classes: Vec<String>,
    style: Vec<(String, String)>,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
}

#[derive(Debug)]
enum SsrNodeKind {
    Text(String),
    Element(SsrElementData),
    DocumentFragment { children: Vec<Node> },
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

fn check_attr_name(name: &str) {
    if name == "class" || name == "style" {
        // Should be possible for `Element`, but not for `HtmlElement`.
        // For `HtmlElement` those accesses should be rewritten to use style declaration or classlist.
        panic!("unable to alter class/style attributes");
    }
}

#[derive(Clone, Debug)]
pub struct SsrElement(SsrNode);
impl SsrElement {
    fn borrow_element(&self) -> Ref<'_, SsrElementData> {
        Ref::map(self.0 .0.borrow(), |node| match &node.kind {
            SsrNodeKind::Element(ssr_element_data) => ssr_element_data,
            SsrNodeKind::Text(_)
            | SsrNodeKind::DocumentFragment { .. }
            | SsrNodeKind::Comment(_) => {
                unreachable!("invalid SsrElement: should be of Element node kind")
            }
        })
    }
    fn borrow_element_mut(&self) -> RefMut<'_, SsrElementData> {
        RefMut::map(self.0 .0.borrow_mut(), |node| match &mut node.kind {
            SsrNodeKind::Element(ssr_element_data) => ssr_element_data,
            SsrNodeKind::Text(_)
            | SsrNodeKind::DocumentFragment { .. }
            | SsrNodeKind::Comment(_) => {
                unreachable!("invalid SsrElement: should be of Element node kind")
            }
        })
    }

    pub fn class_list(&self) -> SsrClassList {
        SsrClassList(self.clone())
    }
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        check_attr_name(name);
        let element = self.borrow_element();

        let (_, v) = element.attrs.iter().find(|(n, _)| n == name)?;
        Some(v.to_owned())
    }
    pub fn set_attribute(&self, name: &str, value: &str) {
        check_attr_name(name);
        let mut element = self.borrow_element_mut();

        if let Some((_, v)) = element.attrs.iter_mut().find(|(n, _)| n == name) {
            *v = value.to_owned();
        } else {
            element.attrs.push((name.to_string(), value.to_string()));
        }
    }
    pub fn remove_attribute(&self, name: &str) {
        check_attr_name(name);
        let mut element = self.borrow_element_mut();

        if let Some(pos) = element.attrs.iter_mut().position(|(n, _)| n == name) {
            element.attrs.remove(pos);
        }
    }
    // TODO: Should conflicts be handled?
    pub fn set_id(&self, id: &str) {
        self.set_attribute("id", id);
    }
}
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

pub struct SsrHtmlElement(SsrElement);
impl SsrHtmlElement {
    pub fn style(&self) -> SsrStyle {
        SsrStyle(self.0.clone())
    }
    pub fn set_inner_text(&self, text: &str) {
        self.set_text_content(Some(text));
    }
}
impl AsRef<SsrNode> for SsrHtmlElement {
    fn as_ref(&self) -> &SsrNode {
        &self.0 .0
    }
}
impl AsRef<SsrElement> for SsrHtmlElement {
    fn as_ref(&self) -> &SsrElement {
        &self.0
    }
}
impl AsRef<SsrHtmlElement> for SsrHtmlElement {
    fn as_ref(&self) -> &SsrHtmlElement {
        self
    }
}
impl Deref for SsrHtmlElement {
    type Target = SsrElement;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl TryFrom<SsrElement> for SsrHtmlElement {
    type Error = ();

    fn try_from(value: SsrElement) -> Result<Self, Self::Error> {
        // Nothing is done for non-html elements yet, all elements are html elements
        Ok(Self(value))
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
        Some(Self(SsrNode(Rc::new(RefCell::new(SsrNodeInner {
            kind: SsrNodeKind::DocumentFragment { children: vec![] },
            parent: None,
        })))))
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
    pub fn set_data(&self, text: &str) {
        self.0.set_text_content(Some(text));
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

// If something is not used in SSR rendering, return this to avoid warnings caused
// by usage of empty tuple.
pub struct Unused;

#[derive(Clone, Debug)]
pub struct SsrNode(Rc<RefCell<SsrNodeInner>>);
impl SsrNode {
    fn take_parent(&self) {
        // borrow_mut to immediately take `node.parent` `Option`, might not be required
        // but doing that to be safe.
        let node = self.0.borrow();
        if let Some(parent) = &node.parent {
            // unreachable!("experiment: check if reparenting happens this way in async-ui");
            if let Some(parent) = parent.upgrade() {
                drop(node);
                parent
                    .remove_child(self)
                    .expect("parent is set for node => parent should contain this child");
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
        let parent = node.parent.take().expect("parent doesn't exists");
        {
            let parent = parent.upgrade().expect("parent should live at this point");
            drop(node);
            assert!(
                Self::ptr_eq(&parent, known),
                "parent is either not set or known"
            );
        }
    }

    pub fn focus(&self) -> Option<()> {
        // Noop, should it somehow be communicated to CSR/hydration,
        // e.g by anchor url or startup script?
        Some(())
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
        let (SsrNodeKind::Element(SsrElementData { children, .. })
        | SsrNodeKind::DocumentFragment { children }) = &node.kind
        else {
            unreachable!("parent might only be element or document fragment");
        };
        let pos = children
            .iter()
            .position(|el| Self::ptr_eq(el, self))
            .expect("parent should contain child");
        let sibling = children.get(pos + 1)?.clone();
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
        assert!(
            !new_node.is_same_node(Some(self)),
            "the new child can't be a parent"
        );
        // insert_before removes node from the previous parent first.
        // Not sure it if matters in async-ui, but matching DOM behavior first.
        new_node.take_parent();

        let mut node = self.0.borrow_mut();
        match &mut node.kind {
            SsrNodeKind::Text(_) | SsrNodeKind::Comment(_) => {
                // TODO: Error: Cannot add children to a Text
                None
            }
            SsrNodeKind::Element(SsrElementData { children, .. })
            | SsrNodeKind::DocumentFragment { children } => {
                // Find the insert position
                let mut pos = if let Some(reference_node) = reference_node {
                    // TODO: Error: Child to insert before is not a child of this node
                    let pos = children
                        .iter()
                        .position(|el| Self::ptr_eq(el, reference_node))?;
                    Some(pos)
                } else {
                    None
                };

                let mut node = new_node.0.borrow_mut();
                if let SsrNodeKind::DocumentFragment {
                    children: frag_child,
                } = &mut node.kind
                {
                    for child in std::mem::take(frag_child) {
                        let mut child_node = child.0.borrow_mut();
                        child_node.parent = Some(Self::downgrade(self));
                        drop(child_node);

                        if let Some(pos) = &mut pos {
                            children.insert(*pos, child.clone());
                            *pos += 1;
                        } else {
                            children.push(child.clone());
                        }
                    }
                } else {
                    // Update node parent
                    node.parent = Some(Self::downgrade(self));
                    drop(node);

                    // Perform insertion
                    if let Some(pos) = pos {
                        children.insert(pos, new_node.clone());
                    } else {
                        children.push(new_node.clone());
                    }
                }
                Some(())
            }
        }
    }

    /// None corresponds to web NotFoundError: the node to be removed is not a child of this node.
    // TODO: Return removed child?
    pub fn remove_child(&self, child: &Node) -> Option<Unused> {
        assert!(!self.is_same_node(Some(child)), "parent != child");
        let mut node = self.0.borrow_mut();
        match &mut node.kind {
            SsrNodeKind::Text(_) | SsrNodeKind::Comment(_) => None,
            SsrNodeKind::Element(SsrElementData { children, .. })
            | SsrNodeKind::DocumentFragment { children } => {
                let pos = children.iter().position(|el| Self::ptr_eq(el, child))?;
                children.remove(pos);
                drop(node);
                child.take_known_parent(self);
                Some(Unused)
            }
        }
    }

    pub fn set_text_content(&self, text: Option<&str>) {
        let mut node = self.0.borrow_mut();
        match &mut node.kind {
            SsrNodeKind::Text(v) | SsrNodeKind::Comment(v) => {
                *v = text.unwrap_or_default().to_owned();
            }
            SsrNodeKind::Element(SsrElementData { children, .. })
            | SsrNodeKind::DocumentFragment { children } => {
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

    pub fn to_html(&self) -> String {
        self.to_html_impl(false)
    }
    pub fn to_inner_html(&self) -> String {
        self.to_html_impl(true)
    }

    fn to_html_impl(&self, mut inner: bool) -> String {
        let mut out = String::new();
        self.serialize_html(&mut inner, &mut HashSet::new(), &mut false, &mut out)
            .expect("fmt shouldn't fail");
        out
    }

    fn serialize_html(
        &self,
        skip_this: &mut bool,
        visited: &mut HashSet<usize>,
        last_is_text: &mut bool,
        out: &mut String,
    ) -> fmt::Result {
        use std::fmt::Write;

        struct Text<'s>(&'s str);
        impl Display for Text<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                for ele in self.0.chars() {
                    match ele {
                        '&' => write!(f, "&amp;")?,
                        '<' => write!(f, "&lt;")?,
                        '>' => write!(f, "&gt;")?,
                        c => write!(f, "{c}")?,
                    }
                }
                Ok(())
            }
        }
        struct AttrValue<'s>(&'s str);
        impl Display for AttrValue<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                for ele in self.0.chars() {
                    match ele {
                        '&' => write!(f, "&amp;")?,
                        '<' => write!(f, "&lt;")?,
                        '>' => write!(f, "&gt;")?,
                        '"' => write!(f, "&quot;")?,
                        c => write!(f, "{c}")?,
                    }
                }
                Ok(())
            }
        }
        let id_addr = Rc::as_ptr(&self.0).addr();
        #[cfg(debug_assertions)]
        if !visited.insert(id_addr) {
            write!(out, "<!--BUG: CYCLE nod=\"{id_addr:x}\"-->")?;
            *last_is_text = false;
            return Ok(());
        }

        let skipped = *skip_this;
        *skip_this = false;

        let node = self.0.borrow();
        match &node.kind {
            SsrNodeKind::DocumentFragment { children } => {
                write!(
                    out,
                    "<!--BUG: documentfragment is never inserted directly-->"
                )?;
                for child in children {
                    child.serialize_html(skip_this, visited, last_is_text, out)?;
                }
                write!(out, "<!--BUG: end of documentfragment-->")?;
                *last_is_text = false;
            }
            SsrNodeKind::Text(t) => {
                if *last_is_text && cfg!(feature = "hydrate") {
                    // For hydration - ensure text nodes are separated, as with real DOM building
                    write!(out, "<!--DUMMY TEXT SEP-->")?;
                }
                write!(out, "{}", Text(t))?;
                *last_is_text = true;
            }
            SsrNodeKind::Element(SsrElementData{
                name,
                classes,
                attrs,
                children,
                style,
            }) => {
                // TODO: Ensure there is nothing criminal in element name/attrs?
                if !skipped {
                    out.push('<');
                    out.push_str(name);
                    // #[cfg(debug_assertions)]
                    // {
                    //     // Debug piece for cycle debugging
                    //     write!(out, " nod=\"{id_addr:x}\"")?;
                    // }
                    {
                        // TODO: Ensure added classes have no spaces in them?
                        if !classes.is_empty() {
                            out.push_str(" class=\"");
                            for (i, ele) in classes.iter().enumerate() {
                                if i != 0 {
                                    out.push(' ');
                                }
                                write!(out, "{}", AttrValue(ele))?;
                            }
                            out.push('"');
                        }
                    }
                    {
                        // TODO: Properties are usually processed by css parser, it would be pretty costly to do that
                        // in SSR, implement some cheaper form of verification.
                        if !style.is_empty() {
                            out.push_str(" style=\"");
                            for (k, v) in style.iter() {
                                write!(out, "{k}: {v};")?;
                            }
                            out.push('"');
                        }
                    }
                    for (k, v) in attrs {
                        write!(out, " {k}=\"{}\"", AttrValue(v))?;
                    }
                }
                if children.is_empty() {
                    // Closing self-closing element is not valid in HTML4, ensure that DOCTYPE html is passed for html5 compat
                    if !skipped {
                        out.push_str("/>");
                    }
                } else {
                    if !skipped {
                        *last_is_text = false;
                        out.push('>');
                    }
                    for child in children {
                        child.serialize_html(skip_this, visited, last_is_text, out)?;
                    }
                    if !skipped {
                        write!(out, "</{name}>")?;
                    }
                }
                if !skipped {
                    *last_is_text = false;
                }
            }
            SsrNodeKind::Comment(c) => {
                // Is comment content even important? Maybe for some hydration markers?
                // TODO: Make sure nothing is broken due to nod display
                // TODO: Ensure proper escaping
                write!(out, "<!--{c}-->")?;
                //nod=\"{id_addr:x}\"-->")?;
                *last_is_text = false;
            }
        }

        #[cfg(debug_assertions)]
        if !visited.remove(&id_addr) {
            panic!("visited marker disappeared");
        }
        Ok(())
    }
}

pub fn create_ssr_element(name: &str) -> SsrElement {
    SsrElement(SsrNode(Rc::new(RefCell::new(SsrNodeInner {
        kind: SsrNodeKind::Element(SsrElementData {
            name: name.to_owned(),
            attrs: vec![],
            children: vec![],
            classes: vec![],
            style: vec![],
        }),
        parent: None,
    }))))
}
pub fn create_ssr_text(name: &str) -> SsrText {
    SsrText(SsrNode(Rc::new(RefCell::new(SsrNodeInner {
        kind: SsrNodeKind::Text(name.to_owned()),
        parent: None,
    }))))
}

#[inline]
pub fn marker_node(dbg: &'static str) -> Comment {
    let c = Comment::new().expect("marker");
    #[cfg(debug_assertions)]
    {
        c.set_data(dbg);
    }
    c
}
