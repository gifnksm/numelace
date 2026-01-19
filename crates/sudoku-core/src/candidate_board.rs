//! Candidate bitboard for sudoku solving.
//!
//! This module provides [`CandidateBoard`], which tracks possible placements
//! for each digit (1-9) across the entire 9x9 board using bitboards.
//!
//! # Type Aliases
//!
//! - [`DigitPositions`] - A [`BitSet81`] tracking positions where a digit can be placed
//! - [`HouseMask`] - A [`BitSet9`] for candidates within a house (row/col/box)
//!
//! # Semantics
//!
//! This module uses semantics implementations defined in the index modules:
//!
//! - [`PositionSemantics`] - Maps [`Position`] to indices (from [`index`])
//! - [`CellIndexSemantics`] - Direct 0-8 mapping (from [`index`])
//!
//! [`index`]: crate::index
//!
//! # Examples
//!
//! ```
//! use sudoku_core::{CandidateBoard, Position};
//!
//! let mut board = CandidateBoard::new();
//!
//! // Place digit 5 at position (4, 4)
//! board.place(Position::new(4, 4), 5);
//!
//! // Check remaining candidates at a position
//! let candidates = board.get_candidates_at(Position::new(4, 5));
//! assert!(!candidates.contains(5)); // 5 was removed from the column
//!
//! // Check for Hidden Single in a row
//! let row_mask = board.get_row(4, 3);
//! if row_mask.len() == 1 {
//!     println!("Found Hidden Single for digit 3 in row 4");
//! }
//! ```

use crate::{
    containers::{Array9, BitSet9, BitSet81},
    digit_candidates::DigitCandidates,
    index::PositionSemantics,
    index::{CellIndexSemantics, DigitSemantics, Index9, Index9Semantics},
    position::Position,
};

/// A set of candidate positions across the board for a single digit.
///
/// This is a type alias for `BitSet81<PositionSemantics>`, representing
/// "which cells can this digit go in?" across the entire 9x9 board.
///
/// # Examples
///
/// ```
/// use sudoku_core::Position;
/// use sudoku_core::candidate_board::DigitPositions;
///
/// let mut positions = DigitPositions::FULL; // All 81 positions initially
/// positions.remove(Position::new(0, 0));
/// positions.remove(Position::new(4, 4));
///
/// assert_eq!(positions.len(), 79);
/// assert!(!positions.contains(Position::new(0, 0)));
/// ```
pub type DigitPositions = BitSet81<PositionSemantics>;

/// A bitmask representing candidate positions within a house (row/col/box).
///
/// This is a type alias for `BitSet9<CellIndexSemantics>`, where each bit
/// represents one of the 9 cells in a house. "House" is a sudoku term for
/// any row, column, or box.
///
/// Used by [`CandidateBoard::get_row`], [`CandidateBoard::get_col`], and
/// [`CandidateBoard::get_box`] to return candidate positions within a specific house.
///
/// # Examples
///
/// ```
/// use sudoku_core::candidate_board::HouseMask;
///
/// let mut mask = HouseMask::new();
/// mask.insert(0); // First cell in house
/// mask.insert(4); // Middle cell
/// mask.insert(8); // Last cell
///
/// assert_eq!(mask.len(), 3);
///
/// // Useful for detecting Hidden Singles
/// if mask.len() == 1 {
///     println!("Found a Hidden Single!");
/// }
/// ```
pub type HouseMask = BitSet9<CellIndexSemantics>;

/// Candidate bitboard for sudoku solving.
///
/// Manages possible placements for each digit (1-9) across the entire board.
/// Used for detecting Hidden Singles, Naked Singles, and other solving techniques.
///
/// # Structure
///
/// Internally stores 9 [`DigitPositions`] (one per digit), each tracking the
/// 81 board positions where that digit can be placed.
///
/// # Examples
///
/// ```
/// use sudoku_core::{CandidateBoard, Position};
///
/// let mut board = CandidateBoard::new();
///
/// // Initially all positions have all candidates
/// let pos = Position::new(0, 0);
/// assert_eq!(board.get_candidates_at(pos).len(), 9);
///
/// // Place digit 1 at (0, 0) - removes candidates from row, col, box
/// board.place(pos, 1);
///
/// // Now (0, 0) only has digit 1
/// let candidates = board.get_candidates_at(pos);
/// assert_eq!(candidates.len(), 1);
/// assert!(candidates.contains(1));
///
/// // Other cells in the row no longer have digit 1 as candidate
/// let row_mask = board.get_row(0, 1);
/// assert_eq!(row_mask.len(), 1); // Only at (0, 0)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateBoard {
    /// `digits[i]` represents possible positions for digit `(i+1)`
    digits: Array9<DigitPositions, DigitSemantics>,
}

impl Default for CandidateBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl CandidateBoard {
    /// Creates a new candidate board with all positions available for all digits.
    #[must_use]
    pub fn new() -> Self {
        Self {
            digits: Array9::from([DigitPositions::FULL; 9]),
        }
    }

    /// Places a digit at a position and updates candidates accordingly.
    ///
    /// This removes all candidates at the position, removes the digit from
    /// the same row, column, and box, then marks the position as containing
    /// the placed digit.
    pub fn place(&mut self, pos: Position, digit: u8) {
        // remove all digits at pos
        for digits in &mut self.digits {
            digits.remove(pos);
        }

        let digits = &mut self.digits[digit];
        for x in 0..9 {
            digits.remove(Position::new(x, pos.y()));
        }
        for y in 0..9 {
            digits.remove(Position::new(pos.x(), y));
        }
        let box_index = pos.box_index();
        for i in 0..9 {
            digits.remove(Position::from_box(box_index, i));
        }
        digits.insert(pos);
    }

    /// Removes a specific digit as a candidate at a position.
    pub fn remove_candidate(&mut self, pos: Position, digit: u8) {
        let digits = &mut self.digits[digit];
        digits.remove(pos);
    }

    /// Returns the set of candidate digits that can be placed at a position.
    #[must_use]
    pub fn get_candidates_at(&self, pos: Position) -> DigitCandidates {
        let mut candidates = DigitCandidates::new();
        for (i, digits) in (0..).zip(&self.digits) {
            if digits.contains(pos) {
                candidates.insert(DigitSemantics::from_index(Index9::new(i)));
            }
        }
        candidates
    }

    /// Returns positions in the specified row where the digit can be placed.
    ///
    /// If the returned mask has only one bit set, a Hidden Single is detected.
    #[must_use]
    pub fn get_row(&self, y: u8, digit: u8) -> HouseMask {
        let digits = &self.digits[digit];

        let mut mask = HouseMask::new();
        for x in 0..9 {
            if digits.contains(Position::new(x, y)) {
                mask.insert(x);
            }
        }
        mask
    }

    /// Returns positions in the specified column where the digit can be placed.
    ///
    /// If the returned mask has only one bit set, a Hidden Single is detected.
    #[must_use]
    pub fn get_col(&self, x: u8, digit: u8) -> HouseMask {
        let digits = &self.digits[digit];

        let mut mask = HouseMask::new();
        for y in 0..9 {
            if digits.contains(Position::new(x, y)) {
                mask.insert(y);
            }
        }
        mask
    }

    /// Returns positions in the specified box where the digit can be placed.
    ///
    /// If the returned mask has only one bit set, a Hidden Single is detected.
    #[must_use]
    pub fn get_box(&self, box_index: u8, digit: u8) -> HouseMask {
        let digits = &self.digits[digit];

        let mut mask = HouseMask::new();
        for i in 0..9 {
            if digits.contains(Position::from_box(box_index, i)) {
                mask.insert(i);
            }
        }
        mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_board_has_all_candidates() {
        let board = CandidateBoard::new();

        // All positions should have all 9 digits as candidates initially
        for y in 0..9 {
            for x in 0..9 {
                let pos = Position::new(x, y);
                let candidates = board.get_candidates_at(pos);
                assert_eq!(candidates.len(), 9);
                for digit in 1..=9 {
                    assert!(candidates.contains(digit));
                }
            }
        }
    }

    #[test]
    fn test_place_digit() {
        let mut board = CandidateBoard::new();

        // Manually set up some candidates
        let pos = Position::new(4, 4); // center
        for digit in &mut board.digits {
            digit.insert(pos);
        }

        // Place digit 5 at center
        board.place(pos, 5);

        // The position should only have digit 5
        let candidates = board.get_candidates_at(pos);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(5));
    }

    #[test]
    fn test_place_removes_row_candidates() {
        let mut board = CandidateBoard::new();

        // Set digit 5 as candidate for entire row 0
        for x in 0..9 {
            board.digits[4].insert(Position::new(x, 0));
        }

        // Place digit 5 at (0, 0)
        board.place(Position::new(0, 0), 5);

        // Digit 5 should be removed from rest of row 0
        for x in 1..9 {
            let row_mask = board.get_row(0, 5);
            assert!(
                !row_mask.contains(x),
                "Position ({x}, 0) should not have digit 5"
            );
        }

        // But (0, 0) should still have it
        assert!(board.get_candidates_at(Position::new(0, 0)).contains(5));
    }

    #[test]
    fn test_place_removes_column_candidates() {
        let mut board = CandidateBoard::new();

        // Set digit 3 as candidate for entire column 5
        for y in 0..9 {
            board.digits[2].insert(Position::new(5, y));
        }

        // Place digit 3 at (5, 3)
        board.place(Position::new(5, 3), 3);

        // Digit 3 should be removed from rest of column 5
        for y in 0..9 {
            if y == 3 {
                continue;
            }
            let col_mask = board.get_col(5, 3);
            assert!(
                !col_mask.contains(y),
                "Position (5, {y}) should not have digit 3"
            );
        }
    }

    #[test]
    fn test_place_removes_box_candidates() {
        let mut board = CandidateBoard::new();

        // Set digit 7 as candidate for entire box 4 (center box)
        for i in 0..9 {
            board.digits[6].insert(Position::from_box(4, i));
        }

        // Place digit 7 at center of center box
        board.place(Position::new(4, 4), 7);

        // Digit 7 should be removed from rest of box 4
        let box_mask = board.get_box(4, 7);
        assert_eq!(box_mask.len(), 1, "Only one position should remain in box");
        assert!(box_mask.contains(4), "Center cell should remain");
    }

    #[test]
    fn test_place_removes_all_candidates_at_position() {
        let mut board = CandidateBoard::new();

        let pos = Position::new(2, 2);

        // Add all digits as candidates at position
        for digit in &mut board.digits {
            digit.insert(pos);
        }

        // Place digit 1 there
        board.place(pos, 1);

        // Only digit 1 should remain
        let candidates = board.get_candidates_at(pos);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(1));
    }

    #[test]
    fn test_remove_candidate() {
        let mut board = CandidateBoard::new();

        let pos = Position::new(3, 3);

        // Initially has all 9 candidates, remove digit 5
        board.remove_candidate(pos, 5);

        let candidates = board.get_candidates_at(pos);
        assert_eq!(candidates.len(), 8);
        assert!(!candidates.contains(5));
        for digit in 1..=9 {
            if digit != 5 {
                assert!(candidates.contains(digit));
            }
        }
    }

    #[test]
    fn test_get_candidates_at_full_position() {
        let board = CandidateBoard::new();
        let candidates = board.get_candidates_at(Position::new(0, 0));
        assert_eq!(candidates.len(), 9);
    }

    #[test]
    fn test_get_candidates_at_with_removed_digits() {
        let mut board = CandidateBoard::new();
        let pos = Position::new(5, 5);

        // Remove digits 1, 3, 5, 7, 9 (keep 2, 4, 6, 8)
        for digit in [1, 3, 5, 7, 9] {
            board.remove_candidate(pos, digit);
        }

        let candidates = board.get_candidates_at(pos);
        assert_eq!(candidates.len(), 4);
        assert!(candidates.contains(2));
        assert!(candidates.contains(4));
        assert!(candidates.contains(6));
        assert!(candidates.contains(8));
    }

    #[test]
    fn test_get_row_full() {
        let board = CandidateBoard::new();
        let mask = board.get_row(0, 5);
        assert_eq!(mask.len(), 9);
    }

    #[test]
    fn test_get_row_with_candidates() {
        let mut board = CandidateBoard::new();

        // Remove digit 3 from all positions in row 2 except (1, 2), (3, 2), (5, 2)
        for x in 0..9 {
            if x != 1 && x != 3 && x != 5 {
                board.remove_candidate(Position::new(x, 2), 3);
            }
        }

        let mask = board.get_row(2, 3);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(1));
        assert!(mask.contains(3));
        assert!(mask.contains(5));
    }

    #[test]
    fn test_get_col_full() {
        let board = CandidateBoard::new();
        let mask = board.get_col(3, 7);
        assert_eq!(mask.len(), 9);
    }

    #[test]
    fn test_get_col_with_candidates() {
        let mut board = CandidateBoard::new();

        // Remove digit 9 from all positions in column 4 except (4, 0), (4, 4), (4, 8)
        for y in 0..9 {
            if y != 0 && y != 4 && y != 8 {
                board.remove_candidate(Position::new(4, y), 9);
            }
        }

        let mask = board.get_col(4, 9);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(0));
        assert!(mask.contains(4));
        assert!(mask.contains(8));
    }

    #[test]
    fn test_get_box_full() {
        let board = CandidateBoard::new();
        let mask = board.get_box(0, 1);
        assert_eq!(mask.len(), 9);
    }

    #[test]
    fn test_get_box_with_candidates() {
        let mut board = CandidateBoard::new();

        // Remove digit 6 from all positions in box 8 except cells 0, 4, 8
        for i in 0..9 {
            if i != 0 && i != 4 && i != 8 {
                board.remove_candidate(Position::from_box(8, i), 6);
            }
        }

        let mask = board.get_box(8, 6);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(0));
        assert!(mask.contains(4));
        assert!(mask.contains(8));
    }

    #[test]
    fn test_hidden_single_in_row() {
        let mut board = CandidateBoard::new();

        // Remove digit 4 from all positions in row 5 except position 7
        for x in 0..9 {
            if x != 7 {
                board.remove_candidate(Position::new(x, 5), 4);
            }
        }

        let mask = board.get_row(5, 4);
        assert_eq!(mask.len(), 1, "Hidden single detected: only one candidate");
        assert!(mask.contains(7));
    }

    #[test]
    fn test_hidden_single_in_column() {
        let mut board = CandidateBoard::new();

        // Remove digit 8 from all positions in column 2 except position 3
        for y in 0..9 {
            if y != 3 {
                board.remove_candidate(Position::new(2, y), 8);
            }
        }

        let mask = board.get_col(2, 8);
        assert_eq!(mask.len(), 1, "Hidden single detected: only one candidate");
        assert!(mask.contains(3));
    }

    #[test]
    fn test_hidden_single_in_box() {
        let mut board = CandidateBoard::new();

        // Remove digit 2 from all positions in box 1 except cell 5
        for i in 0..9 {
            if i != 5 {
                board.remove_candidate(Position::from_box(1, i), 2);
            }
        }

        let mask = board.get_box(1, 2);
        assert_eq!(mask.len(), 1, "Hidden single detected: only one candidate");
        assert!(mask.contains(5));
    }

    #[test]
    fn test_board_clone() {
        let mut board1 = CandidateBoard::new();
        board1.digits[1].insert(Position::new(0, 0));

        let board2 = board1.clone();

        assert_eq!(board1, board2);
    }

    #[test]
    fn test_board_default() {
        let board = CandidateBoard::default();

        // Default should be same as new() - all candidates available
        for y in 0..9 {
            for x in 0..9 {
                assert_eq!(board.get_candidates_at(Position::new(x, y)).len(), 9);
            }
        }
    }
}
