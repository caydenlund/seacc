use std::ops::Add;

// unique identifier of an open file
pub type FileId = usize;

// span of start..end - i.e., [start, end)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub file: FileId,
    pub start: usize,
    pub end: usize,
}

impl Add for Span {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            file: self.file,
            start: self.start.min(rhs.start),
            end: self.end.max(rhs.end),
        }
    }
}

// item that has a span
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

// maps byte indices to lines/columns in the source file.
// each element is the start of a line.
// the last element corresponds to the end of the file.
#[derive(Clone)]
#[repr(transparent)]
pub struct LineMap(Box<[usize]>);

impl From<Box<[usize]>> for LineMap {
    fn from(value: Box<[usize]>) -> Self {
        Self(value)
    }
}

impl AsRef<[usize]> for LineMap {
    fn as_ref(&self) -> &[usize] {
        &self.0
    }
}

impl LineMap {
    // returns the line and column of the given byte by offset, or none if the byte offset is out-of-bounds
    #[must_use]
    pub fn line_col(&self, offset: usize) -> Option<(usize, usize)> {
        if offset > self.0.last().copied()? {
            return None;
        }
        match self.0.binary_search(&offset) {
            Ok(i) => Some((i + 1, 1)),
            Err(i) => Some((i, offset - self.0[i - 1] + 1)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_col() {
        let line_map = LineMap::from(vec![0, 5, 10, 20].into_boxed_slice());
        assert_eq!(line_map.line_col(0), Some((1, 1)));
        assert_eq!(line_map.line_col(4), Some((1, 5)));
        assert_eq!(line_map.line_col(5), Some((2, 1)));
        assert_eq!(line_map.line_col(9), Some((2, 5)));
        assert_eq!(line_map.line_col(10), Some((3, 1)));
        assert_eq!(line_map.line_col(19), Some((3, 10)));
        assert_eq!(line_map.line_col(20), Some((4, 1)));
        assert_eq!(line_map.line_col(21), None);
    }
}
