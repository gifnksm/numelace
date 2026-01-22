//! Index types and semantics for 81-element containers.
//!
//! Provides [`Index81`] and [`Index81Semantics`] for type-safe indexing into 81-element
//! containers like [`BitSet81`], typically used for board positions.
//!
//! [`BitSet81`]: crate::containers::BitSet81

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
    #[inline]
    pub const fn new(index: u8) -> Self {
        assert!(index < 81);
        Self { index }
    }

    /// Returns the underlying index value (0-80).
    #[must_use]
    #[inline]
    pub const fn index(self) -> u8 {
        self.index
    }

    #[inline]
    pub(crate) const fn bit(self) -> u128 {
        1 << self.index
    }

    /// Returns an iterator over all 81 valid bit indices (0-80).
    ///
    /// # Examples
    ///
    /// ```
    /// # use sudoku_core::index::Index81;
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
/// This trait allows generic containers like [`BitSet81`]
/// to work with different value types and mappings. Implementors define how user-facing
/// values are converted to and from internal indices (0-80).
///
/// This trait is the foundation of the [Semantics Pattern](crate#semantics-pattern-type-safe-indexing),
/// providing compile-time type safety for 81-element containers.
///
/// This trait is used by:
/// - [`BitSet81`] - 81-bit sets
/// - [`Array81`] - 81-element arrays with semantic indexing
///
/// [`BitSet81`]: crate::containers::BitSet81
/// [`Array81`]: crate::containers::Array81
///
/// # Common Implementations
///
/// - [`PositionSemantics`] - Maps [`Position`] to row-major indices
///
/// [`Position`]: crate::Position
///
/// See the [crate-level documentation](crate#semantics-pattern-type-safe-indexing) for details.
///
/// # Examples
///
/// ```
/// use sudoku_core::index::{Index81, Index81Semantics};
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
/// [`Position`] coordinates to row-major indices (index = y * 9 + x),
/// providing type safety through the [Semantics Pattern](crate#semantics-pattern-type-safe-indexing).
///
/// This is the standard semantics for sudoku board positions, where a position
/// at (x, y) maps to index y * 9 + x (row-major order).
///
/// [`Position`]: crate::Position
///
/// # Examples
///
/// ```
/// use sudoku_core::{
///     Position,
///     index::{Index81, Index81Semantics, PositionSemantics},
/// };
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

    #[inline]
    fn to_index(value: Self::Value) -> Index81 {
        Index81::new(value.y() * 9 + value.x())
    }

    #[inline]
    fn from_index(index: Index81) -> Self::Value {
        let i = index.index();
        Self::Value::new(i % 9, i / 9)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;

    #[test]
    fn test_position_semantics() {
        // Boundary values: corners and center
        assert_eq!(PositionSemantics::to_index(Position::new(0, 0)).index(), 0);
        assert_eq!(PositionSemantics::to_index(Position::new(8, 0)).index(), 8);
        assert_eq!(PositionSemantics::to_index(Position::new(0, 8)).index(), 72);
        assert_eq!(PositionSemantics::to_index(Position::new(8, 8)).index(), 80);
        assert_eq!(PositionSemantics::to_index(Position::new(4, 4)).index(), 40);

        assert_eq!(
            PositionSemantics::from_index(Index81::new(0)),
            Position::new(0, 0)
        );
        assert_eq!(
            PositionSemantics::from_index(Index81::new(80)),
            Position::new(8, 8)
        );
        assert_eq!(
            PositionSemantics::from_index(Index81::new(40)),
            Position::new(4, 4)
        );

        // Row-major order: y * 9 + x
        for x in 0..9 {
            assert_eq!(PositionSemantics::to_index(Position::new(x, 0)).index(), x);
            assert_eq!(
                PositionSemantics::to_index(Position::new(x, 1)).index(),
                9 + x
            );
        }

        // Round-trip for all positions
        for pos in Position::ALL {
            let index = PositionSemantics::to_index(pos);
            let result = PositionSemantics::from_index(index);
            assert_eq!(result, pos);
        }
    }
}
