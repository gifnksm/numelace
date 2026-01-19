//! A set of numbers from 1 to 9, optimized for sudoku cells.

use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, RangeBounds},
};

const fn is_valid(n: u8) -> bool {
    1 <= n && n <= 9
}

const fn bit(n: u8) -> u16 {
    assert!(is_valid(n));
    1 << (n - 1)
}

/// A set of numbers from 1 to 9, represented as a bitset.
///
/// This type is specifically designed for sudoku solvers, where each cell
/// can contain numbers from 1 to 9. The implementation uses a 16-bit integer
/// where bits 0-8 represent numbers 1-9 respectively.
///
/// # Examples
///
/// ```
/// # use sudoku_core::number_set::NumberSet;
/// let mut set = NumberSet::new();
/// set.insert(1);
/// set.insert(5);
/// set.insert(9);
///
/// assert_eq!(set.len(), 3);
/// assert!(set.contains(1));
/// assert!(!set.contains(2));
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NumberSet {
    bits: u16,
}

impl Default for NumberSet {
    fn default() -> Self {
        Self::new()
    }
}

impl NumberSet {
    /// An empty set containing no numbers.
    pub const EMPTY: Self = Self { bits: 0 };

    /// A full set containing all numbers from 1 to 9.
    pub const FULL: Self = Self { bits: 0x1ff };

    /// Creates a new empty `NumberSet`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let set = NumberSet::new();
    /// assert!(set.is_empty());
    /// ```
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self::EMPTY
    }

    /// Returns a new set containing only the elements in this set that fall within the given range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let set = NumberSet::from_iter([1, 3, 5, 7, 9]);
    /// let subset = set.range(3..=7);
    ///
    /// assert!(!subset.contains(1));
    /// assert!(subset.contains(3));
    /// assert!(subset.contains(5));
    /// assert!(subset.contains(7));
    /// assert!(!subset.contains(9));
    /// ```
    #[must_use]
    pub fn range<R>(self, range: R) -> Self
    where
        R: RangeBounds<u8>,
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let a = NumberSet::from_iter([1, 2, 3]);
    /// let b = NumberSet::from_iter([2, 3, 4]);
    /// let diff = a.difference(b);
    ///
    /// assert!(diff.contains(1));
    /// assert!(!diff.contains(2));
    /// assert!(!diff.contains(3));
    /// assert!(!diff.contains(4));
    /// ```
    #[must_use]
    #[inline]
    pub const fn difference(self, other: Self) -> Self {
        Self {
            bits: self.bits & !other.bits,
        }
    }

    /// Returns the symmetric difference of two sets.
    ///
    /// Returns a new set containing elements in either `self` or `other`, but not in both.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let a = NumberSet::from_iter([1, 2, 3]);
    /// let b = NumberSet::from_iter([2, 3, 4]);
    /// let sym_diff = a.symmetric_difference(b);
    ///
    /// assert!(sym_diff.contains(1));
    /// assert!(!sym_diff.contains(2));
    /// assert!(!sym_diff.contains(3));
    /// assert!(sym_diff.contains(4));
    /// ```
    #[must_use]
    #[inline]
    pub const fn symmetric_difference(self, other: Self) -> Self {
        Self {
            bits: self.bits ^ other.bits,
        }
    }

    /// Returns the intersection of two sets.
    ///
    /// Returns a new set containing elements in both `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let a = NumberSet::from_iter([1, 2, 3]);
    /// let b = NumberSet::from_iter([2, 3, 4]);
    /// let inter = a.intersection(b);
    ///
    /// assert!(!inter.contains(1));
    /// assert!(inter.contains(2));
    /// assert!(inter.contains(3));
    /// assert!(!inter.contains(4));
    /// ```
    #[must_use]
    #[inline]
    pub const fn intersection(self, other: Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }

    /// Returns the union of two sets.
    ///
    /// Returns a new set containing elements in either `self` or `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let a = NumberSet::from_iter([1, 2, 3]);
    /// let b = NumberSet::from_iter([3, 4, 5]);
    /// let union = a.union(b);
    ///
    /// assert_eq!(union.len(), 5);
    /// assert!(union.contains(1));
    /// assert!(union.contains(5));
    /// ```
    #[must_use]
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    /// Clears the set, removing all elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let mut set = NumberSet::from_iter([1, 2, 3]);
    /// set.clear();
    /// assert!(set.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::EMPTY;
    }

    /// Returns `true` if the set contains the specified number.
    ///
    /// # Panics
    ///
    /// Panics if `n` is not in the range 1-9.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let set = NumberSet::from_iter([1, 3, 5]);
    /// assert!(set.contains(1));
    /// assert!(!set.contains(2));
    /// ```
    #[must_use]
    #[inline]
    pub const fn contains(self, n: u8) -> bool {
        assert!(is_valid(n));
        (self.bits & bit(n)) != 0
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let a = NumberSet::from_iter([1, 2, 3]);
    /// let b = NumberSet::from_iter([4, 5, 6]);
    /// assert!(a.is_disjoint(b));
    ///
    /// let c = NumberSet::from_iter([3, 4, 5]);
    /// assert!(!a.is_disjoint(c));
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_disjoint(self, other: Self) -> bool {
        self.intersection(other).is_empty()
    }

    /// Returns `true` if the set is a subset of another.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let a = NumberSet::from_iter([1, 2]);
    /// let b = NumberSet::from_iter([1, 2, 3]);
    /// assert!(a.is_subset(b));
    /// assert!(!b.is_subset(a));
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_subset(self, other: Self) -> bool {
        self.union(other).bits == other.bits
    }

    /// Returns `true` if the set is a superset of another.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let a = NumberSet::from_iter([1, 2, 3]);
    /// let b = NumberSet::from_iter([1, 2]);
    /// assert!(a.is_superset(b));
    /// assert!(!b.is_superset(a));
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_superset(self, other: Self) -> bool {
        self.union(other).bits == self.bits
    }

    /// Returns the smallest element in the set, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let set = NumberSet::from_iter([3, 7, 1]);
    /// assert_eq!(set.first(), Some(1));
    ///
    /// let empty = NumberSet::new();
    /// assert_eq!(empty.first(), None);
    /// ```
    #[must_use]
    #[inline]
    #[expect(clippy::cast_possible_truncation)]
    pub const fn first(self) -> Option<u8> {
        match self.bits.trailing_zeros() {
            16 => None,
            n => Some(n as u8 + 1),
        }
    }

    /// Returns the largest element in the set, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let set = NumberSet::from_iter([3, 7, 1]);
    /// assert_eq!(set.last(), Some(7));
    ///
    /// let empty = NumberSet::new();
    /// assert_eq!(empty.last(), None);
    /// ```
    #[must_use]
    #[inline]
    #[expect(clippy::cast_possible_truncation)]
    pub const fn last(self) -> Option<u8> {
        match self.bits.leading_zeros() {
            16 => None,
            n => Some(16 - n as u8),
        }
    }

    /// Removes and returns the smallest element in the set, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let mut set = NumberSet::from_iter([3, 7, 1]);
    /// assert_eq!(set.pop_first(), Some(1));
    /// assert_eq!(set.pop_first(), Some(3));
    /// assert_eq!(set.len(), 1);
    /// ```
    #[inline]
    pub const fn pop_first(&mut self) -> Option<u8> {
        let Some(n) = self.first() else {
            return None;
        };
        self.bits &= !bit(n);
        Some(n)
    }

    /// Removes and returns the largest element in the set, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let mut set = NumberSet::from_iter([3, 7, 1]);
    /// assert_eq!(set.pop_last(), Some(7));
    /// assert_eq!(set.pop_last(), Some(3));
    /// assert_eq!(set.len(), 1);
    /// ```
    #[inline]
    pub const fn pop_last(&mut self) -> Option<u8> {
        let Some(n) = self.last() else {
            return None;
        };
        self.bits &= !bit(n);
        Some(n)
    }

    /// Adds a number to the set.
    ///
    /// Returns `true` if the number was not already in the set.
    ///
    /// # Panics
    ///
    /// Panics if `n` is not in the range 1-9.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let mut set = NumberSet::new();
    /// assert!(set.insert(1));
    /// assert!(!set.insert(1));
    /// assert_eq!(set.len(), 1);
    /// ```
    #[inline]
    pub const fn insert(&mut self, n: u8) -> bool {
        assert!(is_valid(n));
        let old = self.bits;
        self.bits |= bit(n);
        old != self.bits
    }

    /// Removes a number from the set.
    ///
    /// Returns `true` if the number was present in the set.
    ///
    /// # Panics
    ///
    /// Panics if `n` is not in the range 1-9.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let mut set = NumberSet::from_iter([1, 2, 3]);
    /// assert!(set.remove(2));
    /// assert!(!set.remove(2));
    /// assert_eq!(set.len(), 2);
    /// ```
    #[inline]
    pub const fn remove(&mut self, n: u8) -> bool {
        assert!(is_valid(n));
        let old = self.bits;
        self.bits &= !bit(n);
        old != self.bits
    }

    /// Returns an iterator over the elements of the set in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let set = NumberSet::from_iter([3, 1, 7]);
    /// let vec: Vec<u8> = set.iter().collect();
    /// assert_eq!(vec, vec![1, 3, 7]);
    /// ```
    #[must_use]
    #[inline]
    pub const fn iter(self) -> NumberSetIter {
        NumberSetIter {
            set: Self { bits: self.bits },
        }
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let set = NumberSet::from_iter([1, 3, 5]);
    /// assert_eq!(set.len(), 3);
    /// ```
    #[must_use]
    #[inline]
    pub const fn len(self) -> usize {
        self.bits.count_ones() as usize
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::number_set::NumberSet;
    /// let set = NumberSet::new();
    /// assert!(set.is_empty());
    ///
    /// let set = NumberSet::from_iter([1]);
    /// assert!(!set.is_empty());
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }
}

impl BitAnd for NumberSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits & rhs.bits,
        }
    }
}

impl BitAndAssign for NumberSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bits &= rhs.bits;
    }
}

impl BitOr for NumberSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

impl BitOrAssign for NumberSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

impl BitXor for NumberSet {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits ^ rhs.bits,
        }
    }
}

impl BitXorAssign for NumberSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bits ^= rhs.bits;
    }
}

impl Not for NumberSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            bits: !self.bits & Self::FULL.bits,
        }
    }
}

impl Debug for NumberSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl IntoIterator for &NumberSet {
    type IntoIter = NumberSetIter;
    type Item = u8;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for NumberSet {
    type IntoIter = NumberSetIter;
    type Item = u8;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the elements of a `NumberSet`.
///
/// This iterator yields numbers in ascending order and supports
/// double-ended iteration.
#[derive(Debug)]
pub struct NumberSetIter {
    set: NumberSet,
}

impl Iterator for NumberSetIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.set.pop_first()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.set.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for NumberSetIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.set.pop_last()
    }
}

impl ExactSizeIterator for NumberSetIter {}
impl FusedIterator for NumberSetIter {}

impl FromIterator<u8> for NumberSet {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut set = NumberSet::EMPTY;
        for n in iter {
            set.insert(n);
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper macro to create sets concisely
    macro_rules! set {
        [$($n:expr),* $(,)?] => {
            NumberSet::from_iter([$($n),*])
        };
    }

    mod construction {
        use super::*;

        #[test]
        fn test_new_is_empty() {
            let set = NumberSet::new();
            assert!(set.is_empty());
            assert_eq!(set.len(), 0);
        }

        #[test]
        fn test_empty_constant() {
            assert_eq!(NumberSet::EMPTY, NumberSet::new());
            assert!(NumberSet::EMPTY.is_empty());
        }

        #[test]
        fn test_full_constant() {
            let full = NumberSet::FULL;
            assert_eq!(full.len(), 9);
            for n in 1..=9 {
                assert!(full.contains(n));
            }
        }

        #[test]
        fn test_from_iter() {
            let set = set![1, 3, 5, 7, 9];
            assert_eq!(set.len(), 5);
            assert!(set.contains(1));
            assert!(!set.contains(2));
            assert!(set.contains(3));
        }

        #[test]
        fn test_default() {
            let set = NumberSet::default();
            assert_eq!(set, NumberSet::EMPTY);
        }
    }

    mod basic_operations {
        use super::*;

        #[test]
        fn test_insert() {
            let mut set = NumberSet::new();
            assert!(set.insert(1));
            assert!(!set.insert(1));
            assert_eq!(set.len(), 1);
            assert!(set.contains(1));
        }

        #[test]
        fn test_remove() {
            let mut set = set![1, 2, 3];
            assert!(set.remove(2));
            assert!(!set.remove(2));
            assert_eq!(set.len(), 2);
            assert!(!set.contains(2));
        }

        #[test]
        fn test_contains() {
            let set = set![1, 5, 9];
            assert!(set.contains(1));
            assert!(!set.contains(2));
            assert!(set.contains(5));
            assert!(set.contains(9));
        }

        #[test]
        fn test_clear() {
            let mut set = set![1, 2, 3, 4, 5];
            set.clear();
            assert!(set.is_empty());
            assert_eq!(set.len(), 0);
        }

        #[test]
        #[should_panic(expected = "assertion failed")]
        fn test_insert_zero_panics() {
            let mut set = NumberSet::new();
            set.insert(0);
        }

        #[test]
        #[should_panic(expected = "assertion failed")]
        fn test_insert_ten_panics() {
            let mut set = NumberSet::new();
            set.insert(10);
        }
    }

    mod set_operations {
        use super::*;

        #[test]
        fn test_union() {
            let cases = [
                (set![1, 2], set![2, 3], set![1, 2, 3]),
                (set![1], set![9], set![1, 9]),
                (set![], set![1, 2], set![1, 2]),
                (set![1, 2, 3], set![4, 5, 6], set![1, 2, 3, 4, 5, 6]),
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
                (set![1, 2, 3], set![2, 3, 4], set![2, 3]),
                (set![1, 2], set![3, 4], set![]),
                (set![1, 2, 3], set![1, 2, 3], set![1, 2, 3]),
                (set![], set![1, 2], set![]),
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
                (set![1, 2, 3], set![2, 3, 4], set![1]),
                (set![1, 2, 3], set![4, 5, 6], set![1, 2, 3]),
                (set![1, 2, 3], set![1, 2, 3], set![]),
                (set![], set![1, 2], set![]),
            ];
            for (a, b, expected) in cases {
                assert_eq!(a.difference(b), expected);
            }
        }

        #[test]
        fn test_symmetric_difference() {
            let cases = [
                (set![1, 2, 3], set![2, 3, 4], set![1, 4]),
                (set![1, 2], set![3, 4], set![1, 2, 3, 4]),
                (set![1, 2, 3], set![1, 2, 3], set![]),
                (set![], set![1, 2], set![1, 2]),
            ];
            for (a, b, expected) in cases {
                assert_eq!(a.symmetric_difference(b), expected);
                assert_eq!(b.symmetric_difference(a), expected); // Commutativity
                assert_eq!(a ^ b, expected); // Bit operator
            }
        }

        #[test]
        fn test_not() {
            let set = set![1, 3, 5, 7, 9];
            let complement = !set;
            assert_eq!(complement, set![2, 4, 6, 8]);
            assert_eq!(!NumberSet::EMPTY, NumberSet::FULL);
            assert_eq!(!NumberSet::FULL, NumberSet::EMPTY);
        }

        #[test]
        fn test_assign_operators() {
            let mut set = set![1, 2, 3];
            set |= set![3, 4, 5];
            assert_eq!(set, set![1, 2, 3, 4, 5]);

            set &= set![2, 3, 4];
            assert_eq!(set, set![2, 3, 4]);

            set ^= set![3, 4, 5];
            assert_eq!(set, set![2, 5]);
        }
    }

    mod relations {
        use super::*;

        #[test]
        fn test_is_subset() {
            let cases = [
                (set![1, 2], set![1, 2, 3], true),
                (set![1, 2, 3], set![1, 2], false),
                (set![1, 2], set![1, 2], true),
                (set![], set![1, 2], true),
                (set![1, 2], set![3, 4], false),
            ];
            for (a, b, expected) in cases {
                assert_eq!(a.is_subset(b), expected, "{a:?}.is_subset({b:?})");
            }
        }

        #[test]
        fn test_is_superset() {
            let cases = [
                (set![1, 2, 3], set![1, 2], true),
                (set![1, 2], set![1, 2, 3], false),
                (set![1, 2], set![1, 2], true),
                (set![1, 2], set![], true),
                (set![1, 2], set![3, 4], false),
            ];
            for (a, b, expected) in cases {
                assert_eq!(a.is_superset(b), expected, "{a:?}.is_superset({b:?})");
            }
        }

        #[test]
        fn test_is_disjoint() {
            let cases = [
                (set![1, 2], set![3, 4], true),
                (set![1, 2, 3], set![3, 4, 5], false),
                (set![], set![1, 2], true),
                (set![1], set![1], false),
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
            assert_eq!(set![3, 7, 1].first(), Some(1));
            assert_eq!(set![9].first(), Some(9));
            assert_eq!(set![].first(), None);
        }

        #[test]
        fn test_last() {
            assert_eq!(set![3, 7, 1].last(), Some(7));
            assert_eq!(set![1].last(), Some(1));
            assert_eq!(set![].last(), None);
        }

        #[test]
        fn test_pop_first() {
            let mut set = set![3, 7, 1];
            assert_eq!(set.pop_first(), Some(1));
            assert_eq!(set.pop_first(), Some(3));
            assert_eq!(set.pop_first(), Some(7));
            assert_eq!(set.pop_first(), None);
        }

        #[test]
        fn test_pop_last() {
            let mut set = set![3, 7, 1];
            assert_eq!(set.pop_last(), Some(7));
            assert_eq!(set.pop_last(), Some(3));
            assert_eq!(set.pop_last(), Some(1));
            assert_eq!(set.pop_last(), None);
        }

        #[test]
        fn test_range() {
            let set = set![1, 3, 5, 7, 9];
            assert_eq!(set.range(3..=7), set![3, 5, 7]);
            assert_eq!(set.range(3..7), set![3, 5]);
            assert_eq!(set.range(..5), set![1, 3]);
            assert_eq!(set.range(7..), set![7, 9]);
            assert_eq!(set.range(..), set);
        }
    }

    mod iteration {
        use super::*;

        #[test]
        fn test_iter_ascending() {
            let set = set![5, 1, 9, 3];
            let vec: Vec<u8> = set.iter().collect();
            assert_eq!(vec, vec![1, 3, 5, 9]);
        }

        #[test]
        fn test_iter_double_ended() {
            let set = set![1, 3, 5, 7, 9];
            let mut iter = set.iter();
            assert_eq!(iter.next(), Some(1));
            assert_eq!(iter.next_back(), Some(9));
            assert_eq!(iter.next(), Some(3));
            assert_eq!(iter.next_back(), Some(7));
            assert_eq!(iter.next(), Some(5));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
        }

        #[test]
        fn test_iter_size_hint() {
            let set = set![1, 3, 5];
            let iter = set.iter();
            assert_eq!(iter.size_hint(), (3, Some(3)));
            assert_eq!(iter.len(), 3);
        }

        #[test]
        fn test_into_iter() {
            let set = set![1, 3, 5];
            let vec: Vec<u8> = set.into_iter().collect();
            assert_eq!(vec, vec![1, 3, 5]);
        }

        #[test]
        fn test_iter_ref() {
            let set = set![1, 3, 5];
            let vec: Vec<u8> = (&set).into_iter().collect();
            assert_eq!(vec, vec![1, 3, 5]);
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_boundary_values() {
            let mut set = NumberSet::new();
            set.insert(1);
            set.insert(9);
            assert_eq!(set.len(), 2);
            assert!(set.contains(1));
            assert!(set.contains(9));
        }

        #[test]
        fn test_all_operations_on_empty() {
            let empty = NumberSet::EMPTY;
            assert_eq!(empty.len(), 0);
            assert_eq!(empty.first(), None);
            assert_eq!(empty.last(), None);
            assert_eq!(empty.union(empty), empty);
            assert_eq!(empty.intersection(empty), empty);
            assert_eq!(!empty, NumberSet::FULL);
        }

        #[test]
        fn test_all_operations_on_full() {
            let full = NumberSet::FULL;
            assert_eq!(full.len(), 9);
            assert_eq!(full.first(), Some(1));
            assert_eq!(full.last(), Some(9));
            assert_eq!(full.union(full), full);
            assert_eq!(full.intersection(full), full);
            assert_eq!(!full, NumberSet::EMPTY);
        }

        #[test]
        fn test_single_element_sets() {
            for n in 1..=9 {
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
                set![1],
                set![1, 2, 3],
                set![1, 3, 5, 7, 9],
                NumberSet::FULL,
            ];
            for set in cases {
                assert_eq!(set.len(), set.iter().count());
            }
        }

        #[test]
        fn test_insert_remove_roundtrip() {
            for n in 1..=9 {
                let mut set = NumberSet::new();
                set.insert(n);
                assert!(set.contains(n));
                set.remove(n);
                assert!(!set.contains(n));
                assert!(set.is_empty());
            }
        }

        #[test]
        fn test_union_size_bound() {
            let a = set![1, 2, 3];
            let b = set![3, 4, 5];
            let u = a.union(b);
            assert!(u.len() >= a.len());
            assert!(u.len() >= b.len());
            assert!(u.len() <= a.len() + b.len());
        }
    }

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Strategy to generate valid numbers (1-9)
        fn valid_number() -> impl Strategy<Value = u8> {
            1u8..=9
        }

        // Strategy to generate NumberSet
        fn number_set() -> impl Strategy<Value = NumberSet> {
            prop::collection::vec(valid_number(), 0..=9).prop_map(NumberSet::from_iter)
        }

        proptest! {
            // Set operations are commutative
            #[test]
            fn prop_union_commutative(a in number_set(), b in number_set()) {
                prop_assert_eq!(a.union(b), b.union(a));
                prop_assert_eq!(a | b, b | a);
            }

            #[test]
            fn prop_intersection_commutative(a in number_set(), b in number_set()) {
                prop_assert_eq!(a.intersection(b), b.intersection(a));
                prop_assert_eq!(a & b, b & a);
            }

            #[test]
            fn prop_symmetric_difference_commutative(a in number_set(), b in number_set()) {
                prop_assert_eq!(a.symmetric_difference(b), b.symmetric_difference(a));
                prop_assert_eq!(a ^ b, b ^ a);
            }

            // Set operations are associative
            #[test]
            fn prop_union_associative(a in number_set(), b in number_set(), c in number_set()) {
                prop_assert_eq!(a.union(b).union(c), a.union(b.union(c)));
            }

            #[test]
            fn prop_intersection_associative(a in number_set(), b in number_set(), c in number_set()) {
                prop_assert_eq!(a.intersection(b).intersection(c), a.intersection(b.intersection(c)));
            }

            // Idempotent operations
            #[test]
            fn prop_union_idempotent(a in number_set()) {
                prop_assert_eq!(a.union(a), a);
            }

            #[test]
            fn prop_intersection_idempotent(a in number_set()) {
                prop_assert_eq!(a.intersection(a), a);
            }

            // Identity elements
            #[test]
            fn prop_union_identity(a in number_set()) {
                prop_assert_eq!(a.union(NumberSet::EMPTY), a);
                prop_assert_eq!(NumberSet::EMPTY.union(a), a);
            }

            #[test]
            fn prop_intersection_identity(a in number_set()) {
                prop_assert_eq!(a.intersection(NumberSet::FULL), a);
                prop_assert_eq!(NumberSet::FULL.intersection(a), a);
            }

            // Absorption laws
            #[test]
            fn prop_union_intersection_absorption(a in number_set(), b in number_set()) {
                prop_assert_eq!(a.union(a.intersection(b)), a);
            }

            #[test]
            fn prop_intersection_union_absorption(a in number_set(), b in number_set()) {
                prop_assert_eq!(a.intersection(a.union(b)), a);
            }

            // De Morgan's laws
            #[test]
            fn prop_de_morgan_union(a in number_set(), b in number_set()) {
                prop_assert_eq!(!(a.union(b)), (!a).intersection(!b));
            }

            #[test]
            fn prop_de_morgan_intersection(a in number_set(), b in number_set()) {
                prop_assert_eq!(!(a.intersection(b)), (!a).union(!b));
            }

            // Double negation
            #[test]
            fn prop_double_negation(a in number_set()) {
                prop_assert_eq!(!!a, a);
            }

            // Difference properties
            #[test]
            fn prop_difference_is_disjoint(a in number_set(), b in number_set()) {
                let diff = a.difference(b);
                prop_assert!(diff.is_disjoint(b));
            }

            #[test]
            fn prop_difference_subset(a in number_set(), b in number_set()) {
                let diff = a.difference(b);
                prop_assert!(diff.is_subset(a));
            }

            // Symmetric difference properties
            #[test]
            fn prop_symmetric_difference_involution(a in number_set(), b in number_set()) {
                prop_assert_eq!(a.symmetric_difference(b).symmetric_difference(b), a);
            }

            // Subset/superset properties
            #[test]
            fn prop_subset_reflexive(a in number_set()) {
                prop_assert!(a.is_subset(a));
            }

            #[test]
            fn prop_superset_reflexive(a in number_set()) {
                prop_assert!(a.is_superset(a));
            }

            #[test]
            fn prop_empty_subset(a in number_set()) {
                prop_assert!(NumberSet::EMPTY.is_subset(a));
            }

            #[test]
            fn prop_full_superset(a in number_set()) {
                prop_assert!(NumberSet::FULL.is_superset(a));
            }

            // Iterator properties
            #[test]
            fn prop_len_equals_count(a in number_set()) {
                prop_assert_eq!(a.len(), a.iter().count());
            }

            #[test]
            fn prop_iter_sorted(a in number_set()) {
                let vec: Vec<u8> = a.iter().collect();
                for i in 1..vec.len() {
                    prop_assert!(vec[i - 1] < vec[i]);
                }
            }

            #[test]
            fn prop_iter_double_ended_consistent(a in number_set()) {
                let forward: Vec<u8> = a.iter().collect();
                let backward: Vec<u8> = a.iter().rev().collect();
                prop_assert_eq!(forward, backward.into_iter().rev().collect::<Vec<_>>());
            }

            // Insert/remove properties
            #[test]
            fn prop_insert_increases_or_maintains_len(mut a in number_set(), n in valid_number()) {
                let old_len = a.len();
                a.insert(n);
                prop_assert!(a.len() >= old_len);
                prop_assert!(a.len() <= old_len + 1);
            }

            #[test]
            fn prop_remove_decreases_or_maintains_len(mut a in number_set(), n in valid_number()) {
                let old_len = a.len();
                a.remove(n);
                prop_assert!(a.len() <= old_len);
                prop_assert!(a.len() >= old_len.saturating_sub(1));
            }

            #[test]
            fn prop_insert_contains(mut a in number_set(), n in valid_number()) {
                a.insert(n);
                prop_assert!(a.contains(n));
            }

            #[test]
            fn prop_remove_not_contains(mut a in number_set(), n in valid_number()) {
                a.remove(n);
                prop_assert!(!a.contains(n));
            }

            // Bit operators match methods
            #[test]
            fn prop_bitor_equals_union(a in number_set(), b in number_set()) {
                prop_assert_eq!(a | b, a.union(b));
            }

            #[test]
            fn prop_bitand_equals_intersection(a in number_set(), b in number_set()) {
                prop_assert_eq!(a & b, a.intersection(b));
            }

            #[test]
            fn prop_bitxor_equals_symmetric_difference(a in number_set(), b in number_set()) {
                prop_assert_eq!(a ^ b, a.symmetric_difference(b));
            }

            // Bounds
            #[test]
            fn prop_len_bounded(a in number_set()) {
                prop_assert!(a.len() <= 9);
            }

            #[test]
            fn prop_first_in_range(a in number_set()) {
                if let Some(n) = a.first() {
                    prop_assert!((1..=9).contains(&n));
                    prop_assert!(a.contains(n));
                }
            }

            #[test]
            fn prop_last_in_range(a in number_set()) {
                if let Some(n) = a.last() {
                    prop_assert!((1..=9).contains(&n));
                    prop_assert!(a.contains(n));
                }
            }

            #[test]
            fn prop_first_less_equal_last(a in number_set()) {
                if let (Some(first), Some(last)) = (a.first(), a.last()) {
                    prop_assert!(first <= last);
                }
            }

            // Range properties
            #[test]
            fn prop_range_subset(a in number_set(), start in 0u8..=10, end in 0u8..=10) {
                let ranged = a.range(start..end);
                prop_assert!(ranged.is_subset(a));
            }
        }
    }
}
