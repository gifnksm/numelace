//! Board position types.

use crate::DigitPositions;
use crate::containers::Array9;
use crate::index::CellIndexSemantics;

/// Board position (x, y) where x is column and y is row.
///
/// Both coordinates are in the range 0-8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    x: u8,
    y: u8,
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare by index order: y * 9 + x
        // This ensures ordering matches PositionSemantics::to_index
        (self.y * 9 + self.x).cmp(&(other.y * 9 + other.x))
    }
}

/// Errors that can occur when constructing a [`Position`] with validation.
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum PositionNewError {
    /// The x coordinate is outside the valid range (0-8).
    #[display("invalid x value: {_0}")]
    InvalidXValue(#[error(not(source))] u8),
    /// The y coordinate is outside the valid range (0-8).
    #[display("invalid y value: {_0}")]
    InvalidYValue(#[error(not(source))] u8),
}

impl Position {
    /// All 81 positions on the Sudoku board, in index order.
    ///
    /// The positions are ordered by row-major order (y * 9 + x):
    /// - Index 0: (0, 0) - top-left
    /// - Index 1: (1, 0)
    /// - ...
    /// - Index 8: (8, 0) - top-right
    /// - Index 9: (0, 1) - second row, first column
    /// - ...
    /// - Index 80: (8, 8) - bottom-right
    ///
    /// This order matches `PositionSemantics::to_index` and `Position::Ord`.
    pub const ALL: [Position; 81] = {
        let mut arr = [Position::new(0, 0); 81];
        let mut i = 0;
        while i < 81 {
            arr[i as usize] = Position::new(i % 9, i / 9);
            i += 1;
        }
        arr
    };

    /// All positions in each row, indexed by row number (0-8).
    ///
    /// `ROWS[y]` contains all 9 positions in row `y`, ordered by column (x = 0..9).
    ///
    /// # Example
    ///
    /// ```
    /// # use numelace_core::Position;
    /// // Process all positions in row 3
    /// for pos in Position::ROWS[3] {
    ///     assert_eq!(pos.y(), 3);
    /// }
    /// ```
    pub const ROWS: Array9<[Position; 9], CellIndexSemantics> = {
        let mut rows = [[Position::new(0, 0); 9]; 9];
        let mut y = 0;
        while y < 9 {
            let mut x = 0;
            while x < 9 {
                rows[y as usize][x as usize] = Position::new(x, y);
                x += 1;
            }
            y += 1;
        }
        Array9::from_array(rows)
    };

    /// All positions in each column, indexed by column number (0-8).
    ///
    /// `COLUMNS[x]` contains all 9 positions in column `x`, ordered by row (y = 0..9).
    ///
    /// # Example
    ///
    /// ```
    /// # use numelace_core::Position;
    /// // Process all positions in column 5
    /// for pos in Position::COLUMNS[5] {
    ///     assert_eq!(pos.x(), 5);
    /// }
    /// ```
    pub const COLUMNS: Array9<[Position; 9], CellIndexSemantics> = {
        let mut columns = [[Position::new(0, 0); 9]; 9];
        let mut x = 0;
        while x < 9 {
            let mut y = 0;
            while y < 9 {
                columns[x as usize][y as usize] = Position::new(x, y);
                y += 1;
            }
            x += 1;
        }
        Array9::from_array(columns)
    };

    /// All positions in each 3×3 box, indexed by box number (0-8).
    ///
    /// `BOXES[box_index]` contains all 9 positions in that box, ordered by
    /// cell index within the box (same order as `Position::from_box`).
    ///
    /// Box indices are arranged as:
    /// ```text
    /// 0 1 2
    /// 3 4 5
    /// 6 7 8
    /// ```
    ///
    /// # Example
    ///
    /// ```
    /// # use numelace_core::Position;
    /// // Process all positions in box 4 (center box)
    /// for pos in Position::BOXES[4] {
    ///     assert_eq!(pos.box_index(), 4);
    /// }
    /// ```
    pub const BOXES: Array9<[Position; 9], CellIndexSemantics> = {
        let mut boxes = [[Position::new(0, 0); 9]; 9];
        let mut box_index = 0;
        while box_index < 9 {
            let mut cell_index = 0;
            while cell_index < 9 {
                boxes[box_index as usize][cell_index as usize] =
                    Position::from_box(box_index, cell_index);
                cell_index += 1;
            }
            box_index += 1;
        }
        Array9::from_array(boxes)
    };

    /// Creates a new position from column and row coordinates.
    ///
    /// # Panics
    ///
    /// Panics if `x` or `y` is greater than or equal to 9.
    #[must_use]
    #[inline]
    pub const fn new(x: u8, y: u8) -> Self {
        assert!(x < 9 && y < 9);
        Self { x, y }
    }

    /// Attempts to create a new position from column and row coordinates.
    ///
    /// # Errors
    ///
    /// Returns [`PositionNewError::InvalidXValue`] if `x` is greater than or equal to 9,
    /// and [`PositionNewError::InvalidYValue`] if `y` is greater than or equal to 9.
    #[inline]
    pub const fn try_new(x: u8, y: u8) -> Result<Self, PositionNewError> {
        if x >= 9 {
            return Err(PositionNewError::InvalidXValue(x));
        }
        if y >= 9 {
            return Err(PositionNewError::InvalidYValue(y));
        }
        Ok(Self { x, y })
    }

    /// Returns a new position with the given column (x), preserving the row (y).
    ///
    /// # Panics
    ///
    /// Panics if `x` is greater than or equal to 9.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_core::Position;
    ///
    /// let pos = Position::new(2, 4);
    /// assert_eq!(pos.with_x(7), Position::new(7, 4));
    /// ```
    #[must_use]
    pub const fn with_x(self, x: u8) -> Self {
        Self::new(x, self.y())
    }

    /// Returns a new position with the given row (y), preserving the column (x).
    ///
    /// # Panics
    ///
    /// Panics if `y` is greater than or equal to 9.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_core::Position;
    ///
    /// let pos = Position::new(2, 4);
    /// assert_eq!(pos.with_y(7), Position::new(2, 7));
    /// ```
    #[must_use]
    pub const fn with_y(self, y: u8) -> Self {
        Self::new(self.x(), y)
    }

    /// Returns the position one row above, or `None` if already at the top edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_core::Position;
    ///
    /// assert_eq!(Position::new(3, 5).up(), Some(Position::new(3, 4)));
    /// assert_eq!(Position::new(3, 0).up(), None);
    /// ```
    #[must_use]
    pub const fn up(self) -> Option<Self> {
        if self.y > 0 {
            Some(self.with_y(self.y - 1))
        } else {
            None
        }
    }

    /// Returns the position one row below, or `None` if already at the bottom edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_core::Position;
    ///
    /// assert_eq!(Position::new(3, 5).down(), Some(Position::new(3, 6)));
    /// assert_eq!(Position::new(3, 8).down(), None);
    /// ```
    #[must_use]
    pub const fn down(self) -> Option<Self> {
        if self.y < 8 {
            Some(self.with_y(self.y + 1))
        } else {
            None
        }
    }

    /// Returns the position one column to the left, or `None` if already at the left edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_core::Position;
    ///
    /// assert_eq!(Position::new(3, 5).left(), Some(Position::new(2, 5)));
    /// assert_eq!(Position::new(0, 5).left(), None);
    /// ```
    #[must_use]
    pub const fn left(self) -> Option<Self> {
        if self.x > 0 {
            Some(self.with_x(self.x - 1))
        } else {
            None
        }
    }

    /// Returns the position one column to the right, or `None` if already at the right edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_core::Position;
    ///
    /// assert_eq!(Position::new(3, 5).right(), Some(Position::new(4, 5)));
    /// assert_eq!(Position::new(8, 5).right(), None);
    /// ```
    #[must_use]
    pub const fn right(self) -> Option<Self> {
        if self.x < 8 {
            Some(self.with_x(self.x + 1))
        } else {
            None
        }
    }

    /// Creates a position from box index and cell index within that box.
    ///
    /// # Panics
    ///
    /// Panics if `box_index` or `cell_index` is greater than or equal to 9.
    #[must_use]
    #[inline]
    pub const fn from_box(box_index: u8, cell_index: u8) -> Self {
        assert!(box_index < 9 && cell_index < 9);
        let origin = Self::box_origin(box_index);
        Self::new(origin.x + cell_index % 3, origin.y + cell_index / 3)
    }

    /// Returns the column (x coordinate) of this position.
    #[must_use]
    #[inline]
    pub const fn x(self) -> u8 {
        self.x
    }

    /// Returns the row (y coordinate) of this position.
    #[must_use]
    #[inline]
    pub const fn y(self) -> u8 {
        self.y
    }

    /// Returns the box index (0-8) that this position belongs to.
    #[must_use]
    #[inline]
    pub const fn box_index(&self) -> u8 {
        (self.y / 3) * 3 + (self.x / 3)
    }

    /// Returns the relative position (0-8) within the box.
    #[must_use]
    #[inline]
    pub const fn box_cell_index(&self) -> u8 {
        (self.y % 3) * 3 + (self.x % 3)
    }

    /// Returns the top-left position (origin) of the specified box.
    ///
    /// # Panics
    ///
    /// Panics if `box_index` is greater than or equal to 9.
    #[must_use]
    #[inline]
    pub const fn box_origin(box_index: u8) -> Self {
        assert!(box_index < 9);
        Self::new((box_index % 3) * 3, (box_index / 3) * 3)
    }

    /// Returns the union of row, column, and box positions for this cell.
    ///
    /// The returned set includes this position itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_core::Position;
    ///
    /// let pos = Position::new(4, 4);
    /// let house = pos.house_positions();
    /// assert!(house.contains(pos));
    /// assert!(house.contains(Position::new(4, 0)));
    /// assert!(house.contains(Position::new(0, 4)));
    /// ```
    #[inline]
    #[must_use]
    pub fn house_positions(self) -> DigitPositions {
        DigitPositions::ROW_POSITIONS[self.y()]
            | DigitPositions::COLUMN_POSITIONS[self.x()]
            | DigitPositions::BOX_POSITIONS[self.box_index()]
    }

    /// Returns the positions that share a row, column, or box with this cell.
    ///
    /// The returned set excludes this position itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_core::Position;
    ///
    /// let pos = Position::new(4, 4);
    /// let peers = pos.house_peers();
    /// assert!(!peers.contains(pos));
    /// assert!(peers.contains(Position::new(4, 0)));
    /// assert!(peers.contains(Position::new(0, 4)));
    /// ```
    #[inline]
    #[must_use]
    pub fn house_peers(self) -> DigitPositions {
        let mut set = self.house_positions();
        set.remove(self);
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let pos = Position::new(3, 5);
        assert_eq!(pos.x(), 3);
        assert_eq!(pos.y(), 5);
        assert_eq!(pos.with_x(1), Position::new(1, 5));
        assert_eq!(pos.with_y(7), Position::new(3, 7));
        assert_eq!(pos.up(), Some(Position::new(3, 4)));
        assert_eq!(pos.down(), Some(Position::new(3, 6)));
        assert_eq!(pos.left(), Some(Position::new(2, 5)));
        assert_eq!(pos.right(), Some(Position::new(4, 5)));

        assert_eq!(Position::new(0, 0).box_index(), 0);
        assert_eq!(Position::new(4, 4).box_index(), 4);
        assert_eq!(Position::new(8, 8).box_index(), 8);

        assert_eq!(Position::new(0, 0).box_cell_index(), 0);
        assert_eq!(Position::new(4, 4).box_cell_index(), 4);
        assert_eq!(Position::new(8, 8).box_cell_index(), 8);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_new_position_x_too_large() {
        let _ = Position::new(9, 0);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_new_position_y_too_large() {
        let _ = Position::new(0, 9);
    }

    #[test]
    fn test_try_new() {
        assert_eq!(Position::try_new(0, 0).unwrap(), Position::new(0, 0));
        assert_eq!(Position::try_new(8, 8).unwrap(), Position::new(8, 8));
        assert!(matches!(
            Position::try_new(9, 0).unwrap_err(),
            PositionNewError::InvalidXValue(9)
        ));
        assert!(matches!(
            Position::try_new(0, 9).unwrap_err(),
            PositionNewError::InvalidYValue(9)
        ));
    }

    #[test]
    fn test_from_box() {
        assert_eq!(Position::from_box(0, 0), Position::new(0, 0));
        assert_eq!(Position::from_box(0, 8), Position::new(2, 2));
        assert_eq!(Position::from_box(4, 4), Position::new(4, 4));
        assert_eq!(Position::from_box(8, 8), Position::new(8, 8));

        assert_eq!(Position::box_origin(0), Position::new(0, 0));
        assert_eq!(Position::box_origin(4), Position::new(3, 3));
        assert_eq!(Position::box_origin(8), Position::new(6, 6));
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_from_box_invalid_box_index() {
        let _ = Position::from_box(9, 0);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_from_box_invalid_cell_index() {
        let _ = Position::from_box(0, 9);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_box_origin_invalid() {
        let _ = Position::box_origin(9);
    }

    #[test]
    fn test_from_box_roundtrip() {
        // from_box and box_index/box_cell_index are inverses
        for box_index in 0..9 {
            for cell_index in 0..9 {
                let pos = Position::from_box(box_index, cell_index);
                assert_eq!(pos.box_index(), box_index);
                assert_eq!(pos.box_cell_index(), cell_index);
            }
        }
    }

    #[test]
    fn test_house_positions() {
        let pos = Position::new(4, 4);
        let house = pos.house_positions();

        assert!(house.contains(pos));
        assert_eq!(house.len(), 21);
        assert!(house.contains(Position::new(4, 0)));
        assert!(house.contains(Position::new(0, 4)));
        assert!(house.contains(Position::new(3, 3)));
        assert!(!house.contains(Position::new(0, 0)));
    }

    #[test]
    fn test_house_peers() {
        let pos = Position::new(4, 4);
        let peers = pos.house_peers();

        assert!(!peers.contains(pos));
        assert_eq!(peers.len(), 20);
        assert!(peers.contains(Position::new(4, 0)));
        assert!(peers.contains(Position::new(0, 4)));
        assert!(peers.contains(Position::new(3, 3)));
        assert!(!peers.contains(Position::new(0, 0)));
    }

    #[test]
    fn test_ord_matches_index_order() {
        use crate::index::{Index81Semantics, PositionSemantics};

        // Position ordering matches index order (y * 9 + x)
        let pos_1_0 = Position::new(1, 0);
        let pos_0_1 = Position::new(0, 1);
        assert!(pos_1_0 < pos_0_1);
        assert_eq!(PositionSemantics::to_index(pos_1_0).index(), 1);
        assert_eq!(PositionSemantics::to_index(pos_0_1).index(), 9);

        assert!(Position::new(8, 0) < Position::new(0, 1));
        assert!(Position::new(8, 8) > Position::new(0, 0));
        assert_eq!(PositionSemantics::to_index(Position::new(8, 8)).index(), 80);

        // Position::ALL ordering matches PositionSemantics
        for (expected_index, &pos) in (0u8..).zip(Position::ALL.iter()) {
            assert_eq!(PositionSemantics::to_index(pos).index(), expected_index);
            assert_eq!(pos.x(), expected_index % 9);
            assert_eq!(pos.y(), expected_index / 9);
        }
    }

    #[test]
    fn test_const_arrays() {
        use std::collections::HashSet;

        // Each row/column/box contains exactly 9 unique positions
        for y in 0..9 {
            assert_eq!(Position::ROWS[y].iter().collect::<HashSet<_>>().len(), 9);
            for (x, &pos) in (0u8..).zip(Position::ROWS[y].iter()) {
                assert_eq!(pos, Position::new(x, y));
            }
        }

        for x in 0..9 {
            assert_eq!(Position::COLUMNS[x].iter().collect::<HashSet<_>>().len(), 9);
            for (y, &pos) in (0u8..).zip(Position::COLUMNS[x].iter()) {
                assert_eq!(pos, Position::new(x, y));
            }
        }

        for box_index in 0..9 {
            assert_eq!(
                Position::BOXES[box_index]
                    .iter()
                    .collect::<HashSet<_>>()
                    .len(),
                9
            );
            for (cell_index, &pos) in (0u8..).zip(Position::BOXES[box_index].iter()) {
                assert_eq!(pos, Position::from_box(box_index, cell_index));
            }
        }

        // Position::ALL contains exactly 81 unique positions
        assert_eq!(Position::ALL.len(), 81);
        assert_eq!(Position::ALL.iter().collect::<HashSet<_>>().len(), 81);
    }
}
