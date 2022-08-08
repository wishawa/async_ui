use std::{
    any::{Any, TypeId},
    rc::Rc,
};

use im_rc::HashMap;
#[derive(Clone, Default)]
pub struct ContextMap {
    pub(crate) inner: HashMap<TypeId, Rc<dyn Any>>,
}
