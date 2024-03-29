#[derive(Clone, Copy, Debug)]
pub(crate) struct WakerHashEntry(u64);
impl WakerHashEntry {
    pub fn value(self) -> u64 {
        self.0
    }
    pub fn regular_from(hash: u64, depth: u8) -> Self {
        const INVERT_DEPTH_MASK: u64 = !(u8::MAX as u64);
        Self((hash & INVERT_DEPTH_MASK) | (depth as u64))
    }
    pub fn bubbling_from(hash: u64) -> Self {
        // XOR with a randomly-generated number so that the `bubbling` value
        // is different from the `regular` variant.
        Self(hash ^ 0x33b741db0040f7e)
    }
}
