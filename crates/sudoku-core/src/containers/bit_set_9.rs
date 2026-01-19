//! Generic 9-bit set implementation.
//!
//! Provides [`BitSet9`], a generic bitset for representing sets of up to 9 elements,
//! parameterized by [`Index9Semantics`].
//!
//! [`Index9Semantics`]: crate::index::Index9Semantics

use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    iter::FusedIterator,
    marker::PhantomData,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, RangeBounds},
};

use crate::index::{Index9, Index9Semantics};

/// A generic set of up to 9 elements, represented as a bitset.
///
/// This type uses a 16-bit integer where bits 0-8 represent the 9 possible elements.
/// The specific semantics of the elements are determined by the `S` type parameter,
/// which must implement [`Index9Semantics`].
///
/// # Type Parameters
///
/// * `S` - The semantics implementation (from [`index`] module)
///   that defines how values are converted to and from bit indices.
///
/// [`index`]: crate::index
///
/// # Examples
///
/// See the [`digit_candidates`] module for a concrete example using [`DigitCandidates`].
///
/// [`digit_candidates`]: crate::digit_candidates
/// [`DigitCandidates`]: crate::digit_candidates::DigitCandidates
///
/// For information on defining custom semantics, see [`Index9Semantics`].
pub struct BitSet9<S>
where
    S: Index9Semantics,
{
    bits: u16,
    _marker: PhantomData<S>,
}

impl<S> Copy for BitSet9<S> where S: Index9Semantics {}

impl<S> Clone for BitSet9<S>
where
    S: Index9Semantics,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> PartialEq for BitSet9<S>
where
    S: Index9Semantics,
{
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl<S> Eq for BitSet9<S> where S: Index9Semantics {}

impl<S> Hash for BitSet9<S>
where
    S: Index9Semantics,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bits.hash(state);
    }
}

impl<S> Default for BitSet9<S>
where
    S: Index9Semantics,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> BitSet9<S>
where
    S: Index9Semantics,
{
    /// An empty set containing no elements.
    pub const EMPTY: Self = Self::from_bits(0);

    /// A full set containing all 9 possible elements.
    pub const FULL: Self = Self::from_bits(0x1ff);

    const fn from_bits(bits: u16) -> Self {
        assert!(bits <= 0x1ff);
        Self {
            bits,
            _marker: PhantomData,
        }
    }

    /// Creates a new empty set.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self::EMPTY
    }

    /// Returns a new set containing only the elements in this set that fall within the given range.
    #[must_use]
    pub fn range<R>(self, range: R) -> Self
    where
        R: RangeBounds<S::Value>,
        S::Value: PartialOrd,
    {
        let mut result = Self::new();
        for n in self {
            if range.contains(&n) {
                result.insert(n);
            }
        }
        result
    }

    /// Returns the difference of two sets.
    ///
    /// Returns a new set containing elements in `self` but not in `other`.
    #[must_use]
    #[inline]
    pub const fn difference(self, other: Self) -> Self {
        Self::from_bits(self.bits & !other.bits)
    }

    /// Returns the symmetric difference of two sets.
    ///
    /// Returns a new set containing elements in either `self` or `other`, but not in both.
    #[must_use]
    #[inline]
    pub const fn symmetric_difference(self, other: Self) -> Self {
        Self::from_bits(self.bits ^ other.bits)
    }

    /// Returns the intersection of two sets.
    ///
    /// Returns a new set containing elements in both `self` and `other`.
    #[must_use]
    #[inline]
    pub const fn intersection(self, other: Self) -> Self {
        Self::from_bits(self.bits & other.bits)
    }

    /// Returns the union of two sets.
    ///
    /// Returns a new set containing elements in either `self` or `other`.
    #[must_use]
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self::from_bits(self.bits | other.bits)
    }

    /// Clears the set, removing all elements.
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::EMPTY;
    }

    /// Returns `true` if the set contains the specified element.
    ///
    /// # Panics
    ///
    /// Panics if the value cannot be converted to a valid bit index by the semantics.
    #[must_use]
    #[inline]
    pub fn contains(self, n: S::Value) -> bool {
        let i = S::to_index(n);
        (self.bits & i.bit()) != 0
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    #[must_use]
    #[inline]
    pub const fn is_disjoint(self, other: Self) -> bool {
        self.intersection(other).is_empty()
    }

    /// Returns `true` if the set is a subset of another.
    #[must_use]
    #[inline]
    pub const fn is_subset(self, other: Self) -> bool {
        self.union(other).bits == other.bits
    }

    /// Returns `true` if the set is a superset of another.
    #[must_use]
    #[inline]
    pub const fn is_superset(self, other: Self) -> bool {
        self.union(other).bits == self.bits
    }

    const fn first_index(self) -> Option<Index9> {
        if self.bits == 0 {
            return None;
        }
        #[expect(clippy::cast_possible_truncation)]
        Some(Index9::new(self.bits.trailing_zeros() as u8))
    }

    const fn last_index(self) -> Option<Index9> {
        if self.bits == 0 {
            return None;
        }
        #[expect(clippy::cast_possible_truncation)]
        Some(Index9::new(15 - self.bits.leading_zeros() as u8))
    }

    /// Returns the smallest element in the set, if any.
    #[must_use]
    #[inline]
    pub fn first(self) -> Option<S::Value> {
        self.first_index().map(S::from_index)
    }

    /// Returns the largest element in the set, if any.
    #[must_use]
    #[inline]
    pub fn last(self) -> Option<S::Value> {
        self.last_index().map(S::from_index)
    }

    /// Removes and returns the smallest element in the set, if any.
    #[inline]
    pub fn pop_first(&mut self) -> Option<S::Value> {
        let i = self.first_index()?;
        self.bits &= !i.bit();
        Some(S::from_index(i))
    }

    /// Removes and returns the largest element in the set, if any.
    #[inline]
    pub fn pop_last(&mut self) -> Option<S::Value> {
        let i = self.last_index()?;
        self.bits &= !i.bit();
        Some(S::from_index(i))
    }

    /// Adds an element to the set.
    ///
    /// Returns `true` if the element was not already in the set.
    ///
    /// # Panics
    ///
    /// Panics if the value cannot be converted to a valid bit index by the semantics.
    #[inline]
    pub fn insert(&mut self, n: S::Value) -> bool {
        let i = S::to_index(n);
        let old = self.bits;
        self.bits |= i.bit();
        old != self.bits
    }

    /// Removes an element from the set.
    ///
    /// Returns `true` if the element was present in the set.
    ///
    /// # Panics
    ///
    /// Panics if the value cannot be converted to a valid bit index by the semantics.
    #[inline]
    pub fn remove(&mut self, n: S::Value) -> bool {
        let i = S::to_index(n);
        let old = self.bits;
        self.bits &= !i.bit();
        old != self.bits
    }

    /// Returns an iterator over the elements of the set in ascending order.
    #[must_use]
    #[inline]
    pub const fn iter(self) -> BitSet9Iter<S> {
        BitSet9Iter { set: self }
    }

    /// Returns the number of elements in the set.
    #[must_use]
    #[inline]
    pub const fn len(self) -> usize {
        self.bits.count_ones() as usize
    }

    /// Returns `true` if the set contains no elements.
    #[must_use]
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }
}

impl<S> BitAnd for BitSet9<S>
where
    S: Index9Semantics,
{
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.bits & rhs.bits)
    }
}

impl<S> BitAndAssign for BitSet9<S>
where
    S: Index9Semantics,
{
    fn bitand_assign(&mut self, rhs: Self) {
        self.bits &= rhs.bits;
    }
}

impl<S> BitOr for BitSet9<S>
where
    S: Index9Semantics,
{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.bits | rhs.bits)
    }
}

impl<S> BitOrAssign for BitSet9<S>
where
    S: Index9Semantics,
{
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

impl<S> BitXor for BitSet9<S>
where
    S: Index9Semantics,
{
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.bits ^ rhs.bits)
    }
}

impl<S> BitXorAssign for BitSet9<S>
where
    S: Index9Semantics,
{
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bits ^= rhs.bits;
    }
}

impl<S> Not for BitSet9<S>
where
    S: Index9Semantics,
{
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::from_bits(!self.bits & Self::FULL.bits)
    }
}

impl<S> Debug for BitSet9<S>
where
    S: Index9Semantics,
    S::Value: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<S> IntoIterator for &BitSet9<S>
where
    S: Index9Semantics,
{
    type IntoIter = BitSet9Iter<S>;
    type Item = S::Value;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<S> IntoIterator for BitSet9<S>
where
    S: Index9Semantics,
{
    type IntoIter = BitSet9Iter<S>;
    type Item = S::Value;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the elements of a [`BitSet9`].
///
/// This iterator yields elements in ascending order and supports
/// double-ended iteration.
pub struct BitSet9Iter<S>
where
    S: Index9Semantics,
{
    set: BitSet9<S>,
}

impl<S> Debug for BitSet9Iter<S>
where
    S: Index9Semantics,
    S::Value: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BitSet9Iter")
            .field("set", &self.set)
            .finish()
    }
}

impl<S> Iterator for BitSet9Iter<S>
where
    S: Index9Semantics,
{
    type Item = S::Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.set.pop_first()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.set.len();
        (len, Some(len))
    }
}

impl<S> DoubleEndedIterator for BitSet9Iter<S>
where
    S: Index9Semantics,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.set.pop_last()
    }
}

impl<S> ExactSizeIterator for BitSet9Iter<S> where S: Index9Semantics {}
impl<S> FusedIterator for BitSet9Iter<S> where S: Index9Semantics {}

impl<S> FromIterator<S::Value> for BitSet9<S>
where
    S: Index9Semantics,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = S::Value>,
    {
        let mut set = BitSet9::EMPTY;
        for n in iter {
            set.insert(n);
        }
        set
    }
}

impl<S> Extend<S::Value> for BitSet9<S>
where
    S: Index9Semantics,
{
    fn extend<T: IntoIterator<Item = S::Value>>(&mut self, iter: T) {
        for value in iter {
            self.insert(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct TestSemantics;

    impl Index9Semantics for TestSemantics {
        type Value = u8;

        fn to_index(value: Self::Value) -> Index9 {
            assert!(value < 9, "Test value must be 0-8, got {value}");
            Index9::new(value)
        }

        fn from_index(index: Index9) -> Self::Value {
            index.index()
        }
    }

    type TestSet = BitSet9<TestSemantics>;

    // Helper macro to create sets concisely
    macro_rules! set {
        [$($n:expr),* $(,)?] => {
            TestSet::from_iter([$($n),*])
        };
    }

    mod bit_index {
        use super::*;

        #[test]
        fn test_all() {
            let indices: Vec<_> = Index9::all().collect();
            assert_eq!(indices.len(), 9);
            for (i, idx) in (0..).zip(indices) {
                assert_eq!(idx.index(), i);
            }
        }

        #[test]
        fn test_all_creates_valid_indices() {
            for idx in Index9::all() {
                assert!(idx.index() < 9);
            }
        }
    }

    mod construction {
        use super::*;

        #[test]
        fn test_new_is_empty() {
            let set = TestSet::new();
            assert!(set.is_empty());
            assert_eq!(set.len(), 0);
        }

        #[test]
        fn test_empty_constant() {
            assert_eq!(TestSet::EMPTY, TestSet::new());
            assert!(TestSet::EMPTY.is_empty());
        }

        #[test]
        fn test_full_constant() {
            let full = TestSet::FULL;
            assert_eq!(full.len(), 9);
            for n in 0..9 {
                assert!(full.contains(n));
            }
        }

        #[test]
        fn test_from_iter() {
            let set = set![0, 2, 4, 6, 8];
            assert_eq!(set.len(), 5);
            assert!(set.contains(0));
            assert!(!set.contains(1));
            assert!(set.contains(2));
        }

        #[test]
        fn test_default() {
            let set = TestSet::default();
            assert_eq!(set, TestSet::EMPTY);
        }
    }

    mod basic_operations {
        use super::*;

        #[test]
        fn test_insert() {
            let mut set = TestSet::new();
            assert!(set.insert(0));
            assert!(!set.insert(0));
            assert_eq!(set.len(), 1);
            assert!(set.contains(0));
        }

        #[test]
        fn test_remove() {
            let mut set = set![0, 1, 2];
            assert!(set.remove(1));
            assert!(!set.remove(1));
            assert_eq!(set.len(), 2);
            assert!(!set.contains(1));
        }

        #[test]
        fn test_contains() {
            let set = set![0, 4, 8];
            assert!(set.contains(0));
            assert!(!set.contains(1));
            assert!(set.contains(4));
            assert!(set.contains(8));
        }

        #[test]
        fn test_clear() {
            let mut set = set![0, 1, 2, 3, 4];
            set.clear();
            assert!(set.is_empty());
            assert_eq!(set.len(), 0);
        }

        #[test]
        #[should_panic(expected = "Test value must be 0-8")]
        fn test_insert_nine_panics() {
            let mut set = TestSet::new();
            set.insert(9);
        }

        #[test]
        #[should_panic(expected = "Test value must be 0-8")]
        fn test_insert_ten_panics() {
            let mut set = TestSet::new();
            set.insert(10);
        }
    }

    mod set_operations {
        use super::*;

        #[test]
        fn test_union() {
            let cases = [
                (set![0, 1], set![1, 2], set![0, 1, 2]),
                (set![0], set![8], set![0, 8]),
                (set![], set![0, 1], set![0, 1]),
                (set![0, 1, 2], set![3, 4, 5], set![0, 1, 2, 3, 4, 5]),
            ];
            for (a, b, expected) in cases {
                assert_eq!(a.union(b), expected);
                assert_eq!(b.union(a), expected); // Commutativity
                assert_eq!(a | b, expected); // Bit operator
            }
        }

        #[test]
        fn test_intersection() {
            let cases = [
                (set![0, 1, 2], set![1, 2, 3], set![1, 2]),
                (set![0, 1], set![2, 3], set![]),
                (set![0, 1, 2], set![0, 1, 2], set![0, 1, 2]),
                (set![], set![0, 1], set![]),
            ];
            for (a, b, expected) in cases {
                assert_eq!(a.intersection(b), expected);
                assert_eq!(b.intersection(a), expected); // Commutativity
                assert_eq!(a & b, expected); // Bit operator
            }
        }

        #[test]
        fn test_difference() {
            let cases = [
                (set![0, 1, 2], set![1, 2, 3], set![0]),
                (set![0, 1, 2], set![3, 4, 5], set![0, 1, 2]),
                (set![0, 1, 2], set![0, 1, 2], set![]),
                (set![], set![0, 1], set![]),
            ];
            for (a, b, expected) in cases {
                assert_eq!(a.difference(b), expected);
            }
        }

        #[test]
        fn test_symmetric_difference() {
            let cases = [
                (set![0, 1, 2], set![1, 2, 3], set![0, 3]),
                (set![0, 1], set![2, 3], set![0, 1, 2, 3]),
                (set![0, 1, 2], set![0, 1, 2], set![]),
                (set![], set![0, 1], set![0, 1]),
            ];
            for (a, b, expected) in cases {
                assert_eq!(a.symmetric_difference(b), expected);
                assert_eq!(b.symmetric_difference(a), expected); // Commutativity
                assert_eq!(a ^ b, expected); // Bit operator
            }
        }

        #[test]
        fn test_not() {
            let set = set![0, 2, 4, 6, 8];
            let complement = !set;
            assert_eq!(complement, set![1, 3, 5, 7]);
            assert_eq!(!TestSet::EMPTY, TestSet::FULL);
            assert_eq!(!TestSet::FULL, TestSet::EMPTY);
        }

        #[test]
        fn test_assign_operators() {
            let mut set = set![0, 1, 2];
            set |= set![2, 3, 4];
            assert_eq!(set, set![0, 1, 2, 3, 4]);

            set &= set![1, 2, 3];
            assert_eq!(set, set![1, 2, 3]);

            set ^= set![2, 3, 4];
            assert_eq!(set, set![1, 4]);
        }
    }

    mod relations {
        use super::*;

        #[test]
        fn test_is_subset() {
            let cases = [
                (set![0, 1], set![0, 1, 2], true),
                (set![0, 1, 2], set![0, 1], false),
                (set![0, 1], set![0, 1], true),
                (set![], set![0, 1], true),
                (set![0, 1], set![2, 3], false),
            ];
            for (a, b, expected) in cases {
                assert_eq!(a.is_subset(b), expected, "{a:?}.is_subset({b:?})");
            }
        }

        #[test]
        fn test_is_superset() {
            let cases = [
                (set![0, 1, 2], set![0, 1], true),
                (set![0, 1], set![0, 1, 2], false),
                (set![0, 1], set![0, 1], true),
                (set![0, 1], set![], true),
                (set![0, 1], set![2, 3], false),
            ];
            for (a, b, expected) in cases {
                assert_eq!(a.is_superset(b), expected, "{a:?}.is_superset({b:?})");
            }
        }

        #[test]
        fn test_is_disjoint() {
            let cases = [
                (set![0, 1], set![2, 3], true),
                (set![0, 1, 2], set![2, 3, 4], false),
                (set![], set![0, 1], true),
                (set![0], set![0], false),
            ];
            for (a, b, expected) in cases {
                assert_eq!(a.is_disjoint(b), expected, "{a:?}.is_disjoint({b:?})");
            }
        }
    }

    mod access {
        use super::*;

        #[test]
        fn test_first() {
            assert_eq!(set![2, 6, 0].first(), Some(0));
            assert_eq!(set![8].first(), Some(8));
            assert_eq!(set![].first(), None);
        }

        #[test]
        fn test_last() {
            assert_eq!(set![2, 6, 0].last(), Some(6));
            assert_eq!(set![0].last(), Some(0));
            assert_eq!(set![].last(), None);
        }

        #[test]
        fn test_pop_first() {
            let mut set = set![2, 6, 0];
            assert_eq!(set.pop_first(), Some(0));
            assert_eq!(set.pop_first(), Some(2));
            assert_eq!(set.pop_first(), Some(6));
            assert_eq!(set.pop_first(), None);
        }

        #[test]
        fn test_pop_last() {
            let mut set = set![2, 6, 0];
            assert_eq!(set.pop_last(), Some(6));
            assert_eq!(set.pop_last(), Some(2));
            assert_eq!(set.pop_last(), Some(0));
            assert_eq!(set.pop_last(), None);
        }

        #[test]
        fn test_range() {
            let set = set![0, 2, 4, 6, 8];
            assert_eq!(set.range(2..=6), set![2, 4, 6]);
            assert_eq!(set.range(2..6), set![2, 4]);
            assert_eq!(set.range(..4), set![0, 2]);
            assert_eq!(set.range(6..), set![6, 8]);
            assert_eq!(set.range(..), set);
        }
    }

    mod iteration {
        use super::*;

        #[test]
        fn test_iter_ascending() {
            let set = set![4, 0, 8, 2];
            let vec: Vec<u8> = set.iter().collect();
            assert_eq!(vec, vec![0, 2, 4, 8]);
        }

        #[test]
        fn test_iter_double_ended() {
            let set = set![0, 2, 4, 6, 8];
            let mut iter = set.iter();
            assert_eq!(iter.next(), Some(0));
            assert_eq!(iter.next_back(), Some(8));
            assert_eq!(iter.next(), Some(2));
            assert_eq!(iter.next_back(), Some(6));
            assert_eq!(iter.next(), Some(4));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
        }

        #[test]
        fn test_iter_size_hint() {
            let set = set![0, 2, 4];
            let iter = set.iter();
            assert_eq!(iter.size_hint(), (3, Some(3)));
            assert_eq!(iter.len(), 3);
        }

        #[test]
        fn test_into_iter() {
            let set = set![0, 2, 4];
            let vec: Vec<u8> = set.into_iter().collect();
            assert_eq!(vec, vec![0, 2, 4]);
        }

        #[test]
        fn test_iter_ref() {
            let set = set![0, 2, 4];
            let vec: Vec<u8> = (&set).into_iter().collect();
            assert_eq!(vec, vec![0, 2, 4]);
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_boundary_values() {
            let mut set = TestSet::new();
            set.insert(0);
            set.insert(8);
            assert_eq!(set.len(), 2);
            assert!(set.contains(0));
            assert!(set.contains(8));
        }

        #[test]
        fn test_all_operations_on_empty() {
            let empty = TestSet::EMPTY;
            assert_eq!(empty.len(), 0);
            assert_eq!(empty.first(), None);
            assert_eq!(empty.last(), None);
            assert_eq!(empty.union(empty), empty);
            assert_eq!(empty.intersection(empty), empty);
            assert_eq!(!empty, TestSet::FULL);
        }

        #[test]
        fn test_all_operations_on_full() {
            let full = TestSet::FULL;
            assert_eq!(full.len(), 9);
            assert_eq!(full.first(), Some(0));
            assert_eq!(full.last(), Some(8));
            assert_eq!(full.union(full), full);
            assert_eq!(full.intersection(full), full);
            assert_eq!(!full, TestSet::EMPTY);
        }

        #[test]
        fn test_single_element_sets() {
            for n in 0..9 {
                let set = set![n];
                assert_eq!(set.len(), 1);
                assert_eq!(set.first(), Some(n));
                assert_eq!(set.last(), Some(n));
                assert!(set.contains(n));
            }
        }
    }

    mod invariants {
        use super::*;

        #[test]
        fn test_len_equals_iter_count() {
            let cases = [
                set![],
                set![0],
                set![0, 1, 2],
                set![0, 2, 4, 6, 8],
                TestSet::FULL,
            ];
            for set in cases {
                assert_eq!(set.len(), set.iter().count());
            }
        }

        #[test]
        fn test_insert_remove_roundtrip() {
            for n in 0..9 {
                let mut set = TestSet::new();
                set.insert(n);
                assert!(set.contains(n));
                set.remove(n);
                assert!(!set.contains(n));
                assert!(set.is_empty());
            }
        }

        #[test]
        fn test_union_size_bound() {
            let a = set![0, 1, 2];
            let b = set![2, 3, 4];
            let u = a.union(b);
            assert!(u.len() >= a.len());
            assert!(u.len() >= b.len());
            assert!(u.len() <= a.len() + b.len());
        }

        #[test]
        fn test_hash() {
            use std::collections::HashSet;

            let set1 = set![0, 1, 2];
            let set2 = set![0, 1, 2];
            let set3 = set![0, 1, 3];

            // Same sets should have same hash
            let mut hash_set = HashSet::new();
            hash_set.insert(set1);
            assert!(hash_set.contains(&set2));
            assert!(!hash_set.contains(&set3));
        }

        #[test]
        fn test_extend() {
            let mut set = set![0, 1, 2];
            set.extend([3, 4, 5]);
            assert_eq!(set, set![0, 1, 2, 3, 4, 5]);
        }

        #[test]
        fn test_extend_overlapping() {
            let mut set = set![0, 1, 2];
            set.extend([2, 3, 4]);
            assert_eq!(set, set![0, 1, 2, 3, 4]);
        }

        #[test]
        fn test_extend_empty() {
            let mut set = set![0, 1, 2];
            set.extend(Vec::<u8>::new());
            assert_eq!(set, set![0, 1, 2]);
        }
    }

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Strategy to generate valid numbers (0-8)
        fn valid_number() -> impl Strategy<Value = u8> {
            0u8..9
        }

        // Strategy to generate TestSet
        fn bit_set_9() -> impl Strategy<Value = TestSet> {
            prop::collection::vec(valid_number(), 0..=9).prop_map(TestSet::from_iter)
        }

        proptest! {
            // Set operations are commutative
            #[test]
            fn prop_union_commutative(a in bit_set_9(), b in bit_set_9()) {
                prop_assert_eq!(a.union(b), b.union(a));
                prop_assert_eq!(a | b, b | a);
            }

            #[test]
            fn prop_intersection_commutative(a in bit_set_9(), b in bit_set_9()) {
                prop_assert_eq!(a.intersection(b), b.intersection(a));
                prop_assert_eq!(a & b, b & a);
            }

            #[test]
            fn prop_symmetric_difference_commutative(a in bit_set_9(), b in bit_set_9()) {
                prop_assert_eq!(a.symmetric_difference(b), b.symmetric_difference(a));
                prop_assert_eq!(a ^ b, b ^ a);
            }

            // Set operations are associative
            #[test]
            fn prop_union_associative(a in bit_set_9(), b in bit_set_9(), c in bit_set_9()) {
                prop_assert_eq!(a.union(b).union(c), a.union(b.union(c)));
            }

            #[test]
            fn prop_intersection_associative(a in bit_set_9(), b in bit_set_9(), c in bit_set_9()) {
                prop_assert_eq!(a.intersection(b).intersection(c), a.intersection(b.intersection(c)));
            }

            // Idempotent operations
            #[test]
            fn prop_union_idempotent(a in bit_set_9()) {
                prop_assert_eq!(a.union(a), a);
            }

            #[test]
            fn prop_intersection_idempotent(a in bit_set_9()) {
                prop_assert_eq!(a.intersection(a), a);
            }

            // Identity elements
            #[test]
            fn prop_union_identity(a in bit_set_9()) {
                prop_assert_eq!(a.union(TestSet::EMPTY), a);
                prop_assert_eq!(TestSet::EMPTY.union(a), a);
            }

            #[test]
            fn prop_intersection_identity(a in bit_set_9()) {
                prop_assert_eq!(a.intersection(TestSet::FULL), a);
                prop_assert_eq!(TestSet::FULL.intersection(a), a);
            }

            // Absorption laws
            #[test]
            fn prop_union_intersection_absorption(a in bit_set_9(), b in bit_set_9()) {
                prop_assert_eq!(a.union(a.intersection(b)), a);
            }

            #[test]
            fn prop_intersection_union_absorption(a in bit_set_9(), b in bit_set_9()) {
                prop_assert_eq!(a.intersection(a.union(b)), a);
            }

            // De Morgan's laws
            #[test]
            fn prop_de_morgan_union(a in bit_set_9(), b in bit_set_9()) {
                prop_assert_eq!(!(a.union(b)), (!a).intersection(!b));
            }

            #[test]
            fn prop_de_morgan_intersection(a in bit_set_9(), b in bit_set_9()) {
                prop_assert_eq!(!(a.intersection(b)), (!a).union(!b));
            }

            // Double negation
            #[test]
            fn prop_double_negation(a in bit_set_9()) {
                prop_assert_eq!(!!a, a);
            }

            // Difference properties
            #[test]
            fn prop_difference_is_disjoint(a in bit_set_9(), b in bit_set_9()) {
                let diff = a.difference(b);
                prop_assert!(diff.is_disjoint(b));
            }

            #[test]
            fn prop_difference_subset(a in bit_set_9(), b in bit_set_9()) {
                let diff = a.difference(b);
                prop_assert!(diff.is_subset(a));
            }

            // Symmetric difference properties
            #[test]
            fn prop_symmetric_difference_involution(a in bit_set_9(), b in bit_set_9()) {
                prop_assert_eq!(a.symmetric_difference(b).symmetric_difference(b), a);
            }

            // Subset/superset properties
            #[test]
            fn prop_subset_reflexive(a in bit_set_9()) {
                prop_assert!(a.is_subset(a));
            }

            #[test]
            fn prop_superset_reflexive(a in bit_set_9()) {
                prop_assert!(a.is_superset(a));
            }

            #[test]
            fn prop_empty_subset(a in bit_set_9()) {
                prop_assert!(TestSet::EMPTY.is_subset(a));
            }

            #[test]
            fn prop_full_superset(a in bit_set_9()) {
                prop_assert!(TestSet::FULL.is_superset(a));
            }

            // Iterator properties
            #[test]
            fn prop_len_equals_count(a in bit_set_9()) {
                prop_assert_eq!(a.len(), a.iter().count());
            }

            #[test]
            fn prop_iter_sorted(a in bit_set_9()) {
                let vec: Vec<u8> = a.iter().collect();
                for i in 1..vec.len() {
                    prop_assert!(vec[i - 1] < vec[i]);
                }
            }

            #[test]
            fn prop_iter_double_ended_consistent(a in bit_set_9()) {
                let forward: Vec<u8> = a.iter().collect();
                let backward: Vec<u8> = a.iter().rev().collect();
                prop_assert_eq!(forward, backward.into_iter().rev().collect::<Vec<_>>());
            }

            // Insert/remove properties
            #[test]
            fn prop_insert_increases_or_maintains_len(mut a in bit_set_9(), n in valid_number()) {
                let old_len = a.len();
                a.insert(n);
                prop_assert!(a.len() >= old_len);
                prop_assert!(a.len() <= old_len + 1);
            }

            #[test]
            fn prop_remove_decreases_or_maintains_len(mut a in bit_set_9(), n in valid_number()) {
                let old_len = a.len();
                a.remove(n);
                prop_assert!(a.len() <= old_len);
                prop_assert!(a.len() >= old_len.saturating_sub(1));
            }

            #[test]
            fn prop_insert_contains(mut a in bit_set_9(), n in valid_number()) {
                a.insert(n);
                prop_assert!(a.contains(n));
            }

            #[test]
            fn prop_remove_not_contains(mut a in bit_set_9(), n in valid_number()) {
                a.remove(n);
                prop_assert!(!a.contains(n));
            }

            // Bit operators match methods
            #[test]
            fn prop_bitor_equals_union(a in bit_set_9(), b in bit_set_9()) {
                prop_assert_eq!(a | b, a.union(b));
            }

            #[test]
            fn prop_bitand_equals_intersection(a in bit_set_9(), b in bit_set_9()) {
                prop_assert_eq!(a & b, a.intersection(b));
            }

            #[test]
            fn prop_bitxor_equals_symmetric_difference(a in bit_set_9(), b in bit_set_9()) {
                prop_assert_eq!(a ^ b, a.symmetric_difference(b));
            }

            // Bounds
            #[test]
            fn prop_len_bounded(a in bit_set_9()) {
                prop_assert!(a.len() <= 9);
            }

            #[test]
            fn prop_first_in_range(a in bit_set_9()) {
                if let Some(n) = a.first() {
                    prop_assert!((0..9).contains(&n));
                    prop_assert!(a.contains(n));
                }
            }

            #[test]
            fn prop_last_in_range(a in bit_set_9()) {
                if let Some(n) = a.last() {
                    prop_assert!((0..9).contains(&n));
                    prop_assert!(a.contains(n));
                }
            }

            #[test]
            fn prop_first_less_equal_last(a in bit_set_9()) {
                if let (Some(first), Some(last)) = (a.first(), a.last()) {
                    prop_assert!(first <= last);
                }
            }

            // Range properties
            #[test]
            fn prop_range_subset(a in bit_set_9(), start in 0u8..=10, end in 0u8..=10) {
                let ranged = a.range(start..end);
                prop_assert!(ranged.is_subset(a));
            }
        }
    }
}
