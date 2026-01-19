//! Generic 81-bit set implementation for 9x9 board positions.
//!
//! Provides [`BitSet81`], a generic bitset for representing sets of up to 81 elements,
//! parameterized by [`Index81Semantics`].
//!
//! [`Index81Semantics`]: crate::index::Index81Semantics

use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    iter::FusedIterator,
    marker::PhantomData,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, RangeBounds},
};

use crate::index::{Index81, Index81Semantics};

/// A generic set of up to 81 elements, represented as a bitset.
///
/// This type uses a 128-bit integer where bits 0-80 represent the 81 possible elements.
/// The specific semantics of the elements are determined by the `S` type parameter,
/// which must implement [`Index81Semantics`].
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
/// See [`DigitPositions`] for a concrete example
/// using [`Position`] as the element type.
///
/// [`DigitPositions`]: crate::candidate_board::DigitPositions
/// [`Position`]: crate::Position
///
/// For information on defining custom semantics, see [`Index81Semantics`].
pub struct BitSet81<S>
where
    S: Index81Semantics,
{
    bits: u128,
    _marker: PhantomData<S>,
}

impl<S> Copy for BitSet81<S> where S: Index81Semantics {}

impl<S> Clone for BitSet81<S>
where
    S: Index81Semantics,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> PartialEq for BitSet81<S>
where
    S: Index81Semantics,
{
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl<S> Eq for BitSet81<S> where S: Index81Semantics {}

impl<S> Hash for BitSet81<S>
where
    S: Index81Semantics,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bits.hash(state);
    }
}

impl<S> Default for BitSet81<S>
where
    S: Index81Semantics,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> BitSet81<S>
where
    S: Index81Semantics,
{
    /// An empty set containing no elements.
    pub const EMPTY: Self = Self::from_bits(0);

    /// A full set containing all 81 possible elements.
    pub const FULL: Self = Self::from_bits((1u128 << 81) - 1);

    const fn from_bits(bits: u128) -> Self {
        assert!(bits < (1u128 << 81));
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

    /// Returns the difference of `self` and `other` (all elements in `self` but not in `other`).
    #[must_use]
    #[inline]
    pub const fn difference(self, other: Self) -> Self {
        Self::from_bits(self.bits & !other.bits)
    }

    /// Returns the symmetric difference of `self` and `other` (all elements in exactly one of the sets).
    #[must_use]
    #[inline]
    pub const fn symmetric_difference(self, other: Self) -> Self {
        Self::from_bits(self.bits ^ other.bits)
    }

    /// Returns the intersection of `self` and `other` (all elements in both sets).
    #[must_use]
    #[inline]
    pub const fn intersection(self, other: Self) -> Self {
        Self::from_bits(self.bits & other.bits)
    }

    /// Returns the union of `self` and `other` (all elements in at least one of the sets).
    #[must_use]
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self::from_bits(self.bits | other.bits)
    }

    /// Removes all elements from the set.
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::EMPTY;
    }

    /// Returns `true` if the set contains the specified value.
    #[must_use]
    #[inline]
    pub fn contains(&self, value: S::Value) -> bool {
        let index = S::to_index(value);
        (self.bits & index.bit()) != 0
    }

    /// Returns `true` if the set has no elements in common with `other`.
    #[must_use]
    #[inline]
    pub const fn is_disjoint(self, other: Self) -> bool {
        (self.bits & other.bits) == 0
    }

    /// Returns `true` if all elements of `self` are contained in `other`.
    #[must_use]
    #[inline]
    pub const fn is_subset(self, other: Self) -> bool {
        (self.bits & other.bits) == self.bits
    }

    /// Returns `true` if all elements of `other` are contained in `self`.
    #[must_use]
    #[inline]
    pub const fn is_superset(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    const fn first_index(self) -> Option<Index81> {
        if self.bits == 0 {
            return None;
        }
        #[expect(clippy::cast_possible_truncation)]
        Some(Index81::new(self.bits.trailing_zeros() as u8))
    }

    const fn last_index(self) -> Option<Index81> {
        if self.bits == 0 {
            return None;
        }
        #[expect(clippy::cast_possible_truncation)]
        Some(Index81::new(127 - self.bits.leading_zeros() as u8))
    }

    /// Returns the smallest element in the set, or `None` if the set is empty.
    #[must_use]
    pub fn first(self) -> Option<S::Value> {
        self.first_index().map(S::from_index)
    }

    /// Returns the largest element in the set, or `None` if the set is empty.
    #[must_use]
    pub fn last(self) -> Option<S::Value> {
        self.last_index().map(S::from_index)
    }

    /// Removes and returns the smallest element in the set, or `None` if the set is empty.
    pub fn pop_first(&mut self) -> Option<S::Value> {
        let index = self.first_index()?;
        self.bits &= !index.bit();
        Some(S::from_index(index))
    }

    /// Removes and returns the largest element in the set, or `None` if the set is empty.
    pub fn pop_last(&mut self) -> Option<S::Value> {
        let index = self.last_index()?;
        self.bits &= !index.bit();
        Some(S::from_index(index))
    }

    /// Adds a value to the set.
    ///
    /// Returns `true` if the value was not present in the set.
    #[inline]
    pub fn insert(&mut self, value: S::Value) -> bool {
        let index = S::to_index(value);
        let bit = index.bit();
        let was_present = (self.bits & bit) != 0;
        self.bits |= bit;
        !was_present
    }

    /// Removes a value from the set.
    ///
    /// Returns `true` if the value was present in the set.
    #[inline]
    pub fn remove(&mut self, value: S::Value) -> bool {
        let index = S::to_index(value);
        let bit = index.bit();
        let was_present = (self.bits & bit) != 0;
        self.bits &= !bit;
        was_present
    }

    /// Returns an iterator over the elements of the set in ascending order.
    #[must_use]
    #[inline]
    pub const fn iter(self) -> BitSet81Iter<S> {
        BitSet81Iter { set: self }
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

impl<S> BitAnd for BitSet81<S>
where
    S: Index81Semantics,
{
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl<S> BitAndAssign for BitSet81<S>
where
    S: Index81Semantics,
{
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.intersection(rhs);
    }
}

impl<S> BitOr for BitSet81<S>
where
    S: Index81Semantics,
{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl<S> BitOrAssign for BitSet81<S>
where
    S: Index81Semantics,
{
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

impl<S> BitXor for BitSet81<S>
where
    S: Index81Semantics,
{
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.symmetric_difference(rhs)
    }
}

impl<S> BitXorAssign for BitSet81<S>
where
    S: Index81Semantics,
{
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.symmetric_difference(rhs);
    }
}

impl<S> Not for BitSet81<S>
where
    S: Index81Semantics,
{
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::from_bits(!self.bits & Self::FULL.bits)
    }
}

impl<S> Debug for BitSet81<S>
where
    S: Index81Semantics,
    S::Value: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<S> IntoIterator for &BitSet81<S>
where
    S: Index81Semantics,
{
    type IntoIter = BitSet81Iter<S>;
    type Item = S::Value;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<S> IntoIterator for BitSet81<S>
where
    S: Index81Semantics,
{
    type IntoIter = BitSet81Iter<S>;
    type Item = S::Value;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator over the elements of a [`BitSet81`].
#[derive(Clone)]
pub struct BitSet81Iter<S>
where
    S: Index81Semantics,
{
    set: BitSet81<S>,
}

impl<S> Debug for BitSet81Iter<S>
where
    S: Index81Semantics,
    S::Value: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BitSet81Iter")
            .field("remaining", &self.set)
            .finish()
    }
}

impl<S> Iterator for BitSet81Iter<S>
where
    S: Index81Semantics,
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

impl<S> DoubleEndedIterator for BitSet81Iter<S>
where
    S: Index81Semantics,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.set.pop_last()
    }
}

impl<S> ExactSizeIterator for BitSet81Iter<S> where S: Index81Semantics {}
impl<S> FusedIterator for BitSet81Iter<S> where S: Index81Semantics {}

impl<S> FromIterator<S::Value> for BitSet81<S>
where
    S: Index81Semantics,
{
    fn from_iter<T: IntoIterator<Item = S::Value>>(iter: T) -> Self {
        let mut set = Self::new();
        for value in iter {
            set.insert(value);
        }
        set
    }
}

impl<S> Extend<S::Value> for BitSet81<S>
where
    S: Index81Semantics,
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

    // Test semantics that maps (x, y) coordinates to indices
    struct PositionSemantics;

    impl Index81Semantics for PositionSemantics {
        type Value = (u8, u8);

        fn to_index(value: (u8, u8)) -> Index81 {
            let (x, y) = value;
            assert!(x < 9 && y < 9);
            Index81::new(y * 9 + x)
        }

        fn from_index(index: Index81) -> (u8, u8) {
            let idx = index.index();
            (idx % 9, idx / 9)
        }
    }

    type TestSet = BitSet81<PositionSemantics>;

    macro_rules! set {
        ($($pos:expr),* $(,)?) => {{
            let mut s = TestSet::new();
            $(s.insert($pos);)*
            s
        }};
    }

    mod bit_index {
        use super::*;

        #[test]
        fn test_all() {
            let indices: Vec<_> = Index81::all().collect();
            assert_eq!(indices.len(), 81);
            for (i, index) in (0..).zip(indices) {
                assert_eq!(index.index(), i);
            }
        }

        #[test]
        fn test_all_creates_valid_indices() {
            for index in Index81::all() {
                assert!(index.index() < 81);
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
            let set = TestSet::EMPTY;
            assert!(set.is_empty());
        }

        #[test]
        fn test_full_constant() {
            let set = TestSet::FULL;
            assert_eq!(set.len(), 81);
            for y in 0..9 {
                for x in 0..9 {
                    assert!(set.contains((x, y)));
                }
            }
        }

        #[test]
        fn test_from_iter() {
            let positions = vec![(0, 0), (1, 1), (2, 2)];
            let set: TestSet = positions.into_iter().collect();
            assert_eq!(set.len(), 3);
            assert!(set.contains((0, 0)));
            assert!(set.contains((1, 1)));
            assert!(set.contains((2, 2)));
        }

        #[test]
        fn test_default() {
            let set = TestSet::default();
            assert!(set.is_empty());
        }
    }

    mod basic_operations {
        use super::*;

        #[test]
        fn test_insert() {
            let mut set = TestSet::new();
            assert!(set.insert((0, 0)));
            assert!(set.contains((0, 0)));
            assert!(!set.insert((0, 0)));
            assert_eq!(set.len(), 1);
        }

        #[test]
        fn test_remove() {
            let mut set = set![(0, 0), (1, 1)];
            assert!(set.remove((0, 0)));
            assert!(!set.contains((0, 0)));
            assert!(!set.remove((0, 0)));
            assert_eq!(set.len(), 1);
        }

        #[test]
        fn test_contains() {
            let set = set![(0, 0), (1, 1)];
            assert!(set.contains((0, 0)));
            assert!(set.contains((1, 1)));
            assert!(!set.contains((2, 2)));
        }

        #[test]
        fn test_clear() {
            let mut set = set![(0, 0), (1, 1)];
            set.clear();
            assert!(set.is_empty());
            assert_eq!(set.len(), 0);
        }

        #[test]
        fn test_all_positions() {
            let mut set = TestSet::new();
            for y in 0..9 {
                for x in 0..9 {
                    assert!(set.insert((x, y)));
                }
            }
            assert_eq!(set.len(), 81);
        }
    }

    mod set_operations {
        use super::*;

        #[test]
        fn test_union() {
            let set1 = set![(0, 0), (1, 1)];
            let set2 = set![(1, 1), (2, 2)];
            let union = set1.union(set2);
            assert_eq!(union.len(), 3);
            assert!(union.contains((0, 0)));
            assert!(union.contains((1, 1)));
            assert!(union.contains((2, 2)));
        }

        #[test]
        fn test_intersection() {
            let set1 = set![(0, 0), (1, 1)];
            let set2 = set![(1, 1), (2, 2)];
            let intersection = set1.intersection(set2);
            assert_eq!(intersection.len(), 1);
            assert!(intersection.contains((1, 1)));
        }

        #[test]
        fn test_difference() {
            let set1 = set![(0, 0), (1, 1), (2, 2)];
            let set2 = set![(1, 1)];
            let difference = set1.difference(set2);
            assert_eq!(difference.len(), 2);
            assert!(difference.contains((0, 0)));
            assert!(difference.contains((2, 2)));
        }

        #[test]
        fn test_symmetric_difference() {
            let set1 = set![(0, 0), (1, 1)];
            let set2 = set![(1, 1), (2, 2)];
            let sym_diff = set1.symmetric_difference(set2);
            assert_eq!(sym_diff.len(), 2);
            assert!(sym_diff.contains((0, 0)));
            assert!(sym_diff.contains((2, 2)));
        }

        #[test]
        fn test_not() {
            let set = set![(0, 0)];
            let complement = !set;
            assert_eq!(complement.len(), 80);
            assert!(!complement.contains((0, 0)));
            assert!(complement.contains((1, 1)));
        }

        #[test]
        fn test_assign_operators() {
            let mut set1 = set![(0, 0), (1, 1)];
            let set2 = set![(1, 1), (2, 2)];

            set1 |= set2;
            assert_eq!(set1.len(), 3);

            let mut set3 = set![(0, 0), (1, 1)];
            set3 &= set2;
            assert_eq!(set3.len(), 1);

            let mut set4 = set![(0, 0), (1, 1)];
            set4 ^= set2;
            assert_eq!(set4.len(), 2);
        }
    }

    mod relations {
        use super::*;

        #[test]
        fn test_is_subset() {
            let set1 = set![(0, 0)];
            let set2 = set![(0, 0), (1, 1)];
            assert!(set1.is_subset(set2));
            assert!(!set2.is_subset(set1));
            assert!(set1.is_subset(set1));
        }

        #[test]
        fn test_is_superset() {
            let set1 = set![(0, 0), (1, 1)];
            let set2 = set![(0, 0)];
            assert!(set1.is_superset(set2));
            assert!(!set2.is_superset(set1));
            assert!(set1.is_superset(set1));
        }

        #[test]
        fn test_is_disjoint() {
            let set1 = set![(0, 0)];
            let set2 = set![(1, 1)];
            let set3 = set![(0, 0), (1, 1)];
            assert!(set1.is_disjoint(set2));
            assert!(!set1.is_disjoint(set3));
        }
    }

    mod access {
        use super::*;

        #[test]
        fn test_first() {
            let set = set![(2, 2), (1, 1), (3, 3)];
            assert_eq!(set.first(), Some((1, 1)));
        }

        #[test]
        fn test_last() {
            let set = set![(2, 2), (1, 1), (3, 3)];
            assert_eq!(set.last(), Some((3, 3)));
        }

        #[test]
        fn test_pop_first() {
            let mut set = set![(1, 1), (2, 2)];
            assert_eq!(set.pop_first(), Some((1, 1)));
            assert_eq!(set.len(), 1);
            assert_eq!(set.pop_first(), Some((2, 2)));
            assert_eq!(set.pop_first(), None);
        }

        #[test]
        fn test_pop_last() {
            let mut set = set![(1, 1), (2, 2)];
            assert_eq!(set.pop_last(), Some((2, 2)));
            assert_eq!(set.len(), 1);
            assert_eq!(set.pop_last(), Some((1, 1)));
            assert_eq!(set.pop_last(), None);
        }
    }

    mod iteration {
        use super::*;

        #[test]
        fn test_iter_ascending() {
            let set = set![(2, 2), (1, 1), (0, 0)];
            let collected: Vec<_> = set.iter().collect();
            assert_eq!(collected, vec![(0, 0), (1, 1), (2, 2)]);
        }

        #[test]
        fn test_iter_double_ended() {
            let set = set![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)];
            let mut iter = set.iter();
            assert_eq!(iter.next(), Some((0, 0)));
            assert_eq!(iter.next_back(), Some((4, 4)));
            assert_eq!(iter.next(), Some((1, 1)));
            assert_eq!(iter.next_back(), Some((3, 3)));
            assert_eq!(iter.next(), Some((2, 2)));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
        }

        #[test]
        fn test_iter_size_hint() {
            let set = set![(0, 0), (1, 1), (2, 2)];
            let iter = set.iter();
            assert_eq!(iter.size_hint(), (3, Some(3)));
        }

        #[test]
        fn test_into_iter() {
            let set = set![(0, 0), (1, 1)];
            let collected: Vec<_> = set.into_iter().collect();
            assert_eq!(collected.len(), 2);
        }

        #[test]
        fn test_iter_ref() {
            let set = set![(0, 0), (1, 1)];
            let collected: Vec<_> = (&set).into_iter().collect();
            assert_eq!(collected.len(), 2);
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_boundary_values() {
            let mut set = TestSet::new();
            set.insert((0, 0));
            set.insert((8, 8));
            assert!(set.contains((0, 0)));
            assert!(set.contains((8, 8)));
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_all_operations_on_empty() {
            let empty = TestSet::new();
            assert!(empty.first().is_none());
            assert!(empty.last().is_none());
            assert_eq!(empty.len(), 0);
            assert!(empty.is_empty());
            assert_eq!((!empty).len(), 81);
        }

        #[test]
        fn test_all_operations_on_full() {
            let full = TestSet::FULL;
            assert_eq!(full.first(), Some((0, 0)));
            assert_eq!(full.last(), Some((8, 8)));
            assert_eq!(full.len(), 81);
            assert!(!full.is_empty());
            assert_eq!((!full).len(), 0);
        }

        #[test]
        fn test_single_element_sets() {
            for y in 0..9 {
                for x in 0..9 {
                    let set = set![(x, y)];
                    assert_eq!(set.len(), 1);
                    assert_eq!(set.first(), Some((x, y)));
                    assert_eq!(set.last(), Some((x, y)));
                }
            }
        }
    }

    mod invariants {
        use super::*;

        #[test]
        fn test_len_equals_iter_count() {
            let set = set![(0, 0), (1, 1), (2, 2), (3, 3)];
            assert_eq!(set.len(), set.iter().count());
        }

        #[test]
        fn test_insert_remove_roundtrip() {
            let mut set = TestSet::new();
            let pos = (4, 4);
            assert!(set.insert(pos));
            assert!(set.contains(pos));
            assert!(set.remove(pos));
            assert!(!set.contains(pos));
        }

        #[test]
        fn test_union_size_bound() {
            let set1 = set![(0, 0), (1, 1)];
            let set2 = set![(2, 2), (3, 3)];
            let union = set1.union(set2);
            assert!(union.len() <= set1.len() + set2.len());
        }

        #[test]
        fn test_hash() {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let set1 = set![(0, 0), (1, 1)];
            let set2 = set![(0, 0), (1, 1)];
            let set3 = set![(1, 1), (2, 2)];

            let hash1 = {
                let mut hasher = DefaultHasher::new();
                set1.hash(&mut hasher);
                hasher.finish()
            };

            let hash2 = {
                let mut hasher = DefaultHasher::new();
                set2.hash(&mut hasher);
                hasher.finish()
            };

            let hash3 = {
                let mut hasher = DefaultHasher::new();
                set3.hash(&mut hasher);
                hasher.finish()
            };

            assert_eq!(hash1, hash2);
            assert_ne!(hash1, hash3);
        }

        #[test]
        fn test_extend() {
            let mut set = set![(0, 0)];
            set.extend(vec![(1, 1), (2, 2)]);
            assert_eq!(set.len(), 3);
        }

        #[test]
        fn test_extend_overlapping() {
            let mut set = set![(0, 0), (1, 1)];
            set.extend(vec![(1, 1), (2, 2)]);
            assert_eq!(set.len(), 3);
        }

        #[test]
        fn test_extend_empty() {
            let mut set = set![(0, 0)];
            set.extend(Vec::<(u8, u8)>::new());
            assert_eq!(set.len(), 1);
        }
    }
}
