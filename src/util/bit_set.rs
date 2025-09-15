//! util/bit-set: Definition of a small copyable [`BitSet`]

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl, Shr};

/// A trait for types that can be used as elements in a [`BitSet`]
pub trait BitElem:
    Sized
    + BitAnd<Output = Self>
    + BitAndAssign
    + BitOr<Output = Self>
    + BitOrAssign
    + BitXor<Output = Self>
    + BitXorAssign
    + Copy
    + Clone
    + Eq
    + PartialEq
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + 'static
{
    /// The number of bits in this element type
    const NUM_BITS: usize;
    /// The value with all bits set to 0
    const NONE: Self;
    /// The value with all bits set to 1
    const ALL: Self;
    /// The value with only the least significant bit set to 1
    const UNIT: Self;
}

impl BitElem for u8 {
    const NUM_BITS: usize = Self::BITS as usize;
    const NONE: Self = Self::MIN;
    const ALL: Self = Self::MAX;
    const UNIT: Self = 1;
}

impl BitElem for u16 {
    const NUM_BITS: usize = Self::BITS as usize;
    const NONE: Self = Self::MIN;
    const ALL: Self = Self::MAX;
    const UNIT: Self = 1;
}

impl BitElem for u32 {
    const NUM_BITS: usize = Self::BITS as usize;
    const NONE: Self = Self::MIN;
    const ALL: Self = Self::MAX;
    const UNIT: Self = 1;
}

impl BitElem for u64 {
    const NUM_BITS: usize = Self::BITS as usize;
    const NONE: Self = Self::MIN;
    const ALL: Self = Self::MAX;
    const UNIT: Self = 1;
}

/// Iterator over the set bits in a [`BitSet`]
///
/// Returns the indices of bits that are set to 1, in ascending order.
#[derive(Clone)]
pub struct Iter<'a, E: BitElem, const N: usize> {
    /// Reference to the bit set being iterated
    set: &'a BitSet<E, N>,
    /// Index of the next bit to check
    next_ind: usize,
}

impl<'a, E: BitElem, const N: usize> Iter<'a, E, N> {
    /// Creates a new iterator for the given bit set
    const fn new(set: &'a BitSet<E, N>) -> Self {
        Self { set, next_ind: 0 }
    }
}

impl<E: BitElem, const N: usize> Iterator for Iter<'_, E, N> {
    type Item = usize;

    /// Returns the next set bit index, or `None` if no more set bits exist
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_ind >= BitSet::<E, N>::NUM_BITS {
            return None;
        }
        while !self.set.get(self.next_ind) {
            self.next_ind += 1;
            if self.next_ind >= BitSet::<E, N>::NUM_BITS {
                return None;
            }
        }
        self.next_ind += 1;
        Some(self.next_ind - 1)
    }
}

/// A fixed-size bit set implemented using an array of bit elements
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BitSet<E: BitElem, const N: usize> {
    /// Array of elements storing the bits
    elems: [E; N],
}

impl<E: BitElem, const N: usize> BitSet<E, N> {
    /// A bit set with all bits set to 0
    pub const NONE: Self = Self {
        elems: [E::NONE; N],
    };
    /// A bit set with all bits set to 1
    pub const ALL: Self = Self { elems: [E::ALL; N] };
    /// The total number of bits in this bit set
    pub const NUM_BITS: usize = E::NUM_BITS * N;

    /// Creates a new empty bit set with all bits set to 0
    #[must_use]
    pub const fn new() -> Self {
        Self::NONE
    }

    /// Sets the bit at the given index to 1
    ///
    /// Returns the previous value of the bit (true if it was 1, false if it was 0).
    #[inline]
    pub fn set(&mut self, bit: usize) -> bool {
        let old = self.get(bit);
        self.elems[bit / E::NUM_BITS] |= E::UNIT << (bit % E::NUM_BITS);
        old
    }

    /// Gets the value of the bit at the given index
    ///
    /// Returns true if the bit is 1, false if it is 0.
    #[inline]
    pub fn get(&self, bit: usize) -> bool {
        (self.elems[bit / E::NUM_BITS] >> (bit % E::NUM_BITS)) & E::UNIT == E::UNIT
    }

    /// Returns an iterator over the indices of set bits
    pub const fn iter(&self) -> Iter<'_, E, N> {
        Iter::new(self)
    }
}

impl<'a, E: BitElem, const N: usize> IntoIterator for &'a BitSet<E, N> {
    type Item = usize;
    type IntoIter = Iter<'a, E, N>;

    /// Returns an iterator over the indices of set bits
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Elem: BitElem, const N: usize> Default for BitSet<Elem, N> {
    /// Returns an empty bit set (equivalent to [`BitSet::NONE`])
    fn default() -> Self {
        Self::NONE
    }
}
