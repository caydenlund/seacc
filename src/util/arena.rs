use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

/// A generational arena
///
/// Supports insertion and removal.
/// If elements are removed, their space can be reused; trying to access that element by ID later will report `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arena<T> {
    data: Vec<Option<T>>,
    generation: Vec<u32>,
    free: Vec<u32>,
}

impl<T> Arena<T> {
    /// Initializes a new generational arena
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes a new generational arena with the given `capacity`
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            generation: Vec::with_capacity(capacity),
            free: Vec::new(),
        }
    }

    /// Reports the number of elements in this arena
    #[must_use]
    pub const fn len(&self) -> usize {
        self.data.len() - self.free.len()
    }

    /// Reports whether this arena has no elements in it
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.len() == self.free.len()
    }

    /// Inserts the given element into the arena at an arbitrary index, returning a corresponding unique generational ID
    ///
    /// # Panics
    /// If there more than 2^32-1 items in the arena
    pub fn insert(&mut self, item: T) -> ArenaId<T> {
        if let Some(idx) = self.free.pop() {
            let idx = idx as usize;
            debug_assert!(self.generation[idx].is_multiple_of(2));
            debug_assert!(self.data[idx].is_none());
            self.data[idx] = Some(item);
            self.generation[idx] += 1;
            ArenaId::new(idx, self.generation[idx])
        } else {
            self.data.push(Some(item));
            self.generation.push(1);
            ArenaId::new(self.data.len() - 1, 1)
        }
    }

    /// Removes the given element (by `id`) from the arena, returning the removed element if successful
    pub fn remove(&mut self, id: ArenaId<T>) -> Option<T> {
        let (idx, generation) = id.into_parts();
        debug_assert!(!generation.is_multiple_of(2));
        if idx >= self.data.len() || generation != self.generation[idx] {
            return None;
        }
        let mut old = None;
        std::mem::swap(&mut old, &mut self.data[idx]);
        debug_assert!(old.is_some());
        self.generation[idx] += 1;
        self.free.push(id.idx);
        old
    }

    /// Returns a reference the given element (by `id`) in the arena if the generations match
    #[must_use]
    pub fn get(&self, id: ArenaId<T>) -> Option<&T> {
        let (idx, generation) = id.into_parts();
        debug_assert!(!generation.is_multiple_of(2));
        if idx >= self.data.len() || generation != self.generation[idx] {
            None
        } else {
            debug_assert!(self.data[idx].is_some());
            self.data[idx].as_ref()
        }
    }

    /// Returns a mutable reference the given element (by id) in the arena if the generations match
    #[must_use]
    pub fn get_mut(&mut self, id: ArenaId<T>) -> Option<&mut T> {
        let (idx, generation) = id.into_parts();
        debug_assert!(!generation.is_multiple_of(2));
        if idx >= self.data.len() || generation != self.generation[idx] {
            None
        } else {
            debug_assert!(self.data[idx].is_some());
            self.data[idx].as_mut()
        }
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            generation: Vec::new(),
            free: Vec::new(),
        }
    }
}

// ================================================================================

pub struct ArenaId<T> {
    idx: u32,
    generation: u32,
    _t: PhantomData<T>,
}

impl<T> ArenaId<T> {
    pub(crate) fn new(idx: usize, generation: u32) -> Self {
        Self {
            idx: u32::try_from(idx).unwrap(),
            generation,
            _t: PhantomData,
        }
    }

    pub(crate) const fn into_parts(self) -> (usize, u32) {
        (self.idx as usize, self.generation)
    }
}

impl<T> Debug for ArenaId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArenaId")
            .field("idx", &self.idx)
            .field("generation", &self.generation)
            .finish()
    }
}

impl<T> Clone for ArenaId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ArenaId<T> {}

impl<T> PartialEq for ArenaId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.generation == other.generation
    }
}

impl<T> Eq for ArenaId<T> {}

impl<T> Hash for ArenaId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.idx.hash(state);
        self.generation.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::Arena;

    #[test]
    fn insert() {
        let mut arena = Arena::new();
        let first = arena.insert("first");
        let second = arena.insert("second");

        assert_eq!(arena.get(first), Some(&"first"));
        assert_eq!(arena.get(second), Some(&"second"));
        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn remove() {
        let mut arena = Arena::new();
        let id = arena.insert(42);

        assert_eq!(arena.remove(id), Some(42));
        assert!(arena.is_empty());
        assert_eq!(arena.get(id), None);
        assert_eq!(arena.remove(id), None);
    }

    #[test]
    fn reuse_slot() {
        let mut arena = Arena::new();
        let old_id = arena.insert("old");
        arena.remove(old_id);
        let new_id = arena.insert("new");

        assert_eq!(arena.len(), 1);
        assert_ne!(old_id, new_id);
        assert_eq!(arena.get(old_id), None);
        assert_eq!(arena.get(new_id), Some(&"new"));
    }

    #[test]
    fn mutate_values() {
        let mut arena = Arena::with_capacity(4);
        let id = arena.insert(String::from("before"));

        arena.get_mut(id).unwrap().push_str(" after");

        assert_eq!(arena.get(id).map(String::as_str), Some("before after"));
    }
}
