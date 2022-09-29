use std::{borrow::Cow, cell::RefCell, collections::HashSet};

use smallvec::SmallVec;
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
    Inserted(SmallVec<[DomTokenList; 1]>),
}

impl<'a> ClassList<'a> {
    pub fn new<S: Into<Cow<'a, str>>, I: IntoIterator<Item = S>>(classes: I) -> Self {
        Self {
            inner: RefCell::new(Inner {
                rust: classes.into_iter().map(Into::into).collect(),
                dom: DomEnum::None,
            }),
        }
    }
    pub fn add<S: Into<Cow<'a, str>>>(&self, class_name: S) {
        let mut bm = self.inner.borrow_mut();
        let v = class_name.into();
        if let DomEnum::Inserted(dom) = &bm.dom {
            dom.iter()
                .for_each(|dom| dom.add_1(&*v).expect("ClassList add failed"));
        }
        bm.rust.insert(v);
    }
    pub fn remove<S: Into<Cow<'a, str>>>(&self, class_name: S) {
        let mut bm = self.inner.borrow_mut();
        let v = class_name.into();
        if let DomEnum::Inserted(dom) = &bm.dom {
            dom.iter()
                .for_each(|dom| dom.remove_1(&*v).expect("ClassList remove failed"));
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
                dom.iter()
                    .for_each(|dom| dom.remove_1(&*v).expect("ClassList remove failed"));
            }
        } else {
            if let DomEnum::Inserted(dom) = &bm.dom {
                dom.iter()
                    .for_each(|dom| dom.add_1(&*v).expect("ClassList add failed"));
            }
            bm.rust.insert(v);
        }
    }
    pub fn set<S: Into<Cow<'a, str>>>(&self, class_name: S, value: bool) {
        match value {
            true => self.add(class_name),
            false => self.remove(class_name),
        }
    }
    pub(crate) fn set_dom(&self, dom: DomTokenList) {
        let mut bm = self.inner.borrow_mut();
        for item in bm.rust.iter() {
            dom.add_1(&*item).expect("ClassList add failed");
        }
        if let DomEnum::Inserted(lst) = &mut bm.dom {
            lst.push(dom);
        } else {
            bm.dom = DomEnum::Inserted(SmallVec::from([dom]));
        }
    }
}
