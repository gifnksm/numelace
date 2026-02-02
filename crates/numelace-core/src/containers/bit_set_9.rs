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
/// The `S` parameter provides **compile-time type safety** through the
/// [Semantics Pattern](crate#semantics-pattern-type-safe-indexing), preventing
/// accidental use of incorrect element types in set operations.
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
/// See [`DigitSet`] for a concrete example of `BitSet9` specialized with [`DigitSemantics`].
///
/// [`DigitSet`]: crate::DigitSet
/// [`DigitSemantics`]: crate::index::DigitSemantics
///
/// For defining custom semantics, see [`Index9Semantics`].
///
/// See the [crate-level documentation](crate#semantics-pattern-type-safe-indexing) for details.
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

    /// Creates a set from the given raw bits.
    ///
    /// # Panics
    ///
    /// Panics if `bits` contains any bits outside the lower 9 bits.
    #[must_use]
    pub const fn from_bits(bits: u16) -> Self {
        assert!(bits <= 0x1ff);
        Self {
            bits,
            _marker: PhantomData,
        }
    }

    /// Attempts to create a set from the given raw bits.
    ///
    /// Returns `None` if `bits` contains any bits outside the lower 9 bits.
    #[must_use]
    pub const fn try_from_bits(bits: u16) -> Option<Self> {
        if bits > 0x1ff {
            return None;
        }
        Some(Self::from_bits(bits))
    }

    /// Returns the raw bits representing the set.
    #[must_use]
    pub fn bits(self) -> u16 {
        self.bits
    }

    /// Creates a new empty set.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self::EMPTY
    }

    /// Creates a set containing exactly one element.
    ///
    /// # Panics
    ///
    /// Panics if the value cannot be converted to a valid bit index by the semantics.
    #[must_use]
    #[inline]
    pub fn from_elem(value: S::Value) -> Self {
        let index = S::to_index(value);
        Self::from_bits(index.bit())
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

    fn nth_index(self, n: usize) -> Option<Index9> {
        let mut count = 0;
        let start = self.first_index()?.index();
        for i in start..9 {
            let idx = Index9::new(i);
            if self.bits & idx.bit() != 0 {
                if count == n {
                    return Some(idx);
                }
                count += 1;
            }
        }
        None
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

    /// Returns the n-th smallest element in the set (0-indexed), if any.
    ///
    /// This operation has O(n) time complexity.
    #[must_use]
    #[inline]
    pub fn nth(self, n: usize) -> Option<S::Value> {
        self.nth_index(n).map(S::from_index)
    }

    /// Returns the only element in the set, or `None` if the set has zero or multiple elements.
    #[must_use]
    #[inline]
    pub fn as_single(self) -> Option<S::Value> {
        if self.len() == 1 { self.first() } else { None }
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

    /// Removes and returns the n-th smallest element in the set (0-indexed), if any.
    ///
    /// This operation has O(n) time complexity.
    #[inline]
    pub fn pop_nth(&mut self, n: usize) -> Option<S::Value> {
        let i = self.nth_index(n)?;
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

    /// Sets the presence of a value in the set.
    ///
    /// If `present` is `true`, the value is added to the set;
    /// if `false`, the value is removed from the set.
    ///
    /// Returns `true` if the set was modified.
    #[inline]
    pub fn set(&mut self, value: S::Value, present: bool) -> bool {
        if present {
            self.insert(value)
        } else {
            self.remove(value)
        }
    }

    /// Toggles the presence of a value in the set.
    ///
    /// If the value is present, it is removed; otherwise, it is added.
    #[inline]
    pub fn toggle(&mut self, value: S::Value) {
        let i = S::to_index(value);
        if (self.bits & i.bit()) != 0 {
            self.bits &= !i.bit();
        } else {
            self.bits |= i.bit();
        }
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

    #[test]
    fn test_construction() {
        let set = TestSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert_eq!(TestSet::EMPTY, TestSet::new());
        assert_eq!(TestSet::default(), TestSet::EMPTY);

        let full = TestSet::FULL;
        assert_eq!(full.len(), 9);
        for n in 0..9 {
            assert!(full.contains(n));
        }

        let set = set![0, 2, 4, 6, 8];
        assert_eq!(set.len(), 5);
        assert!(set.contains(0));
        assert!(!set.contains(1));
    }

    #[test]
    fn test_try_from_bits() {
        let set = TestSet::try_from_bits(0x1ff).expect("valid bits");
        assert_eq!(set.bits(), 0x1ff);

        let set = TestSet::try_from_bits(0).expect("valid bits");
        assert!(set.is_empty());

        assert!(TestSet::try_from_bits(0x200).is_none());
    }

    #[test]
    fn test_from_elem() {
        let set = TestSet::from_elem(4);
        assert_eq!(set.len(), 1);
        assert!(set.contains(4));
    }

    #[test]
    fn test_insert_remove_contains() {
        // Insert is idempotent - duplicate insertions are no-ops
        let mut set = TestSet::new();
        assert!(set.insert(0));
        assert!(!set.insert(0));
        assert_eq!(set.len(), 1);
        assert!(set.contains(0));

        // Remove is idempotent - removing non-existent element is no-op
        let mut set = set![0, 1, 2];
        assert!(set.remove(1));
        assert!(!set.remove(1));
        assert_eq!(set.len(), 2);
        assert!(!set.contains(1));

        set.clear();
        assert!(set.is_empty());
    }

    #[test]
    fn test_toggle() {
        let mut set = TestSet::new();
        set.toggle(3);
        assert!(set.contains(3));
        set.toggle(3);
        assert!(!set.contains(3));

        let mut set = set![0, 2];
        set.toggle(2);
        set.toggle(4);
        assert_eq!(set, set![0, 4]);
    }

    // Values outside 0-8 range are invalid
    #[test]
    #[should_panic(expected = "Test value must be 0-8")]
    fn test_insert_out_of_bounds_9() {
        TestSet::new().insert(9);
    }

    #[test]
    #[should_panic(expected = "Test value must be 0-8")]
    fn test_insert_out_of_bounds_10() {
        TestSet::new().insert(10);
    }

    #[test]
    fn test_union() {
        // Empty set is identity element for union
        let cases = [
            (set![0, 1], set![1, 2], set![0, 1, 2]),
            (set![0], set![8], set![0, 8]),
            (set![], set![0, 1], set![0, 1]),
            (set![0, 1, 2], set![3, 4, 5], set![0, 1, 2, 3, 4, 5]),
        ];
        for (a, b, expected) in cases {
            assert_eq!(a.union(b), expected);
            assert_eq!(b.union(a), expected); // Commutativity
            assert_eq!(a | b, expected);
        }

        // Associativity
        let (a, b, c) = (set![0, 1], set![2, 3], set![4, 5]);
        assert_eq!((a | b) | c, a | (b | c));
    }

    #[test]
    fn test_intersection() {
        // Empty set is absorbing element for intersection
        let cases = [
            (set![0, 1, 2], set![1, 2, 3], set![1, 2]),
            (set![0, 1], set![2, 3], set![]),
            (set![0, 1, 2], set![0, 1, 2], set![0, 1, 2]),
            (set![], set![0, 1], set![]),
        ];
        for (a, b, expected) in cases {
            assert_eq!(a.intersection(b), expected);
            assert_eq!(b.intersection(a), expected); // Commutativity
            assert_eq!(a & b, expected);
        }

        // Associativity
        let (a, b, c) = (set![0, 1, 2, 3], set![1, 2, 3, 4], set![2, 3, 4, 5]);
        assert_eq!((a & b) & c, a & (b & c));
    }

    #[test]
    fn test_difference() {
        let cases = [
            (set![0, 1, 2], set![1, 2, 3], set![0]),
            (set![0, 1, 2], set![3, 4, 5], set![0, 1, 2]),
            (set![0, 1, 2], set![0, 1, 2], set![]),
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
        ];
        for (a, b, expected) in cases {
            assert_eq!(a.symmetric_difference(b), expected);
            assert_eq!(b.symmetric_difference(a), expected); // Commutativity
            assert_eq!(a ^ b, expected);
        }
    }

    #[test]
    fn test_not() {
        // Complement: !EMPTY = FULL, !FULL = EMPTY
        let set = set![0, 2, 4, 6, 8];
        assert_eq!(!set, set![1, 3, 5, 7]);
        assert_eq!(!TestSet::EMPTY, TestSet::FULL);
        assert_eq!(!TestSet::FULL, TestSet::EMPTY);

        // Double negation
        assert_eq!(!!set, set);
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

    #[test]
    fn test_is_subset() {
        // Subset is reflexive; empty set is subset of all sets
        let cases = [
            (set![0, 1], set![0, 1, 2], true),
            (set![0, 1, 2], set![0, 1], false),
            (set![0, 1], set![0, 1], true),
            (set![], set![0, 1], true),
        ];
        for (a, b, expected) in cases {
            assert_eq!(a.is_subset(b), expected);
        }
    }

    #[test]
    fn test_is_superset() {
        // Superset is reflexive; all sets are supersets of empty set
        let cases = [
            (set![0, 1, 2], set![0, 1], true),
            (set![0, 1], set![0, 1, 2], false),
            (set![0, 1], set![0, 1], true),
        ];
        for (a, b, expected) in cases {
            assert_eq!(a.is_superset(b), expected);
        }
    }

    #[test]
    fn test_is_disjoint() {
        // Empty set is disjoint with all sets
        let cases = [
            (set![0, 1], set![2, 3], true),
            (set![0, 1, 2], set![2, 3, 4], false),
            (set![], set![0, 1], true),
        ];
        for (a, b, expected) in cases {
            assert_eq!(a.is_disjoint(b), expected);
        }
    }

    #[test]
    fn test_first_last() {
        assert_eq!(set![2, 6, 0].first(), Some(0));
        assert_eq!(set![2, 6, 0].last(), Some(6));
        assert_eq!(set![8].first(), Some(8));
        assert_eq!(set![].first(), None);
        assert_eq!(set![].last(), None);
    }

    #[test]
    fn test_as_single() {
        assert_eq!(set![].as_single(), None);
        assert_eq!(set![3].as_single(), Some(3));
        assert_eq!(set![2, 6, 0].as_single(), None);
    }

    #[test]
    fn test_pop_first_last() {
        let mut set = set![2, 6, 0];
        assert_eq!(set.pop_first(), Some(0));
        assert_eq!(set.pop_first(), Some(2));
        assert_eq!(set.pop_first(), Some(6));
        assert_eq!(set.pop_first(), None);

        let mut set = set![2, 6, 0];
        assert_eq!(set.pop_last(), Some(6));
        assert_eq!(set.pop_last(), Some(2));
        assert_eq!(set.pop_last(), Some(0));
        assert_eq!(set.pop_last(), None);
    }

    #[test]
    fn test_nth() {
        let set = set![0, 2, 4, 6];
        assert_eq!(set.nth(0), Some(0));
        assert_eq!(set.nth(1), Some(2));
        assert_eq!(set.nth(3), Some(6));
        assert_eq!(set.nth(4), None);

        let mut set = set![1, 3, 5, 7];
        assert_eq!(set.pop_nth(2), Some(5));
        assert_eq!(set, set![1, 3, 7]);
    }

    #[test]
    fn test_range() {
        let set = set![0, 2, 4, 6, 8];
        assert_eq!(set.range(2..=6), set![2, 4, 6]);
        assert_eq!(set.range(2..6), set![2, 4]);
        assert_eq!(set.range(..4), set![0, 2]);
        assert_eq!(set.range(6..), set![6, 8]);
    }

    #[test]
    fn test_iter() {
        // Iterator maintains sorted order invariant
        let set = set![4, 0, 8, 2];
        let vec: Vec<u8> = set.iter().collect();
        assert_eq!(vec, vec![0, 2, 4, 8]);

        let vec: Vec<u8> = set.into_iter().collect();
        assert_eq!(vec, vec![0, 2, 4, 8]);

        let set = set![0, 2, 4];
        let vec: Vec<u8> = (&set).into_iter().collect();
        assert_eq!(vec, vec![0, 2, 4]);
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
        // ExactSizeIterator provides exact size
        let set = set![0, 2, 4];
        let iter = set.iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.len(), 3);
    }

    #[test]
    fn test_empty_and_full() {
        // Complement properties: !EMPTY = FULL, !FULL = EMPTY
        let empty = TestSet::EMPTY;
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.first(), None);
        assert_eq!(empty.union(empty), empty);
        assert_eq!(!empty, TestSet::FULL);

        let full = TestSet::FULL;
        assert_eq!(full.len(), 9);
        assert_eq!(full.first(), Some(0));
        assert_eq!(full.last(), Some(8));
        assert_eq!(full.union(full), full);
        assert_eq!(!full, TestSet::EMPTY);
    }

    #[test]
    fn test_boundary_values() {
        // Minimum (0) and maximum (8) values are valid
        let mut set = TestSet::new();
        set.insert(0);
        set.insert(8);
        assert_eq!(set.len(), 2);
        assert!(set.contains(0));
        assert!(set.contains(8));
    }

    #[test]
    fn test_hash() {
        // Equal sets produce equal hashes
        use std::collections::HashSet;

        let set1 = set![0, 1, 2];
        let set2 = set![0, 1, 2];
        let set3 = set![0, 1, 3];

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

        let mut set = set![0, 1, 2];
        set.extend([2, 3, 4]);
        assert_eq!(set, set![0, 1, 2, 3, 4]);
    }
}
