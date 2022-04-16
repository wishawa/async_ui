use scoped_tls::ScopedKey;

use super::control::Control;

pub trait Backend: Sized + 'static {
    type NodeType: Clone + 'static;
    fn get_tls() -> &'static ScopedKey<Control<Self>>;
    fn get_dummy_control() -> Control<Self>;
}
