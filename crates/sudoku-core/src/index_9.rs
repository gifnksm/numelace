//! Index types and semantics for 9-element sets.
//!
//! This module provides [`Index9`] and [`Index9Semantics`], which are used to
//! define how values map to bit indices in generic 9-element containers like
//! [`BitSet9`](crate::bit_set_9::BitSet9).
//!
//! # Examples
//!
//! ```
//! use sudoku_core::index_9::{Index9, Index9Semantics};
//!
//! // Define semantics that map digits 1-9 to indices 0-8
//! struct DigitSemantics;
//!
//! impl Index9Semantics for DigitSemantics {
//!     type Value = u8;
//!
//!     fn to_index(value: u8) -> Index9 {
//!         assert!((1..=9).contains(&value));
//!         Index9::new(value - 1)
//!     }
//!
//!     fn from_index(index: Index9) -> u8 {
//!         index.index() + 1
//!     }
//! }
//! ```

/// A bit index in the range 0-8.
///
/// This type represents a valid index into a 9-element container (such as a 9-bit bitset
/// or a 9-element array). It ensures at construction time that the index is within the
/// valid range.
#[derive(Debug, Clone, Copy)]
pub struct Index9 {
    index: u8,
}

impl Index9 {
    /// Creates a new bit index.
    ///
    /// # Panics
    ///
    /// Panics if `index` is not in the range 0-8.
    #[must_use]
    pub const fn new(index: u8) -> Self {
        assert!(index < 9);
        Self { index }
    }

    /// Returns the underlying index value (0-8).
    #[must_use]
    pub const fn index(self) -> u8 {
        self.index
    }

    pub(crate) const fn bit(self) -> u16 {
        1 << self.index
    }

    /// Returns an iterator over all 9 valid bit indices (0-8).
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::index_9::Index9;
    /// let indices: Vec<_> = Index9::all().collect();
    /// assert_eq!(indices.len(), 9);
    /// assert_eq!(indices[0].index(), 0);
    /// assert_eq!(indices[8].index(), 8);
    /// ```
    pub fn all() -> impl Iterator<Item = Self> {
        (0..9).map(Index9::new)
    }
}

/// Defines the semantics for mapping values to indices in 9-element containers.
///
/// This trait allows generic containers like [`BitSet9`](crate::bit_set_9::BitSet9)
/// to work with different value types and mappings. Implementors define how user-facing
/// values are converted to and from internal indices (0-8).
///
/// This trait is used by:
/// - [`BitSet9`](crate::bit_set_9::BitSet9) - 9-bit sets
/// - (Future) `Array9` - 9-element arrays with semantic indexing
///
/// # Common Implementations
///
/// - [`DigitSemantics`] - Maps digits 1-9 to indices 0-8
/// - [`CellIndexSemantics`] - Direct 0-8 mapping
///
/// # Examples
///
/// ```
/// use sudoku_core::index_9::{Index9, Index9Semantics};
///
/// // A semantics that maps 1-9 to indices 0-8
/// struct NumberSemantics;
///
/// impl Index9Semantics for NumberSemantics {
///     type Value = u8;
///
///     fn to_index(value: u8) -> Index9 {
///         assert!((1..=9).contains(&value));
///         Index9::new(value - 1)
///     }
///
///     fn from_index(index: Index9) -> u8 {
///         index.index() + 1
///     }
/// }
/// ```
pub trait Index9Semantics {
    /// The type of values that can be stored in the set.
    type Value;

    /// Converts a value to a bit index.
    ///
    /// # Panics
    ///
    /// Should panic if the value cannot be represented as a valid bit index (0-8).
    fn to_index(value: Self::Value) -> Index9;

    /// Converts a bit index back to a value.
    fn from_index(index: Index9) -> Self::Value;
}

/// Semantics for digits 1-9.
///
/// This type implements [`Index9Semantics`]
/// to map user-facing digit values (1-9) to internal bit indices (0-8).
///
/// This is the standard semantics for sudoku digits, where digit 1 maps to
/// index 0, digit 2 to index 1, and so on.
///
/// # Panics
///
/// The `to_index` method panics if a value outside the range 1-9 is provided.
///
/// # Examples
///
/// ```
/// use sudoku_core::index_9::{Index9, Index9Semantics, DigitSemantics};
///
/// // Digit 1 maps to index 0
/// let index = DigitSemantics::to_index(1);
/// assert_eq!(index.index(), 0);
///
/// // Digit 9 maps to index 8
/// let index = DigitSemantics::to_index(9);
/// assert_eq!(index.index(), 8);
///
/// // Index 0 maps back to digit 1
/// let digit = DigitSemantics::from_index(Index9::new(0));
/// assert_eq!(digit, 1);
/// ```
#[derive(Debug)]
pub struct DigitSemantics;

impl Index9Semantics for DigitSemantics {
    type Value = u8;

    fn to_index(value: Self::Value) -> Index9 {
        assert!(
            (1..=9).contains(&value),
            "Number must be between 1 and 9, got {value}"
        );
        Index9::new(value - 1)
    }

    fn from_index(index: Index9) -> Self::Value {
        index.index() + 1
    }
}

/// Semantics for cell indices (0-8) within a house.
///
/// This type implements [`Index9Semantics`] with a direct identity mapping
/// where values 0-8 map to indices 0-8.
///
/// This is useful for representing positions within a sudoku house (row, column, or box),
/// where cells are naturally indexed 0-8.
///
/// # Panics
///
/// The `to_index` method panics if a value is 9 or greater.
///
/// # Examples
///
/// ```
/// use sudoku_core::index_9::{Index9, Index9Semantics, CellIndexSemantics};
///
/// // Direct mapping
/// let index = CellIndexSemantics::to_index(0);
/// assert_eq!(index.index(), 0);
///
/// let index = CellIndexSemantics::to_index(8);
/// assert_eq!(index.index(), 8);
///
/// // Round-trip
/// let value = CellIndexSemantics::from_index(Index9::new(5));
/// assert_eq!(value, 5);
/// ```
#[derive(Debug)]
pub struct CellIndexSemantics;

impl Index9Semantics for CellIndexSemantics {
    type Value = u8;

    fn to_index(value: Self::Value) -> Index9 {
        assert!(value < 9, "Cell index must be 0-8, got {value}");
        Index9::new(value)
    }

    fn from_index(index: Index9) -> Self::Value {
        index.index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod digit_semantics {
        use super::*;

        #[test]
        fn test_digit_to_index() {
            assert_eq!(DigitSemantics::to_index(1).index(), 0);
            assert_eq!(DigitSemantics::to_index(5).index(), 4);
            assert_eq!(DigitSemantics::to_index(9).index(), 8);
        }

        #[test]
        fn test_index_to_digit() {
            assert_eq!(DigitSemantics::from_index(Index9::new(0)), 1);
            assert_eq!(DigitSemantics::from_index(Index9::new(4)), 5);
            assert_eq!(DigitSemantics::from_index(Index9::new(8)), 9);
        }

        #[test]
        fn test_round_trip() {
            for digit in 1..=9 {
                let index = DigitSemantics::to_index(digit);
                let result = DigitSemantics::from_index(index);
                assert_eq!(result, digit);
            }
        }

        #[test]
        #[should_panic(expected = "Number must be between 1 and 9")]
        fn test_rejects_zero() {
            DigitSemantics::to_index(0);
        }

        #[test]
        #[should_panic(expected = "Number must be between 1 and 9")]
        fn test_rejects_ten() {
            DigitSemantics::to_index(10);
        }
    }

    mod cell_index_semantics {
        use super::*;

        #[test]
        fn test_identity_mapping() {
            for i in 0..9 {
                assert_eq!(CellIndexSemantics::to_index(i).index(), i);
                assert_eq!(CellIndexSemantics::from_index(Index9::new(i)), i);
            }
        }

        #[test]
        fn test_round_trip() {
            for value in 0..9 {
                let index = CellIndexSemantics::to_index(value);
                let result = CellIndexSemantics::from_index(index);
                assert_eq!(result, value);
            }
        }

        #[test]
        #[should_panic(expected = "Cell index must be 0-8")]
        fn test_rejects_nine() {
            CellIndexSemantics::to_index(9);
        }

        #[test]
        #[should_panic(expected = "Cell index must be 0-8")]
        fn test_rejects_larger() {
            CellIndexSemantics::to_index(10);
        }
    }
}
