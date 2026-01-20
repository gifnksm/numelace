//! Candidate digits (1-9) for a single cell.
//!
//! This module provides [`DigitCandidates`], a specialized implementation of
//! [`BitSet9`] for representing sets of digits 1-9.
//!
//! The [`DigitSemantics`] type (defined in [`index`])
//! implements [`Index9Semantics`]
//! to map digits 1-9 to internal bit indices 0-8.
//!
//! [`index`]: crate::index
//! [`Index9Semantics`]: crate::index::Index9Semantics
//!
//! # Examples
//!
//! ```
//! use sudoku_core::{Digit, DigitCandidates};
//!
//! let mut candidates = DigitCandidates::new();
//! candidates.insert(Digit::D1);
//! candidates.insert(Digit::D5);
//! candidates.insert(Digit::D9);
//!
//! assert_eq!(candidates.len(), 3);
//! assert!(candidates.contains(Digit::D5));
//! assert!(!candidates.contains(Digit::D2));
//!
//! // Remove a candidate
//! candidates.remove(Digit::D5);
//! assert_eq!(candidates.len(), 2);
//! ```

use crate::{containers::BitSet9, index::DigitSemantics};

/// A set of candidate digits (1-9) for a single cell.
///
/// This is a specialized version of [`BitSet9`] that represents
/// digits in the range 1-9, commonly used to track which digits
/// can be placed in a sudoku cell.
///
/// The implementation uses a 16-bit integer where bits 0-8 represent digits 1-9 respectively,
/// providing efficient storage and fast set operations.
///
/// # Examples
///
/// ```
/// use sudoku_core::{Digit, DigitCandidates};
///
/// // Create a set with all candidates available
/// let mut candidates = DigitCandidates::FULL;
///
/// // Remove some numbers
/// candidates.remove(Digit::D5);
/// candidates.remove(Digit::D7);
///
/// assert_eq!(candidates.len(), 7);
/// assert!(!candidates.contains(Digit::D5));
/// assert!(candidates.contains(Digit::D1));
/// ```
///
/// # Set Operations
///
/// ```
/// use sudoku_core::{Digit, DigitCandidates};
///
/// let a = DigitCandidates::from_iter([Digit::D1, Digit::D2, Digit::D3]);
/// let b = DigitCandidates::from_iter([Digit::D2, Digit::D3, Digit::D4]);
///
/// // Union
/// let union = a | b;
/// assert_eq!(
///     union,
///     DigitCandidates::from_iter([Digit::D1, Digit::D2, Digit::D3, Digit::D4])
/// );
///
/// // Intersection
/// let intersection = a & b;
/// assert_eq!(
///     intersection,
///     DigitCandidates::from_iter([Digit::D2, Digit::D3])
/// );
///
/// // Difference
/// let diff = a.difference(b);
/// assert_eq!(diff, DigitCandidates::from_iter([Digit::D1]));
/// ```
pub type DigitCandidates = BitSet9<DigitSemantics>;

#[cfg(test)]
mod tests {
    use crate::digit::Digit::{self, *};

    use super::*;

    #[test]
    fn test_number_range() {
        let mut set = DigitCandidates::new();
        set.insert(D1);
        set.insert(D9);
        assert!(set.contains(D1));
        assert!(set.contains(D9));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_from_iter() {
        let set = DigitCandidates::from_iter([D1, D5, D9]);
        assert_eq!(set.len(), 3);
        assert!(set.contains(D1));
        assert!(set.contains(D5));
        assert!(set.contains(D9));
    }

    #[test]
    fn test_iteration_order() {
        let set = DigitCandidates::from_iter([D9, D1, D5, D3]);
        let collected: Vec<_> = set.iter().collect();
        assert_eq!(collected, vec![D1, D3, D5, D9]);
    }

    #[test]
    fn test_operations() {
        let a = DigitCandidates::from_iter([D1, D2, D3]);
        let b = DigitCandidates::from_iter([D2, D3, D4]);

        assert_eq!(a.union(b).len(), 4);
        assert_eq!(a.intersection(b).len(), 2);
        assert_eq!(a.difference(b).len(), 1);
    }

    #[test]
    fn test_constants() {
        assert_eq!(DigitCandidates::EMPTY.len(), 0);
        assert_eq!(DigitCandidates::FULL.len(), 9);

        for n in Digit::ALL {
            assert!(DigitCandidates::FULL.contains(n));
        }
    }
}
