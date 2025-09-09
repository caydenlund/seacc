use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl, Shr};

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
    const NUM_BITS: usize;
    const NONE: Self;
    const ALL: Self;
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

#[derive(Clone)]
pub struct Iter<'a, E: BitElem, const N: usize> {
    set: &'a BitSet<E, N>,
    next_ind: usize,
}

impl<'a, E: BitElem, const N: usize> Iter<'a, E, N> {
    fn new(set: &'a BitSet<E, N>) -> Self {
        Self { set, next_ind: 0 }
    }
}

impl<'a, E: BitElem, const N: usize> Iterator for Iter<'a, E, N> {
    type Item = usize;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BitSet<E: BitElem, const N: usize> {
    elems: [E; N],
}

impl<E: BitElem, const N: usize> BitSet<E, N> {
    pub const NONE: Self = Self { elems: [E::NONE; N] };
    pub const ALL: Self = Self { elems: [E::ALL; N] };
    pub const NUM_BITS: usize = E::NUM_BITS * N;

    pub fn new() -> Self {
        Self::NONE
    }

    #[inline(always)]
    pub fn set(&mut self, bit: usize) -> bool {
        let old = self.get(bit);
        self.elems[bit / E::NUM_BITS] |= E::UNIT << (bit % E::NUM_BITS);
        old
    }

    #[inline(always)]
    pub fn get(&self, bit: usize) -> bool {
        (self.elems[bit / E::NUM_BITS] >> (bit % E::NUM_BITS)) & E::UNIT == E::UNIT
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, E, N> {
        Iter::new(self)
    }
}

impl<Elem: BitElem, const N: usize> Default for BitSet<Elem, N> {
    fn default() -> Self {
        Self::NONE
    }
}
