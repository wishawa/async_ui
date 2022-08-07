#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u64);

impl Version {
    pub fn is_null(self) -> bool {
        self.0 == 0
    }
    pub fn incremented(self) -> Self {
        Self(self.0 + 1)
    }
    pub const fn new_null() -> Self {
        Self(0)
    }
    pub const fn new() -> Self {
        Self(1)
    }
}
impl Default for Version {
    fn default() -> Self {
        Self::new_null()
    }
}
