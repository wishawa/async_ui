use x_bow_macros::Trackable;
#[allow(dead_code)]
#[derive(Trackable)]
#[x_bow(module_prefix = crate::__private_macro_only)]
#[x_bow(remote_type = Option)]
#[track(deep)]
pub enum ImitateOption<T> {
    Some(T),
    None,
}
