//! Candidate grid for sudoku solving.

use std::iter;

use crate::{
    containers::{Array9, BitSet9, BitSet81},
    digit::Digit,
    index::{CellIndexSemantics, DigitSemantics, Index9, Index9Semantics, PositionSemantics},
    position::Position,
};

/// Set of candidate digits (1-9) for a single cell.
///
/// Returned by [`CandidateGrid::get_candidates_at`].
pub type DigitCandidates = BitSet9<DigitSemantics>;

/// Set of grid positions where a specific digit can be placed.
///
/// Returned by [`CandidateGrid::get_positions`].
pub type DigitPositions = BitSet81<PositionSemantics>;

/// Bitmask of candidate positions within a house (row, column, or box).
///
/// Returned by [`CandidateGrid::get_row`], [`CandidateGrid::get_col`], and
/// [`CandidateGrid::get_box`]. Useful for detecting Hidden Singles (when `len() == 1`).
pub type HouseMask = BitSet9<CellIndexSemantics>;

/// Candidate grid for sudoku solving.
///
/// Manages possible placements for each digit (1-9) across the entire 9x9 grid.
/// Internally stores 9 [`DigitPositions`] (one per digit), each tracking the 81 grid
/// positions where that digit can be placed.
///
/// Used for detecting Hidden Singles, Naked Singles, and other solving techniques.
///
/// # Examples
///
/// ```
/// use sudoku_core::{CandidateGrid, Digit, Position};
///
/// let mut grid = CandidateGrid::new();
///
/// // Initially all positions have all candidates
/// let pos = Position::new(0, 0);
/// assert_eq!(grid.get_candidates_at(pos).len(), 9);
///
/// // Place digit 1 at (0, 0) - removes candidates from row, col, box
/// grid.place(pos, Digit::D1);
///
/// // Now (0, 0) only has digit 1
/// let candidates = grid.get_candidates_at(pos);
/// assert_eq!(candidates.len(), 1);
/// assert!(candidates.contains(Digit::D1));
///
/// // Other cells in the row no longer have digit 1 as candidate
/// let row_mask = grid.get_row(0, Digit::D1);
/// assert_eq!(row_mask.len(), 1); // Only at (0, 0)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGrid {
    /// `digits[i]` represents possible positions for digit `(i+1)`
    digits: Array9<DigitPositions, DigitSemantics>,
}

impl Default for CandidateGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl CandidateGrid {
    /// Creates a new candidate grid with all positions available for all digits.
    #[must_use]
    pub fn new() -> Self {
        Self {
            digits: Array9::from([DigitPositions::FULL; 9]),
        }
    }

    /// Places a digit at a position and updates candidates accordingly.
    ///
    /// This removes all other digit candidates at the position, removes the digit
    /// from the same row, column, and box, then marks the position as containing
    /// the placed digit.
    ///
    /// # Returns
    ///
    /// Returns `true` if any candidates were changed, `false` if the operation
    /// had no effect (e.g., placing the same digit again at an already decided position).
    ///
    /// # Example
    ///
    /// ```
    /// use sudoku_core::{CandidateGrid, Digit, Position};
    ///
    /// let mut grid = CandidateGrid::new();
    /// let changed = grid.place(Position::new(0, 0), Digit::D5);
    /// assert!(changed); // First placement changes the grid
    ///
    /// let changed = grid.place(Position::new(0, 0), Digit::D5);
    /// assert!(!changed); // Placing again has no effect
    /// ```
    pub fn place(&mut self, pos: Position, digit: Digit) -> bool {
        let mut changed = false;

        // remove all digits around the pos
        for (d, digits) in iter::zip(Digit::ALL, &mut self.digits) {
            if d != digit {
                changed |= digits.remove(pos);
            }
        }

        let digits = &mut self.digits[digit];
        for x in 0..9 {
            if x != pos.x() {
                changed |= digits.remove(Position::new(x, pos.y()));
            }
        }
        for y in 0..9 {
            if y != pos.y() {
                changed |= digits.remove(Position::new(pos.x(), y));
            }
        }
        let box_index = pos.box_index();
        let cell_index = pos.box_cell_index();
        for i in 0..9 {
            if i != cell_index {
                changed |= digits.remove(Position::from_box(box_index, i));
            }
        }
        changed |= digits.insert(pos);
        changed
    }

    /// Removes a specific digit as a candidate at a position.
    ///
    /// # Returns
    ///
    /// Returns `true` if the candidate was removed, `false` if the digit was
    /// not a candidate at that position.
    ///
    /// # Example
    ///
    /// ```
    /// use sudoku_core::{CandidateGrid, Digit, Position};
    ///
    /// let mut grid = CandidateGrid::new();
    /// let changed = grid.remove_candidate(Position::new(0, 0), Digit::D1);
    /// assert!(changed); // D1 was a candidate
    ///
    /// let changed = grid.remove_candidate(Position::new(0, 0), Digit::D1);
    /// assert!(!changed); // D1 is already removed
    /// ```
    pub fn remove_candidate(&mut self, pos: Position, digit: Digit) -> bool {
        let digits = &mut self.digits[digit];
        digits.remove(pos)
    }

    /// Returns all positions where the specified digit can be placed.
    #[must_use]
    pub fn get_positions(&self, digit: Digit) -> DigitPositions {
        self.digits[digit]
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
    pub fn get_row(&self, y: u8, digit: Digit) -> HouseMask {
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
    pub fn get_col(&self, x: u8, digit: Digit) -> HouseMask {
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
    pub fn get_box(&self, box_index: u8, digit: Digit) -> HouseMask {
        let digits = &self.digits[digit];

        let mut mask = HouseMask::new();
        for i in 0..9 {
            if digits.contains(Position::from_box(box_index, i)) {
                mask.insert(i);
            }
        }
        mask
    }

    /// Checks if the grid is **consistent** (no contradictions).
    ///
    /// Returns `true` if:
    ///
    /// - Every position has at least one candidate
    /// - No duplicate definite digits in any row, column, or box
    ///
    /// Unlike [`is_solved`], this does NOT require all cells to be decided.
    /// It can be used during solving to detect contradictions early.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::{CandidateGrid, Digit, Position};
    ///
    /// let mut grid = CandidateGrid::new();
    /// assert!(grid.is_consistent());
    ///
    /// grid.place(Position::new(0, 0), Digit::D5);
    /// assert!(grid.is_consistent()); // Still consistent after placing
    /// ```
    ///
    /// [`is_solved`]: CandidateGrid::is_solved
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        let [empty_cells, decided_cells] = self.classify_cells();
        empty_cells.is_empty() && self.placed_digits_are_unique(decided_cells)
    }

    /// Checks if the puzzle is **solved** (complete and consistent).
    ///
    /// A grid is solved if:
    ///
    /// - All 81 positions have exactly one candidate (complete)
    /// - No position has zero candidates (no contradictions)
    /// - All definite digits satisfy sudoku uniqueness constraints (no duplicates)
    ///
    /// This is equivalent to `is_complete() && is_consistent()`, but more efficient
    /// as it only computes the cell classification once.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::CandidateGrid;
    ///
    /// let grid = CandidateGrid::new();
    /// assert!(!grid.is_solved()); // Empty grid is not solved
    /// ```
    #[must_use]
    pub fn is_solved(&self) -> bool {
        let [empty_cells, decided_cells] = self.classify_cells();
        empty_cells.is_empty()
            && decided_cells.len() == 81
            && self.placed_digits_are_unique(decided_cells)
    }

    /// Returns all positions that have exactly one candidate (decided cells).
    ///
    /// A cell is considered "decided" when it has only one possible candidate digit,
    /// either because it was explicitly placed or because all other candidates were
    /// eliminated through constraint propagation.
    ///
    /// # Example
    ///
    /// ```
    /// use sudoku_core::{CandidateGrid, Digit, Position};
    ///
    /// let mut grid = CandidateGrid::new();
    ///
    /// // Initially no decided cells
    /// assert_eq!(grid.decided_cells().len(), 0);
    ///
    /// // Place a digit
    /// grid.place(Position::new(0, 0), Digit::D5);
    /// assert_eq!(grid.decided_cells().len(), 1);
    /// ```
    #[must_use]
    pub fn decided_cells(&self) -> DigitPositions {
        let [_empty_cells, decided_cells] = self.classify_cells();
        decided_cells
    }

    /// Classifies all grid positions by candidate count.
    /// Classifies all cells by their candidate count.
    ///
    /// Returns an array of length `N` where the element at index `i` contains
    /// the positions of all cells that have exactly `i` candidates.
    ///
    /// # Type Parameters
    ///
    /// * `N` - Number of candidate counts to track (typically 2 for basic solving,
    ///   or up to 10 for advanced analysis)
    ///
    /// # Returns
    ///
    /// An array `[cells_0, cells_1, ..., cells_N-1]` where:
    ///
    /// - `cells[0]`: Positions with zero candidates (contradictions)
    /// - `cells[1]`: Positions with exactly one candidate (decided cells)
    /// - `cells[2]`: Positions with exactly two candidates
    /// - ...
    /// - `cells[i]`: Positions with exactly `i` candidates
    ///
    /// Positions with `N` or more candidates are not included in any element.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::{CandidateGrid, Digit, Position};
    ///
    /// let mut grid = CandidateGrid::new();
    /// grid.place(Position::new(0, 0), Digit::D1);
    ///
    /// // Get empty and decided cells only
    /// let [empty_cells, decided_cells] = grid.classify_cells();
    /// assert_eq!(empty_cells.len(), 0);
    /// assert_eq!(decided_cells.len(), 1);
    ///
    /// // Get more detailed classification
    /// let [empty, one, two, three] = grid.classify_cells();
    /// // empty: 0 candidates, one: 1 candidate, two: 2 candidates, three: 3 candidates
    /// ```
    ///
    /// This method performs a single pass over all digits to efficiently compute
    /// all classifications simultaneously using bitwise operations.
    #[must_use]
    pub fn classify_cells<const N: usize>(&self) -> [DigitPositions; N] {
        let mut cells = [DigitPositions::EMPTY; N];
        cells[0] = DigitPositions::FULL;
        for (n, digit_pos) in iter::zip(1.., self.digits.iter().copied()) {
            let end = usize::min(n + 1, N);
            for i in (1..end).rev() {
                cells[i] &= !digit_pos;
                cells[i] |= cells[i - 1] & digit_pos;
            }
            cells[0] &= !digit_pos;
        }
        cells
    }

    /// Checks that definite digits have no duplicates in rows, columns, or boxes.
    ///
    /// For each position in `decided_cells`, verifies that its digit appears
    /// exactly once in its respective row, column, and 3×3 box.
    ///
    /// # Arguments
    ///
    /// * `decided_cells` - Positions where exactly one candidate remains
    ///
    /// # Returns
    ///
    /// `true` if all definite digits satisfy sudoku uniqueness constraints,
    /// `false` if any digit appears multiple times in the same row, column, or box.
    fn placed_digits_are_unique(&self, decided_cells: DigitPositions) -> bool {
        for digit in Digit::ALL {
            let digit_cells = &self.digits[digit];
            for pos in *digit_cells & decided_cells {
                if self.get_row(pos.y(), digit).len() != 1 {
                    return false;
                }
                if self.get_col(pos.x(), digit).len() != 1 {
                    return false;
                }
                if self.get_box(pos.box_index(), digit).len() != 1 {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use Digit::*;

    use super::*;

    #[test]
    fn test_new_grid_has_all_candidates() {
        let grid = CandidateGrid::new();

        // All positions should have all 9 digits as candidates initially
        for y in 0..9 {
            for x in 0..9 {
                let pos = Position::new(x, y);
                let candidates = grid.get_candidates_at(pos);
                assert_eq!(candidates.len(), 9);
                for digit in Digit::ALL {
                    assert!(candidates.contains(digit));
                }
            }
        }
    }

    #[test]
    fn test_place_digit() {
        let mut grid = CandidateGrid::new();

        // Manually set up some candidates
        let pos = Position::new(4, 4); // center
        for digit in &mut grid.digits {
            digit.insert(pos);
        }

        // Place digit 5 at center
        let changed = grid.place(pos, D5);
        assert!(changed, "Placing a digit should return true");

        // The position should only have digit 5
        let candidates = grid.get_candidates_at(pos);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(D5));
    }

    #[test]
    fn test_place_returns_false_when_no_change() {
        let mut grid = CandidateGrid::new();
        let pos = Position::new(0, 0);

        // First placement should return true
        let changed = grid.place(pos, D1);
        assert!(changed);

        // Placing again at the same position should return false
        // (no candidates are removed because it's already decided)
        let changed = grid.place(pos, D1);
        assert!(!changed, "Placing same digit again should return false");
    }

    #[test]
    fn test_place_removes_row_candidates() {
        let mut grid = CandidateGrid::new();

        // Set digit 5 as candidate for entire row 0
        for x in 0..9 {
            grid.digits[D4].insert(Position::new(x, 0));
        }

        // Place digit 5 at (0, 0)
        grid.place(Position::new(0, 0), D5);

        // Digit 5 should be removed from rest of row 0
        for x in 1..9 {
            let row_mask = grid.get_row(0, D5);
            assert!(
                !row_mask.contains(x),
                "Position ({x}, 0) should not have digit 5"
            );
        }

        // But (5, 3) should still have it
        assert!(grid.get_candidates_at(Position::new(5, 3)).contains(D3));
    }

    #[test]
    fn test_place_removes_column_candidates() {
        let mut grid = CandidateGrid::new();

        // Set digit 3 as candidate for entire column 5
        for y in 0..9 {
            grid.digits[D2].insert(Position::new(5, y));
        }

        // Place digit 3 at (5, 3)
        grid.place(Position::new(5, 3), D3);

        // Digit 3 should be removed from rest of column 5
        for y in 0..9 {
            if y == 3 {
                continue;
            }
            let col_mask = grid.get_col(5, D3);
            assert!(
                !col_mask.contains(y),
                "Position (5, {y}) should not have digit 3"
            );
        }
    }

    #[test]
    fn test_place_removes_box_candidates() {
        let mut grid = CandidateGrid::new();

        // Set digit 7 as candidate for entire box 4 (center box)
        for i in 0..9 {
            grid.digits[D6].insert(Position::from_box(4, i));
        }

        // Place digit 7 at center of center box
        grid.place(Position::new(4, 4), D7);

        // Digit 7 should only be at (4, 4) in box 4
        let box_mask = grid.get_box(4, D7);
        assert_eq!(box_mask.len(), 1, "Only one position should remain in box");
        assert!(box_mask.contains(4), "Center cell should remain");
    }

    #[test]
    fn test_place_removes_all_candidates_at_position() {
        let mut grid = CandidateGrid::new();

        let pos = Position::new(2, 2);

        // Add all digits as candidates at position
        for digit in &mut grid.digits {
            digit.insert(pos);
        }

        // Place digit 1 there
        grid.place(pos, D1);

        // Now only digit 1 should be there
        let candidates = grid.get_candidates_at(pos);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(D1));
    }

    #[test]
    fn test_remove_candidate() {
        let mut grid = CandidateGrid::new();

        let pos = Position::new(3, 3);

        // Initially has all 9 candidates, remove digit 5
        let changed = grid.remove_candidate(pos, D5);
        assert!(changed, "Removing a candidate should return true");

        let candidates = grid.get_candidates_at(pos);
        assert_eq!(candidates.len(), 8);
        assert!(!candidates.contains(D5));
        for digit in Digit::ALL {
            if digit != D5 {
                assert!(candidates.contains(digit));
            }
        }
    }

    #[test]
    fn test_remove_candidate_returns_false_when_not_present() {
        let mut grid = CandidateGrid::new();
        let pos = Position::new(0, 0);

        // Remove D1 first time - should return true
        let changed = grid.remove_candidate(pos, D1);
        assert!(changed);

        // Remove D1 again - should return false (already removed)
        let changed = grid.remove_candidate(pos, D1);
        assert!(
            !changed,
            "Removing already-removed candidate should return false"
        );
    }

    #[test]
    fn test_decided_cells() {
        let mut grid = CandidateGrid::new();

        // Initially no decided cells
        let decided = grid.decided_cells();
        assert_eq!(decided.len(), 0);

        // Place a digit
        grid.place(Position::new(0, 0), D1);
        let decided = grid.decided_cells();
        assert_eq!(decided.len(), 1);
        assert!(decided.contains(Position::new(0, 0)));

        // Place another digit
        grid.place(Position::new(5, 5), D5);
        let decided = grid.decided_cells();
        assert_eq!(decided.len(), 2);
        assert!(decided.contains(Position::new(0, 0)));
        assert!(decided.contains(Position::new(5, 5)));
    }

    #[test]
    fn test_decided_cells_after_manual_candidate_removal() {
        let mut grid = CandidateGrid::new();
        let pos = Position::new(3, 3);

        // Manually remove all candidates except D7
        for digit in Digit::ALL {
            if digit != D7 {
                grid.remove_candidate(pos, digit);
            }
        }

        // Position should be considered decided (only one candidate)
        let decided = grid.decided_cells();
        assert_eq!(decided.len(), 1);
        assert!(decided.contains(pos));
    }

    #[test]
    fn test_get_positions_full_grid() {
        let grid = CandidateGrid::new();

        // Initially all 81 positions are candidates for any digit
        for digit in Digit::ALL {
            let positions = grid.get_positions(digit);
            assert_eq!(positions.len(), 81);
        }
    }

    #[test]
    fn test_get_positions_after_placement() {
        let mut grid = CandidateGrid::new();

        // Place digit 5 at (4, 4)
        let pos = Position::new(4, 4);
        grid.place(pos, D5);

        let positions = grid.get_positions(D5);

        // D5 should be at the placed position
        assert!(positions.contains(pos));

        // D5 should be removed from same row, column, and box
        assert!(!positions.contains(Position::new(0, 4))); // Same row
        assert!(!positions.contains(Position::new(4, 0))); // Same column
        assert!(!positions.contains(Position::new(3, 3))); // Same box

        // But D5 can still be placed in other rows/columns/boxes
        assert!(positions.contains(Position::new(0, 0))); // Different row, column, and box

        // Other digits should be removed from the placed cell
        let positions_d1 = grid.get_positions(D1);
        assert!(!positions_d1.contains(pos)); // Cell itself removed
        assert!(positions_d1.contains(Position::new(0, 4))); // Same row is OK
        assert!(positions_d1.contains(Position::new(4, 0))); // Same column is OK
        assert!(positions_d1.contains(Position::new(3, 3))); // Same box is OK
    }

    #[test]
    fn test_get_candidates_at_full_position() {
        let grid = CandidateGrid::new();
        let candidates = grid.get_candidates_at(Position::new(4, 4));
        assert_eq!(candidates.len(), 9);
    }

    #[test]
    fn test_get_candidates_at_with_removed_digits() {
        let mut board = CandidateGrid::new();
        let pos = Position::new(5, 5);

        // Remove digits 1, 3, 5, 7, 9 (keep 2, 4, 6, 8)
        for digit in [D1, D3, D5, D7, D9] {
            board.remove_candidate(pos, digit);
        }

        let candidates = board.get_candidates_at(pos);
        assert_eq!(candidates.len(), 4);
        assert!(candidates.contains(D2));
        assert!(candidates.contains(D4));
        assert!(candidates.contains(D6));
        assert!(candidates.contains(D8));
    }

    #[test]
    fn test_get_row_full() {
        let grid = CandidateGrid::new();
        let mask = grid.get_row(0, D1);
        assert_eq!(mask.len(), 9);
    }

    #[test]
    fn test_get_row_with_candidates() {
        let mut board = CandidateGrid::new();

        // Remove digit 3 from all positions in row 2 except (1, 2), (3, 2), (5, 2)
        for x in 0..9 {
            if x != 1 && x != 3 && x != 5 {
                board.remove_candidate(Position::new(x, 2), D3);
            }
        }

        let mask = board.get_row(2, D3);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(1));
        assert!(mask.contains(3));
        assert!(mask.contains(5));
    }

    #[test]
    fn test_get_col_full() {
        let grid = CandidateGrid::new();
        let mask = grid.get_col(0, D1);
        assert_eq!(mask.len(), 9);
    }

    #[test]
    fn test_get_col_with_candidates() {
        let mut board = CandidateGrid::new();

        // Remove digit 9 from all positions in column 4 except (4, 0), (4, 4), (4, 8)
        for y in 0..9 {
            if y != 0 && y != 4 && y != 8 {
                board.remove_candidate(Position::new(4, y), D9);
            }
        }

        let mask = board.get_col(4, D9);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(0));
        assert!(mask.contains(4));
        assert!(mask.contains(8));
    }

    #[test]
    fn test_get_box_full() {
        let grid = CandidateGrid::new();
        let mask = grid.get_box(4, D1);
        assert_eq!(mask.len(), 9);
    }

    #[test]
    fn test_get_box_with_candidates() {
        let mut board = CandidateGrid::new();

        // Remove digit 6 from all positions in box 8 except cells 0, 4, 8
        for i in 0..9 {
            if i != 0 && i != 4 && i != 8 {
                board.remove_candidate(Position::from_box(8, i), D6);
            }
        }

        let mask = board.get_box(8, D6);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(0));
        assert!(mask.contains(4));
        assert!(mask.contains(8));
    }

    #[test]
    fn test_hidden_single_in_row() {
        let mut grid = CandidateGrid::new();

        // Set up: digit 5 can only go in position (3, 0) in row 0
        for x in 0..9 {
            if x != 3 {
                grid.remove_candidate(Position::new(x, 0), D5);
            }
        }

        let mask = grid.get_row(0, D5);
        assert_eq!(mask.len(), 1, "Hidden single detected: only one candidate");
        assert!(mask.contains(3)); // x=3 is the only position
    }

    #[test]
    fn test_hidden_single_in_column() {
        let mut grid = CandidateGrid::new();

        // Set up: digit 7 can only go in position (5, 4) in column 5
        for y in 0..9 {
            if y != 4 {
                grid.remove_candidate(Position::new(5, y), D7);
            }
        }

        let mask = grid.get_col(5, D7);
        assert_eq!(mask.len(), 1, "Hidden single detected: only one candidate");
        assert!(mask.contains(4)); // y=4 is the only position
    }

    #[test]
    fn test_hidden_single_in_box() {
        let mut grid = CandidateGrid::new();

        // Set up: digit 9 can only go in position (4, 4) (center of box 4)
        for i in 0..9 {
            if i != 4 {
                grid.remove_candidate(Position::from_box(4, i), D9);
            }
        }

        let mask = grid.get_box(4, D9);
        assert_eq!(mask.len(), 1, "Hidden single detected: only one candidate");
        assert!(mask.contains(4)); // cell_index=4 is the center of the box
    }

    #[test]
    fn test_board_clone() {
        let mut board1 = CandidateGrid::new();
        board1.digits[D1].insert(Position::new(0, 0));

        let board2 = board1.clone();

        assert_eq!(board1, board2);
    }

    #[test]
    fn test_board_default() {
        let board = CandidateGrid::default();

        // Default should be same as new() - all candidates available
        for y in 0..9 {
            for x in 0..9 {
                assert_eq!(board.get_candidates_at(Position::new(x, y)).len(), 9);
            }
        }
    }

    #[test]
    fn test_is_consistent_empty_grid() {
        let grid = CandidateGrid::new();
        assert!(grid.is_consistent());
    }

    #[test]
    fn test_is_consistent_after_single_placement() {
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D5);
        assert!(grid.is_consistent());
    }

    #[test]
    fn test_is_consistent_after_multiple_placements() {
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D1);
        grid.place(Position::new(1, 1), D2);
        grid.place(Position::new(2, 2), D3);
        assert!(grid.is_consistent());
    }

    #[test]
    fn test_is_consistent_detects_empty_cell() {
        let mut grid = CandidateGrid::new();
        let pos = Position::new(4, 4);
        // Remove all candidates from a position
        for digit in Digit::ALL {
            grid.remove_candidate(pos, digit);
        }
        assert!(!grid.is_consistent());
    }

    #[test]
    fn test_is_solved_empty_grid() {
        let grid = CandidateGrid::new();
        assert!(!grid.is_solved());
    }

    #[test]
    fn test_is_solved_partially_filled() {
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D1);
        grid.place(Position::new(1, 1), D2);
        assert!(!grid.is_solved());
    }

    #[test]
    fn test_classify_cells_empty_grid() {
        let grid = CandidateGrid::new();
        let [empty, decided] = grid.classify_cells();

        // No empty cells
        assert_eq!(empty.len(), 0);
        // No decided cells
        assert_eq!(decided.len(), 0);
    }

    #[test]
    fn test_classify_cells_after_placement() {
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D5);

        let [empty, decided] = grid.classify_cells();

        // No empty cells
        assert_eq!(empty.len(), 0);
        // One decided cell
        assert_eq!(decided.len(), 1);
        assert!(decided.contains(Position::new(0, 0)));
    }

    #[test]
    fn test_classify_cells_with_empty_position() {
        let mut grid = CandidateGrid::new();
        let pos = Position::new(4, 4);

        // Remove all candidates to create an empty cell
        for digit in Digit::ALL {
            grid.remove_candidate(pos, digit);
        }

        let [empty] = grid.classify_cells();

        // One empty cell
        assert_eq!(empty.len(), 1);
        assert!(empty.contains(pos));
    }

    #[test]
    fn test_placed_digits_are_unique_empty_grid() {
        let grid = CandidateGrid::new();
        let decided_cells = DigitPositions::new();
        assert!(grid.placed_digits_are_unique(decided_cells));
    }

    #[test]
    fn test_placed_digits_are_unique_single_digit() {
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D1);

        let [_, decided_cells] = grid.classify_cells();
        assert!(grid.placed_digits_are_unique(decided_cells));
    }

    #[test]
    fn test_placed_digits_are_unique_valid_placements() {
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D1);
        grid.place(Position::new(1, 1), D2);
        grid.place(Position::new(2, 2), D3);
        grid.place(Position::new(3, 3), D4);

        let [_, decided_cells] = grid.classify_cells();
        assert!(grid.placed_digits_are_unique(decided_cells));
    }

    #[test]
    fn test_classify_cells_with_n3_multiple_candidates() {
        let mut grid = CandidateGrid::new();

        // Create a cell with exactly 2 candidates
        let pos_two = Position::new(0, 0);
        for digit in [D3, D4, D5, D6, D7, D8, D9] {
            grid.remove_candidate(pos_two, digit);
        }
        // pos_two now has candidates: D1, D2 (2 candidates)

        // Create a cell with exactly 1 candidate
        let pos_one = Position::new(1, 1);
        for digit in [D2, D3, D4, D5, D6, D7, D8, D9] {
            grid.remove_candidate(pos_one, digit);
        }
        // pos_one now has candidate: D1 (1 candidate)

        let [empty, one_candidate, two_candidates] = grid.classify_cells();

        assert_eq!(empty.len(), 0);
        assert_eq!(one_candidate.len(), 1);
        assert!(one_candidate.contains(pos_one));
        assert_eq!(two_candidates.len(), 1);
        assert!(two_candidates.contains(pos_two));
    }

    #[test]
    fn test_classify_cells_with_n4_various_counts() {
        let mut grid = CandidateGrid::new();

        // Cell with 1 candidate
        let pos_one = Position::new(0, 0);
        for digit in [D2, D3, D4, D5, D6, D7, D8, D9] {
            grid.remove_candidate(pos_one, digit);
        }

        // Cell with 2 candidates
        let pos_two = Position::new(1, 1);
        for digit in [D3, D4, D5, D6, D7, D8, D9] {
            grid.remove_candidate(pos_two, digit);
        }

        // Cell with 3 candidates
        let pos_three = Position::new(2, 2);
        for digit in [D4, D5, D6, D7, D8, D9] {
            grid.remove_candidate(pos_three, digit);
        }

        let [empty, one, two, three] = grid.classify_cells();

        assert_eq!(empty.len(), 0);
        assert_eq!(one.len(), 1);
        assert!(one.contains(pos_one));
        assert_eq!(two.len(), 1);
        assert!(two.contains(pos_two));
        assert_eq!(three.len(), 1);
        assert!(three.contains(pos_three));
    }

    #[test]
    fn test_classify_cells_with_large_n() {
        let mut grid = CandidateGrid::new();

        // Place one digit to create a decided cell
        grid.place(Position::new(0, 0), D1);

        // Request more classifications than possible (max 9 candidates)
        let [
            _empty,
            one,
            _two,
            _three,
            _four,
            _five,
            _six,
            _seven,
            _eight,
            nine,
            ten,
        ]: [DigitPositions; 11] = grid.classify_cells();

        // Should have one decided cell
        assert_eq!(one.len(), 1);

        // Positions with 10 or more candidates don't exist in sudoku
        assert_eq!(ten.len(), 0);

        // Most cells should have fewer than 9 candidates due to the placement
        // (cells in same row/col/box will have 8 or fewer)
        assert!(nine.len() < 81);
    }

    #[test]
    fn test_classify_cells_n10_full_range() {
        let grid = CandidateGrid::new();

        // In an empty grid, all 81 cells have 9 candidates
        let [c0, c1, c2, c3, c4, c5, c6, c7, c8, c9]: [DigitPositions; 10] = grid.classify_cells();

        assert_eq!(c0.len(), 0); // No empty cells
        assert_eq!(c1.len(), 0); // No decided cells
        assert_eq!(c2.len(), 0); // No cells with 2 candidates
        assert_eq!(c3.len(), 0); // No cells with 3 candidates
        assert_eq!(c4.len(), 0); // No cells with 4 candidates
        assert_eq!(c5.len(), 0); // No cells with 5 candidates
        assert_eq!(c6.len(), 0); // No cells with 6 candidates
        assert_eq!(c7.len(), 0); // No cells with 7 candidates
        assert_eq!(c8.len(), 0); // No cells with 8 candidates
        assert_eq!(c9.len(), 81); // All 81 cells have 9 candidates
    }
}
