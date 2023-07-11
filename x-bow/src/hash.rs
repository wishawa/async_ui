#[derive(Clone, Copy, Debug)]
pub(crate) struct WakerHashEntry(u64);
impl WakerHashEntry {
    pub fn value(self) -> u64 {
        self.0
    }
    pub fn regular_from(hash: u64) -> Self {
        Self(hash)
    }
    pub fn bubbling_from(hash: u64) -> Self {
        Self(hash ^ 0b1)
    }
}
