enum Empty {}
#[derive(Clone, Copy)]
pub struct Pointer {
    start: *const Empty,
    size: usize,
}
impl Pointer {
    pub fn new<T: ?Sized>(ptr: &T) -> Self {
        Self {
            start: ptr as *const T as *const Empty,
            size: std::mem::size_of_val(ptr),
        }
    }
    pub fn contains(&self, other: Pointer) -> bool {
        let ts = self.start as usize;
        let os = other.start as usize;
        (os >= ts) && (os + other.size <= ts + self.size)
    }
}
