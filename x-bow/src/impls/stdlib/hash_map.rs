use std::{
    cell::{Ref, RefMut},
    collections::HashMap,
    hash::Hash,
    ops::Deref,
};

use crate::{impls::leaf::LeafPathBuilder, path::Path, trackable::Trackable};

#[derive(x_bow_macros::IntoInnerPath)]
#[into_inner_path(prefix = crate::trackable)]
pub struct HashMapPathBuilder<K: Eq + Clone + Hash, V, P: Path<Out = HashMap<K, V>>> {
    inner_path: P,
}

impl<K: Eq + Clone + Hash, V, P: Path<Out = HashMap<K, V>>> Deref for HashMapPathBuilder<K, V, P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.inner_path
    }
}

impl<K: Eq + Clone + Hash, V> Trackable for HashMap<K, V> {
    type PathBuilder<P: Path<Out = Self>> = HashMapPathBuilder<K, V, P>;

    fn new_path_builder<P: Path<Out = Self>>(parent: P) -> Self::PathBuilder<P> {
        HashMapPathBuilder { inner_path: parent }
    }
}

impl<K: Eq + Clone + Hash, V, P: Path<Out = HashMap<K, V>> + Clone> Clone
    for HashMapPathBuilder<K, V, P>
{
    fn clone(&self) -> Self {
        Self {
            inner_path: self.inner_path.clone(),
        }
    }
}

impl<K: Eq + Clone + Hash, V, P: Path<Out = HashMap<K, V>> + Copy> Copy
    for HashMapPathBuilder<K, V, P>
{
}

impl<K: Eq + Clone + Hash, V: Trackable, P: Path<Out = HashMap<K, V>>> HashMapPathBuilder<K, V, P> {
    pub fn key(self, key: K) -> V::PathBuilder<HashMapKeyMapper<K, V, P>> {
        V::new_path_builder(HashMapKeyMapper {
            parent: self.inner_path,
            key,
        })
    }
}

impl<K: Eq + Clone + Hash, V, P: Path<Out = HashMap<K, V>>> HashMapPathBuilder<K, V, P> {
    pub fn key_shallow(self, key: K) -> LeafPathBuilder<HashMapKeyMapper<K, V, P>> {
        LeafPathBuilder::new(HashMapKeyMapper {
            parent: self.inner_path,
            key,
        })
    }
}

pub struct HashMapKeyMapper<K: Eq + Clone + Hash, V, P: Path<Out = HashMap<K, V>>> {
    parent: P,
    key: K,
}

impl<K: Eq + Clone + Hash, V, P: Path<Out = HashMap<K, V>> + Clone> Clone
    for HashMapKeyMapper<K, V, P>
{
    fn clone(&self) -> Self {
        Self {
            parent: self.parent.clone(),
            key: self.key.clone(),
        }
    }
}
impl<K: Eq + Clone + Hash + Copy, V, P: Path<Out = HashMap<K, V>> + Copy> Copy
    for HashMapKeyMapper<K, V, P>
{
}

impl<K: Eq + Clone + Hash, V, P: Path<Out = HashMap<K, V>>> Path for HashMapKeyMapper<K, V, P> {
    type Out = V;

    fn path_borrow(&self) -> Option<Ref<'_, Self::Out>> {
        self.parent
            .path_borrow()
            .and_then(|r| Ref::filter_map(r, |hm| hm.get(&self.key)).ok())
    }

    fn path_borrow_mut(&self) -> Option<RefMut<'_, Self::Out>> {
        self.parent
            .path_borrow_mut()
            .and_then(|r| RefMut::filter_map(r, |hm| hm.get_mut(&self.key)).ok())
    }

    fn visit_hashes(&self, visitor: &mut crate::hash_visitor::HashVisitor) {
        self.parent.visit_hashes(visitor);
        self.key.hash(&mut **visitor);
        visitor.finish_one();
    }

    fn store_wakers(&self) -> &std::cell::RefCell<crate::wakers::StoreWakers> {
        self.parent.store_wakers()
    }
}
