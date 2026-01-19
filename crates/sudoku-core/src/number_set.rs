//! A set of numbers from 1 to 9, optimized for sudoku cells.
//!
//! This module provides [`NumberSet`], a specialized implementation of
//! [`BitSet9`] for representing sets of numbers 1-9.
//!
//! # Examples
//!
//! ```
//! use sudoku_core::number_set::NumberSet;
//!
//! let mut set = NumberSet::new();
//! set.insert(1);
//! set.insert(5);
//! set.insert(9);
//!
//! assert_eq!(set.len(), 3);
//! assert!(set.contains(5));
//! ```

use crate::bit_set_9::{BitIndex9, BitSet9, BitSet9Semantics};

/// Semantics for numbers 1-9.
///
/// This type implements [`BitSet9Semantics`]
/// to map user-facing values (1-9) to internal bit indices (0-8).
///
/// # Panics
///
/// The `to_index` method panics if a value outside the range 1-9 is provided.
#[derive(Debug)]
pub struct NumberSemantics;

impl BitSet9Semantics for NumberSemantics {
    type Value = u8;
    fn to_index(value: Self::Value) -> BitIndex9 {
        assert!(
            (1..=9).contains(&value),
            "Number must be between 1 and 9, got {value}"
        );
        BitIndex9::new(value - 1)
    }
    fn from_index(index: BitIndex9) -> Self::Value {
        index.index() + 1
    }
}

/// A set of numbers from 1 to 9, represented as a bitset.
///
/// This is a specialized version of [`BitSet9`] that represents
/// numbers in the range 1-9, commonly used in sudoku puzzles.
///
/// The implementation uses a 16-bit integer where bits 0-8 represent numbers 1-9 respectively,
/// providing efficient storage and fast set operations.
///
/// # Examples
///
/// ```
/// use sudoku_core::number_set::NumberSet;
///
/// // Create a set with all candidates available
/// let mut candidates = NumberSet::FULL;
///
/// // Remove some numbers
/// candidates.remove(5);
/// candidates.remove(7);
///
/// assert_eq!(candidates.len(), 7);
/// assert!(!candidates.contains(5));
/// assert!(candidates.contains(1));
/// ```
///
/// # Set Operations
///
/// ```
/// use sudoku_core::number_set::NumberSet;
///
/// let a = NumberSet::from_iter([1, 2, 3]);
/// let b = NumberSet::from_iter([2, 3, 4]);
///
/// // Union
/// let union = a | b;
/// assert_eq!(union, NumberSet::from_iter([1, 2, 3, 4]));
///
/// // Intersection
/// let intersection = a & b;
/// assert_eq!(intersection, NumberSet::from_iter([2, 3]));
///
/// // Difference
/// let diff = a.difference(b);
/// assert_eq!(diff, NumberSet::from_iter([1]));
/// ```
pub type NumberSet = BitSet9<NumberSemantics>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_range() {
        let mut set = NumberSet::new();
        set.insert(1);
        set.insert(9);
        assert!(set.contains(1));
        assert!(set.contains(9));
        assert_eq!(set.len(), 2);
    }

    #[test]
    #[should_panic(expected = "Number must be")]
    fn test_rejects_zero() {
        let mut set = NumberSet::new();
        set.insert(0);
    }

    #[test]
    #[should_panic(expected = "Number must be")]
    fn test_rejects_ten() {
        let mut set = NumberSet::new();
        set.insert(10);
    }

    #[test]
    fn test_from_iter() {
        let set = NumberSet::from_iter([1, 5, 9]);
        assert_eq!(set.len(), 3);
        assert!(set.contains(1));
        assert!(set.contains(5));
        assert!(set.contains(9));
    }

    #[test]
    fn test_iteration_order() {
        let set = NumberSet::from_iter([9, 1, 5, 3]);
        let collected: Vec<_> = set.iter().collect();
        assert_eq!(collected, vec![1, 3, 5, 9]);
    }

    #[test]
    fn test_operations() {
        let a = NumberSet::from_iter([1, 2, 3]);
        let b = NumberSet::from_iter([2, 3, 4]);

        assert_eq!(a.union(b).len(), 4);
        assert_eq!(a.intersection(b).len(), 2);
        assert_eq!(a.difference(b).len(), 1);
    }

    #[test]
    fn test_constants() {
        assert_eq!(NumberSet::EMPTY.len(), 0);
        assert_eq!(NumberSet::FULL.len(), 9);

        for n in 1..=9 {
            assert!(NumberSet::FULL.contains(n));
        }
    }
}
