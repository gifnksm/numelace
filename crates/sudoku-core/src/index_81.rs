//! Index types and semantics for 81-element sets.
//!
//! This module provides [`Index81`] and [`Index81Semantics`], which are used to
//! define how values map to bit indices in generic 81-element containers like
//! [`BitSet81`](crate::bit_set_81::BitSet81).
//!
//! # Examples
//!
//! ```
//! use sudoku_core::index_81::{Index81, Index81Semantics};
//! use sudoku_core::Position;
//!
//! // Define semantics that map Position to indices
//! struct PositionSemantics;
//!
//! impl Index81Semantics for PositionSemantics {
//!     type Value = Position;
//!
//!     fn to_index(value: Position) -> Index81 {
//!         Index81::new(value.y() * 9 + value.x())
//!     }
//!
//!     fn from_index(index: Index81) -> Position {
//!         let i = index.index();
//!         Position::new(i % 9, i / 9)
//!     }
//! }
//! ```

/// A bit index in the range 0-80.
///
/// This type represents a valid index into an 81-element container (such as an 81-bit bitset
/// or an 81-element array). It ensures at construction time that the index is within the
/// valid range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index81 {
    index: u8,
}

impl Index81 {
    /// Creates a new bit index.
    ///
    /// # Panics
    ///
    /// Panics if `index` is not in the range 0-80.
    #[must_use]
    pub const fn new(index: u8) -> Self {
        assert!(index < 81);
        Self { index }
    }

    /// Returns the underlying index value (0-80).
    #[must_use]
    pub const fn index(self) -> u8 {
        self.index
    }

    pub(crate) const fn bit(self) -> u128 {
        1 << self.index
    }

    /// Returns an iterator over all 81 valid bit indices (0-80).
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::index_81::Index81;
    /// let indices: Vec<_> = Index81::all().collect();
    /// assert_eq!(indices.len(), 81);
    /// assert_eq!(indices[0].index(), 0);
    /// assert_eq!(indices[80].index(), 80);
    /// ```
    pub fn all() -> impl Iterator<Item = Self> {
        (0..81).map(Index81::new)
    }
}

/// Defines the semantics for mapping values to indices in 81-element containers.
///
/// This trait allows generic containers like [`BitSet81`](crate::bit_set_81::BitSet81)
/// to work with different value types and mappings. Implementors define how user-facing
/// values are converted to and from internal indices (0-80).
///
/// This trait is used by:
/// - [`BitSet81`](crate::bit_set_81::BitSet81) - 81-bit sets
/// - (Future) `Array81` - 81-element arrays with semantic indexing
///
/// # Common Implementations
///
/// - [`PositionSemantics`] - Maps [`Position`](crate::Position) to row-major indices
///
/// # Examples
///
/// ```
/// use sudoku_core::index_81::{Index81, Index81Semantics};
///
/// // A semantics that maps (x, y) coordinates to indices
/// struct PositionSemantics;
///
/// impl Index81Semantics for PositionSemantics {
///     type Value = (u8, u8);
///
///     fn to_index(value: (u8, u8)) -> Index81 {
///         let (x, y) = value;
///         assert!(x < 9 && y < 9);
///         Index81::new(y * 9 + x)
///     }
///
///     fn from_index(index: Index81) -> (u8, u8) {
///         let idx = index.index();
///         (idx % 9, idx / 9)
///     }
/// }
/// ```
pub trait Index81Semantics {
    /// The type of values that can be stored in the set.
    type Value;

    /// Converts a value to a bit index.
    ///
    /// # Panics
    ///
    /// Should panic if the value cannot be represented as a valid bit index (0-80).
    fn to_index(value: Self::Value) -> Index81;

    /// Converts a bit index back to a value.
    fn from_index(index: Index81) -> Self::Value;
}

/// Semantics for board positions.
///
/// This type implements [`Index81Semantics`] to map
/// [`Position`](crate::Position) coordinates to row-major indices (index = y * 9 + x).
///
/// This is the standard semantics for sudoku board positions, where a position
/// at (x, y) maps to index y * 9 + x (row-major order).
///
/// # Examples
///
/// ```
/// use sudoku_core::index_81::{Index81, Index81Semantics, PositionSemantics};
/// use sudoku_core::Position;
///
/// // Position (0, 0) maps to index 0
/// let index = PositionSemantics::to_index(Position::new(0, 0));
/// assert_eq!(index.index(), 0);
///
/// // Position (8, 8) maps to index 80
/// let index = PositionSemantics::to_index(Position::new(8, 8));
/// assert_eq!(index.index(), 80);
///
/// // Position (4, 4) maps to index 40 (center of board)
/// let index = PositionSemantics::to_index(Position::new(4, 4));
/// assert_eq!(index.index(), 40);
///
/// // Round-trip
/// let pos = PositionSemantics::from_index(Index81::new(40));
/// assert_eq!(pos, Position::new(4, 4));
/// ```
#[derive(Debug)]
pub struct PositionSemantics;

impl Index81Semantics for PositionSemantics {
    type Value = crate::Position;

    fn to_index(value: Self::Value) -> Index81 {
        Index81::new(value.y() * 9 + value.x())
    }

    fn from_index(index: Index81) -> Self::Value {
        let i = index.index();
        Self::Value::new(i % 9, i / 9)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;

    mod position_semantics {
        use super::*;

        #[test]
        fn test_corners() {
            let index = PositionSemantics::to_index(Position::new(0, 0));
            assert_eq!(index.index(), 0);

            let index = PositionSemantics::to_index(Position::new(8, 0));
            assert_eq!(index.index(), 8);

            let index = PositionSemantics::to_index(Position::new(0, 8));
            assert_eq!(index.index(), 72);

            let index = PositionSemantics::to_index(Position::new(8, 8));
            assert_eq!(index.index(), 80);
        }

        #[test]
        fn test_center() {
            let index = PositionSemantics::to_index(Position::new(4, 4));
            assert_eq!(index.index(), 40);
        }

        #[test]
        fn test_row_major_order() {
            // First row
            for x in 0..9 {
                let index = PositionSemantics::to_index(Position::new(x, 0));
                assert_eq!(index.index(), x);
            }

            // Second row
            for x in 0..9 {
                let index = PositionSemantics::to_index(Position::new(x, 1));
                assert_eq!(index.index(), 9 + x);
            }
        }

        #[test]
        fn test_from_index() {
            let pos = PositionSemantics::from_index(Index81::new(0));
            assert_eq!(pos, Position::new(0, 0));

            let pos = PositionSemantics::from_index(Index81::new(80));
            assert_eq!(pos, Position::new(8, 8));

            let pos = PositionSemantics::from_index(Index81::new(40));
            assert_eq!(pos, Position::new(4, 4));
        }

        #[test]
        fn test_round_trip_all_positions() {
            for y in 0..9 {
                for x in 0..9 {
                    let pos = Position::new(x, y);
                    let index = PositionSemantics::to_index(pos);
                    let result = PositionSemantics::from_index(index);
                    assert_eq!(result, pos);
                }
            }
        }

        #[test]
        fn test_all_indices() {
            for i in 0..81 {
                let index = Index81::new(i);
                let pos = PositionSemantics::from_index(index);
                let result = PositionSemantics::to_index(pos);
                assert_eq!(result.index(), i);
            }
        }
    }
}
