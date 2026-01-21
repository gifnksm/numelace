use std::{
    fmt::{self, Display},
    ops::{Index, IndexMut},
    slice,
    str::FromStr,
};

use crate::{CandidateGrid, Digit, Position, containers::Array81, index::PositionSemantics};

/// A simple cell-centric grid for storing Sudoku digits.
///
/// `DigitGrid` provides an intuitive interface for storing and accessing digits
/// in a Sudoku puzzle. Each cell can either contain a digit (1-9) or be empty.
///
/// # Examples
///
/// ```
/// use sudoku_core::{DigitGrid, Digit, Position};
///
/// let mut grid = DigitGrid::new();
/// grid.set(Position::new(0, 0), Some(Digit::D5));
/// assert_eq!(grid.get(Position::new(0, 0)), Some(Digit::D5));
/// ```
///
/// # String Parsing
///
/// `DigitGrid` supports parsing from strings for easy puzzle input:
///
/// ```
/// use sudoku_core::DigitGrid;
///
/// let grid: DigitGrid = "123456789........................................................................".parse().unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigitGrid {
    cells: Array81<Option<Digit>, PositionSemantics>,
}

impl Default for DigitGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl DigitGrid {
    /// Creates a new empty grid with all cells set to `None`.
    #[must_use]
    pub fn new() -> Self {
        Self::from_array([None; 81])
    }

    /// Creates a grid from an array of 81 cells.
    ///
    /// Cells are ordered row by row, left to right, top to bottom.
    #[must_use]
    pub fn from_array(cells: [Option<Digit>; 81]) -> Self {
        Self {
            cells: Array81::from(cells),
        }
    }

    /// Returns the digit at the given position, or `None` if the cell is empty.
    #[must_use]
    pub fn get(&self, pos: Position) -> Option<Digit> {
        self.cells[pos]
    }

    /// Sets the digit at the given position.
    ///
    /// Use `None` to clear the cell.
    pub fn set(&mut self, pos: Position, digit: Option<Digit>) {
        self.cells[pos] = digit;
    }

    /// Clears the cell at the given position (sets it to `None`).
    pub fn clear(&mut self, pos: Position) {
        self.cells[pos] = None;
    }

    /// Returns `true` if the cell at the given position is empty.
    #[must_use]
    pub fn is_empty(&self, pos: Position) -> bool {
        self.cells[pos].is_none()
    }

    /// Returns an iterator over all cells in the grid.
    ///
    /// Cells are iterated in row-major order (left to right, top to bottom).
    pub fn iter(&self) -> slice::Iter<'_, Option<Digit>> {
        self.cells.iter()
    }

    /// Returns a mutable iterator over all cells in the grid.
    ///
    /// Cells are iterated in row-major order (left to right, top to bottom).
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, Option<Digit>> {
        self.cells.iter_mut()
    }
}

impl<'a> IntoIterator for &'a DigitGrid {
    type Item = &'a Option<Digit>;
    type IntoIter = slice::Iter<'a, Option<Digit>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}

impl<'a> IntoIterator for &'a mut DigitGrid {
    type Item = &'a mut Option<Digit>;
    type IntoIter = slice::IterMut<'a, Option<Digit>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter_mut()
    }
}

impl From<DigitGrid> for CandidateGrid {
    fn from(digit_grid: DigitGrid) -> Self {
        CandidateGrid::from_digit_grid(&digit_grid)
    }
}

impl Index<Position> for DigitGrid {
    type Output = Option<Digit>;
    fn index(&self, pos: Position) -> &Self::Output {
        &self.cells[pos]
    }
}

impl IndexMut<Position> for DigitGrid {
    fn index_mut(&mut self, pos: Position) -> &mut Self::Output {
        &mut self.cells[pos]
    }
}

impl Display for DigitGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..9 {
            for x in 0..9 {
                let pos = Position::new(x, y);
                if let Some(digit) = self.get(pos) {
                    write!(f, "{digit}")?;
                } else {
                    write!(f, ".")?;
                }
            }
            if f.alternate() {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl FromStr for DigitGrid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cells = [None; 81];
        let mut cells_iter = cells.iter_mut();
        let mut chars = s.chars().filter(|c| !c.is_whitespace());

        // Parse characters and assign to cells
        for (cell, ch) in cells_iter.by_ref().zip(chars.by_ref()) {
            *cell = match ch {
                '.' | '0' | '_' => None,
                '1' => Some(Digit::D1),
                '2' => Some(Digit::D2),
                '3' => Some(Digit::D3),
                '4' => Some(Digit::D4),
                '5' => Some(Digit::D5),
                '6' => Some(Digit::D6),
                '7' => Some(Digit::D7),
                '8' => Some(Digit::D8),
                '9' => Some(Digit::D9),
                _ => return Err(format!("Invalid character '{ch}'")),
            };
        }

        // Check if there are too many characters
        if chars.next().is_some() {
            return Err("Invalid grid length: expected 81 characters, got more than 81".to_owned());
        }

        // Check if there are too few characters
        if cells_iter.next().is_some() {
            return Err(
                "Invalid grid length: expected 81 characters, got fewer than 81".to_owned(),
            );
        }

        Ok(Self::from_array(cells))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_empty_grid() {
        let s = ".................................................................................";
        let grid: DigitGrid = s.parse().unwrap();

        for y in 0..9 {
            for x in 0..9 {
                assert_eq!(grid.get(Position::new(x, y)), None);
            }
        }
    }

    #[test]
    fn test_from_str_with_digits() {
        let s = format!("123456789{}", ".".repeat(72));
        let grid: DigitGrid = s.parse().unwrap();

        assert_eq!(grid.get(Position::new(0, 0)), Some(Digit::D1));
        assert_eq!(grid.get(Position::new(1, 0)), Some(Digit::D2));
        assert_eq!(grid.get(Position::new(8, 0)), Some(Digit::D9));
        assert_eq!(grid.get(Position::new(0, 1)), None);
    }

    #[test]
    fn test_from_str_with_newlines() {
        let s = "123456789\n\
                 .........\n\
                 .........\n\
                 .........\n\
                 .........\n\
                 .........\n\
                 .........\n\
                 .........\n\
                 .........";
        let grid: DigitGrid = s.parse().unwrap();

        assert_eq!(grid.get(Position::new(0, 0)), Some(Digit::D1));
        assert_eq!(grid.get(Position::new(8, 0)), Some(Digit::D9));
        assert_eq!(grid.get(Position::new(0, 1)), None);
    }

    #[test]
    fn test_from_str_with_zeros() {
        let s = format!("000000000{}", "0".repeat(72));
        let grid: DigitGrid = s.parse().unwrap();

        for y in 0..9 {
            for x in 0..9 {
                assert_eq!(grid.get(Position::new(x, y)), None);
            }
        }
    }

    #[test]
    fn test_from_str_with_underscores() {
        let s = format!("_________{}", "_".repeat(72));
        let grid: DigitGrid = s.parse().unwrap();

        for y in 0..9 {
            for x in 0..9 {
                assert_eq!(grid.get(Position::new(x, y)), None);
            }
        }
    }

    #[test]
    fn test_from_str_invalid_length() {
        let s = "123456789";
        let result: Result<DigitGrid, _> = s.parse();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected 81"));
    }

    #[test]
    fn test_from_str_invalid_character() {
        let s = format!("X{}", ".".repeat(80));
        let result: Result<DigitGrid, _> = s.parse();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid character"));
    }

    #[test]
    fn test_display_roundtrip() {
        let original = format!("123456789{}", ".".repeat(72));
        let grid: DigitGrid = original.parse().unwrap();
        let displayed = grid.to_string();
        let reparsed: DigitGrid = displayed.parse().unwrap();

        for y in 0..9 {
            for x in 0..9 {
                let pos = Position::new(x, y);
                assert_eq!(grid.get(pos), reparsed.get(pos));
            }
        }
    }

    #[test]
    fn test_display_alternate_roundtrip() {
        let original = format!("123456789{}", ".".repeat(72));
        let grid: DigitGrid = original.parse().unwrap();

        // Format with alternate (pretty) format
        let displayed = format!("{grid:#}");

        // Should have 9 lines with newlines
        assert_eq!(displayed.lines().count(), 9);

        // Parse it back - whitespace should be ignored
        let reparsed: DigitGrid = displayed.parse().unwrap();

        // Verify all cells match
        for y in 0..9 {
            for x in 0..9 {
                let pos = Position::new(x, y);
                assert_eq!(grid.get(pos), reparsed.get(pos));
            }
        }
    }
}
