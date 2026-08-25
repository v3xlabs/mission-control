#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Persisted {
    ToDisk,
    MemoryOnly,
}
