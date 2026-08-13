use std::{collections::HashMap, ops::Index};

/// A key referring to a unique string in the [`StringIntern`]
///
/// The private field enforces private construction, guaranteeing in-bounds access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StringRef(u32);

/// A string interner
///
/// Because programs often reuse the same strings frequently (e.g., variable names),
/// we cache them in this lookup table here to avoid copying and passing around duplicates.
#[derive(Default, Clone)]
pub struct StringIntern {
    strings: Vec<String>,
    ids: HashMap<String, StringRef>,
}

impl StringIntern {
    /// Construct a new string interner
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a string into the string interner, resolving duplicates
    ///
    /// Returns a key to the inserted string.
    pub fn insert(&mut self, s: impl Into<String>) -> StringRef {
        let s: String = s.into();
        if let Some(sr) = self.ids.get(&s) {
            return *sr;
        }
        #[allow(clippy::cast_possible_truncation)]
        let sr = StringRef(self.strings.len() as u32);
        self.ids.insert(s, sr);
        sr
    }

    /// Gets the string associated with a particular string key
    ///
    /// No need for `Option` since [`StringRef`] will always be in-bounds.
    #[must_use]
    #[inline]
    pub fn get(&self, sref: StringRef) -> &String {
        &self.strings[sref.0 as usize]
    }
}

impl Index<StringRef> for StringIntern {
    type Output = String;

    fn index(&self, sref: StringRef) -> &Self::Output {
        self.get(sref)
    }
}
