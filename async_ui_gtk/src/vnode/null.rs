use async_ui_core::local::control::position::PositionIndices;
use gtk::Widget;

use super::VNodeDispatch;

#[derive(Debug)]
pub(crate) struct NullVNode;
impl VNodeDispatch for NullVNode {
    fn dispatch_ins_node(&self, _position: PositionIndices, _node: Widget) {
        panic!("Null VNode called");
    }
    fn dispatch_del_node(&self, _position: PositionIndices) -> Widget {
        panic!("Null VNode called");
    }
}
