use crate::{
    mapper::Mapper,
    node_down::NodeDownTrait,
    node_up::{NodeUp, NodeUpTrait},
    track_api::Store,
    trackable::Trackable,
};

mod vec {
    use crate::is_guaranteed::IsGuaranteed;

    use super::*;
    use std::{
        cell::RefCell,
        collections::{btree_map::Entry, BTreeMap},
        marker::PhantomData,
        rc::{Rc, Weak},
    };

    pub struct TrackedVec<'u, T: Trackable + 'u, const G: bool> {
        items: Rc<RefCell<BTreeMap<usize, Weak<Store<'u, T, false>>>>>,
        up_node: Rc<dyn NodeUpTrait<Data = Vec<T>> + 'u>,
    }

    impl<'u, T: Trackable + 'u, const G: bool> Clone for TrackedVec<'u, T, G> {
        fn clone(&self) -> Self {
            Self {
                items: self.items.clone(),
                up_node: self.up_node.clone(),
            }
        }
    }

    impl<'u, T: Trackable + 'u, const G: bool> IsGuaranteed<G> for TrackedVec<'u, T, G> {}

    pub struct MapperVec<T> {
        index: usize,
        _phantom: PhantomData<T>,
    }
    impl<T> Mapper for MapperVec<T> {
        type In = Vec<T>;
        type Out = T;
        fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
            input.get(self.index)
        }
        fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
            input.get_mut(self.index)
        }
    }
    impl<'u, T: Trackable, const G: bool> NodeDownTrait<'u, Vec<T>> for TrackedVec<'u, T, G> {
        fn invalidate_down(&self) {
            self.items.borrow_mut().retain(|_key, item| {
                if let Some(item) = item.upgrade() {
                    item.node_up().get_listener().invalidate_down();
                    item.invalidate_down();
                    true
                } else {
                    false
                }
            })
        }
        fn node_up(&self) -> &Rc<dyn NodeUpTrait<Data = Vec<T>> + 'u> {
            &self.up_node
        }
    }
    impl<T: Trackable> Trackable for Vec<T> {
        type NodeDown<'u, const G: bool> = TrackedVec<'u, T, G> where Self: 'u;

        fn new_node<'u, const G: bool>(
            up_node: Rc<dyn NodeUpTrait<Data = Self> + 'u>,
        ) -> Self::NodeDown<'u, G>
        where
            Self: 'u,
        {
            TrackedVec {
                items: Default::default(),
                up_node,
            }
        }
    }
    impl<'u, T: Trackable + 'u, const G: bool> TrackedVec<'u, T, G> {
        fn create_item(&self, index: usize) -> Rc<Store<'u, T, false>> {
            Rc::new(T::new_node(Rc::new(NodeUp::new(
                self.up_node.clone(),
                MapperVec {
                    index,
                    _phantom: PhantomData,
                },
            ))))
        }
        pub fn track_index(&self, index: usize) -> Rc<Store<'u, T, false>> {
            match self.items.borrow_mut().entry(index) {
                Entry::Vacant(vacant) => {
                    let tracked = self.create_item(index);
                    vacant.insert(Rc::downgrade(&tracked));
                    tracked
                }
                Entry::Occupied(mut occupied) => {
                    let value = occupied.get_mut();
                    if let Some(tracked) = value.upgrade() {
                        tracked
                    } else {
                        let tracked = self.create_item(index);
                        *value = Rc::downgrade(&tracked);
                        tracked
                    }
                }
            }
        }
    }
}

mod hashmap {
    use std::{
        cell::RefCell,
        collections::{hash_map::Entry, HashMap},
        hash::Hash,
        marker::PhantomData,
        rc::{Rc, Weak},
    };

    use crate::is_guaranteed::IsGuaranteed;

    use super::*;

    pub struct TrackedHashMap<'u, K: Eq + Hash, V: Trackable + 'u, const G: bool> {
        items: Rc<RefCell<HashMap<K, Weak<Store<'u, V, false>>>>>,
        up_node: Rc<dyn NodeUpTrait<Data = HashMap<K, V>> + 'u>,
    }

    impl<'u, K: Eq + Hash, V: Trackable + 'u, const G: bool> Clone for TrackedHashMap<'u, K, V, G> {
        fn clone(&self) -> Self {
            Self {
                items: self.items.clone(),
                up_node: self.up_node.clone(),
            }
        }
    }

    impl<'u, K: Eq + Hash, V: Trackable + 'u, const G: bool> IsGuaranteed<G>
        for TrackedHashMap<'u, K, V, G>
    {
    }

    pub struct MapperHashMap<K, V> {
        key: K,
        _phantom: PhantomData<V>,
    }
    impl<K: Eq + Hash, V> Mapper for MapperHashMap<K, V> {
        type In = HashMap<K, V>;
        type Out = V;
        fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
            input.get(&self.key)
        }
        fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
            input.get_mut(&self.key)
        }
    }
    impl<'u, K: Eq + Hash, V: Trackable, const G: bool> NodeDownTrait<'u, HashMap<K, V>>
        for TrackedHashMap<'u, K, V, G>
    {
        fn invalidate_down(&self) {
            self.items.borrow_mut().retain(|_key, item| {
                if let Some(item) = item.upgrade() {
                    item.node_up().get_listener().invalidate_down();
                    item.invalidate_down();
                    true
                } else {
                    false
                }
            })
        }

        fn node_up(&self) -> &Rc<dyn NodeUpTrait<Data = HashMap<K, V>> + 'u> {
            &self.up_node
        }
    }

    impl<K: Eq + Hash, V: Trackable> Trackable for HashMap<K, V> {
        type NodeDown<'u, const G: bool> = TrackedHashMap<'u, K, V, G> where Self : 'u;

        fn new_node<'u, const G: bool>(
            up_node: Rc<dyn NodeUpTrait<Data = Self> + 'u>,
        ) -> Self::NodeDown<'u, G>
        where
            Self: 'u,
        {
            TrackedHashMap {
                items: Default::default(),
                up_node,
            }
        }
    }
    impl<'u, K: Eq + Hash + Clone + 'u, V: Trackable + 'u, const G: bool> TrackedHashMap<'u, K, V, G> {
        fn create_item(&self, key: K) -> Rc<Store<'u, V, false>> {
            Rc::new(V::new_node(Rc::new(NodeUp::new(
                self.up_node.clone(),
                MapperHashMap {
                    key,
                    _phantom: PhantomData,
                },
            ))))
        }
        pub fn track_key(&self, key: K) -> Rc<Store<'u, V, false>> {
            match self.items.borrow_mut().entry(key.clone()) {
                Entry::Vacant(vacant) => {
                    let tracked = self.create_item(key);
                    vacant.insert(Rc::downgrade(&tracked));
                    tracked
                }
                Entry::Occupied(mut occupied) => {
                    let value = occupied.get_mut();
                    if let Some(tracked) = value.upgrade() {
                        tracked
                    } else {
                        let tracked = self.create_item(key);
                        *value = Rc::downgrade(&tracked);
                        tracked
                    }
                }
            }
        }
    }
}