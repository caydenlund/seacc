// unique identifier of an open file
pub type FileId = usize;

// span of start..end - i.e., [start, end)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub file: FileId,
    pub start: usize,
    pub end: usize,
}
