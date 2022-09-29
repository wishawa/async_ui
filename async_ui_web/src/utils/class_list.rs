use std::{
    borrow::{Borrow, Cow},
    cell::RefCell,
    collections::HashSet,
};

use web_sys::DomTokenList;

pub struct ClassList<'a> {
    inner: RefCell<Inner<'a>>,
}
struct Inner<'a> {
    rust: HashSet<Cow<'a, str>>,
    dom: DomEnum,
}
enum DomEnum {
    None,
    Inserted(DomTokenList),
    Dummy,
}
thread_local! {
    static DUMMY_CLASS_LIST: ClassList<'static> = ClassList { inner: RefCell::new(Inner {
        rust: HashSet::new(),
        dom: DomEnum::Dummy
    }) }
}
impl<'a> ClassList<'a> {
    pub fn add<S: Into<Cow<'a, str>>>(&self, class_name: S) {
        let mut bm = self.inner.borrow_mut();
        let v = class_name.into();
        if let DomEnum::Inserted(dom) = &bm.dom {
            dom.add_1(&*v).expect("ClassList add failed");
        }
        bm.rust.insert(v);
    }
    pub fn remove<S: Into<Cow<'a, str>>>(&self, class_name: S) {
        let mut bm = self.inner.borrow_mut();
        let v = class_name.into();
        if let DomEnum::Inserted(dom) = &bm.dom {
            dom.remove_1(&*v).expect("ClassList remove failed");
        }
        bm.rust.remove(&*v);
    }
    pub fn contains<S: Into<Cow<'a, str>>>(&self, class_name: S) -> bool {
        self.inner.borrow().rust.contains(&*class_name.into())
    }
    pub fn toggle<S: Into<Cow<'a, str>>>(&self, class_name: S) {
        let mut bm = self.inner.borrow_mut();
        let v = class_name.into();
        if bm.rust.remove(&*v) {
            if let DomEnum::Inserted(dom) = &bm.dom {
                dom.remove_1(&*v).expect("ClassList remove failed");
            }
        } else {
            if let DomEnum::Inserted(dom) = &bm.dom {
                dom.add_1(&*v).expect("ClassList add failed");
            }
            bm.rust.insert(v);
        }
    }
    pub(crate) fn set_dom(&self, dom: DomTokenList) {
        let mut bm = self.inner.borrow_mut();
        if let DomEnum::None = &bm.dom {
            bm.dom = DomEnum::Inserted(dom);
        }
    }
}
impl ClassList<'static> {}
