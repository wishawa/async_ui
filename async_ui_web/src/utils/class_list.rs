use std::{borrow::Cow, cell::RefCell, collections::HashSet, hash::Hash};

use smallvec::SmallVec;
use web_sys::DomTokenList;

pub struct ClassList<'a> {
    inner: RefCell<Inner<'a>>,
}
struct Inner<'a> {
    rust: SmallSet<Cow<'a, str>, 4>,
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
        bm.rust.remove(&v);
    }
    pub fn contains<S: Into<Cow<'a, str>>>(&self, class_name: S) -> bool {
        self.inner.borrow().rust.contains(&class_name.into())
    }
    pub fn toggle<S: Into<Cow<'a, str>>>(&self, class_name: S) {
        let mut bm = self.inner.borrow_mut();
        let v = class_name.into();
        if bm.rust.remove(&v) {
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
    pub fn clear(&self) {
        let mut bm = self.inner.borrow_mut();
        if let DomEnum::Inserted(doms) = &bm.dom {
            for dom in doms.iter() {
                dom.set_value("");
            }
        }
        bm.rust = SmallSet::new();
    }
    pub(crate) fn set_dom(&self, dom: DomTokenList) {
        let mut bm = self.inner.borrow_mut();
        bm.rust.for_each(|item| {
            dom.add_1(&*item).expect("ClassList add failed");
        });
        if let DomEnum::Inserted(lst) = &mut bm.dom {
            lst.push(dom);
        } else {
            bm.dom = DomEnum::Inserted(SmallVec::from([dom]));
        }
    }
}
impl<'a, 'b: 'a> From<&'b str> for ClassList<'a> {
    fn from(source: &'b str) -> Self {
        Self::new(source.split_ascii_whitespace())
    }
}
enum SmallSet<T: Eq + Hash, const N: usize> {
    Small(OptArray<T, N>),
    Spilled(HashSet<T>),
}

struct OptArray<T, const N: usize> {
    array: [Option<T>; N],
}

impl<T, const N: usize> OptArray<T, N> {
    fn new() -> Self {
        Self {
            array: [(); N].map(|_| Option::None),
        }
    }
    fn position<U: PartialEq<T>>(&self, value: &U) -> Option<usize> {
        self.array.iter().position(|item| {
            if let Some(item) = item.as_ref() {
                value.eq(item)
            } else {
                false
            }
        })
    }
    fn insert(&mut self, value: T) -> Result<(), T> {
        for slot in self.array.iter_mut() {
            if slot.is_none() {
                *slot = Some(value);
                return Ok(());
            }
        }
        Err(value)
    }
    fn remove<U: PartialEq<T>>(&mut self, value: &U) -> Option<T> {
        for slot in self.array.iter_mut() {
            if let Some(v) = slot {
                if value.eq(v) {
                    return slot.take();
                }
            }
        }
        None
    }
}

impl<T: Eq + Hash, const N: usize> FromIterator<T> for SmallSet<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let size = iter.size_hint().1.unwrap_or(0);
        let mut this = if size > N {
            Self::Spilled(HashSet::with_capacity(size))
        } else {
            Self::Small(OptArray::new())
        };
        for item in iter {
            this.insert(item);
        }
        this
    }
}

impl<T: Eq + Hash, const N: usize> SmallSet<T, N> {
    pub fn new() -> Self {
        Self::Small(OptArray::new())
    }
    pub fn insert(&mut self, value: T) {
        match self {
            SmallSet::Small(s) => {
                if let Err(value) = s.insert(value) {
                    let mut hs: HashSet<T> =
                        s.array.iter_mut().filter_map(|item| item.take()).collect();
                    hs.insert(value);
                    *self = SmallSet::Spilled(hs);
                }
            }
            SmallSet::Spilled(h) => {
                h.insert(value);
            }
        }
    }
    pub fn remove(&mut self, value: &T) -> bool {
        match self {
            SmallSet::Small(s) => s.remove(value).is_some(),
            SmallSet::Spilled(h) => h.remove(value),
        }
    }
    pub fn contains(&self, value: &T) -> bool {
        match self {
            SmallSet::Small(s) => s.position(value).is_some(),
            SmallSet::Spilled(h) => h.contains(value),
        }
    }
    pub fn for_each<F: FnMut(&T)>(&self, mut f: F) {
        match self {
            SmallSet::Small(s) => {
                for item in s.array.iter().filter_map(|it| it.as_ref()) {
                    f(item);
                }
            }
            SmallSet::Spilled(h) => {
                h.iter().for_each(f);
            }
        }
    }
}
