/// A physical byte offset into the physical source buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteOffset(pub u32);

/// A half-open span `[start, end)` of physical bytes in the source buffer,
/// including a reference to the source file path
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span<'src> {
    pub file: &'src str,
    pub start: ByteOffset,
    pub end: ByteOffset,
}

impl<'src> Span<'src> {
    #[must_use]
    pub const fn new(file: &'src str, start: ByteOffset, end: ByteOffset) -> Self {
        debug_assert!(start.0 <= end.0);
        Self { file, start, end }
    }
}
