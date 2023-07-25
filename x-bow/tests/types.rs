use std::collections::HashMap;

use x_bow::Trackable;

#[derive(Trackable, Default)]
#[track(deep)]
pub struct Root {
    pub field_1: Struct1,
    pub field_2: Vec<Enum2>,
}

#[derive(Trackable, Default)]
#[track(deep)]
pub struct Struct1 {
    pub field_11: String,
    pub field_12: Vec<()>,
}

#[derive(Trackable, Default)]
#[track(deep)]
pub struct Struct3<T> {
    pub data: HashMap<i32, T>,
}

#[derive(Trackable)]
#[track(deep)]
pub enum Enum2 {
    VariantA(Struct1),
    VariantB { field: Struct3<String> },
    VariantC,
}

impl Default for Enum2 {
    fn default() -> Self {
        Self::VariantC
    }
}
