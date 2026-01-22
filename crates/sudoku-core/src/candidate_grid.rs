//! Candidate grid for sudoku solving.

use std::iter;

use crate::{
    DigitGrid,
    containers::{Array9, BitSet9, BitSet81},
    digit::Digit,
    index::{CellIndexSemantics, DigitSemantics, Index9, Index9Semantics, PositionSemantics},
    position::Position,
};

/// A set of sudoku digits (1-9).
///
/// A specialized [`BitSet9`] using [`DigitSemantics`], providing type-safe set operations
/// on digits through the [Semantics Pattern](crate#semantics-pattern-type-safe-indexing).
///
/// See the [crate-level documentation](crate#semantics-pattern-type-safe-indexing) for details.
pub type DigitSet = BitSet9<DigitSemantics>;

/// A set of grid positions.
///
/// A specialized [`BitSet81`] using [`PositionSemantics`], providing type-safe set operations
/// on positions through the [Semantics Pattern](crate#semantics-pattern-type-safe-indexing).
///
/// See the [crate-level documentation](crate#semantics-pattern-type-safe-indexing) for details.
pub type DigitPositions = BitSet81<PositionSemantics>;

impl DigitPositions {
    /// Precomputed positions for each row.
    ///
    /// `ROW_POSITIONS[y]` contains all 9 positions in row `y`.
    pub const ROW_POSITIONS: Array9<DigitPositions, CellIndexSemantics> = {
        let mut masks = [DigitPositions::EMPTY; 9];
        let mut y = 0u8;
        while y < 9 {
            let mut bits = 0u128;
            let mut x = 0u8;
            while x < 9 {
                // Position index: y * 9 + x
                bits |= 1u128 << (y * 9 + x);
                x += 1;
            }
            masks[y as usize] = DigitPositions::from_bits(bits);
            y += 1;
        }
        Array9::from_array(masks)
    };

    /// Precomputed positions for each column.
    ///
    /// `COLUMN_POSITIONS[x]` contains all 9 positions in column `x`.
    pub const COLUMN_POSITIONS: Array9<DigitPositions, CellIndexSemantics> = {
        let mut masks = [DigitPositions::EMPTY; 9];
        let mut x = 0u8;
        while x < 9 {
            let mut bits = 0u128;
            let mut y = 0u8;
            while y < 9 {
                // Position index: y * 9 + x
                bits |= 1u128 << (y * 9 + x);
                y += 1;
            }
            masks[x as usize] = DigitPositions::from_bits(bits);
            x += 1;
        }
        Array9::from_array(masks)
    };

    /// Precomputed positions for each 3×3 box.
    ///
    /// `BOX_POSITIONS[box_index]` contains all 9 positions in that box.
    /// Boxes are numbered 0-8 from left to right, top to bottom.
    pub const BOX_POSITIONS: Array9<DigitPositions, CellIndexSemantics> = {
        let mut masks = [DigitPositions::EMPTY; 9];
        let mut box_index = 0u8;
        while box_index < 9 {
            let mut bits = 0u128;
            let box_x = (box_index % 3) * 3;
            let box_y = (box_index / 3) * 3;
            let mut dy = 0u8;
            while dy < 3 {
                let mut dx = 0u8;
                while dx < 3 {
                    let x = box_x + dx;
                    let y = box_y + dy;
                    // Position index: y * 9 + x
                    bits |= 1u128 << (y * 9 + x);
                    dx += 1;
                }
                dy += 1;
            }
            masks[box_index as usize] = DigitPositions::from_bits(bits);
            box_index += 1;
        }
        Array9::from_array(masks)
    };

    /// Returns a bitmask of positions in the specified row.
    ///
    /// The returned mask contains the column indices (0-8) where positions exist
    /// in the given row.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::{DigitPositions, Position};
    ///
    /// let mut positions = DigitPositions::new();
    /// positions.insert(Position::new(2, 0)); // Column 2, Row 0
    /// positions.insert(Position::new(5, 0)); // Column 5, Row 0
    /// positions.insert(Position::new(3, 1)); // Column 3, Row 1
    ///
    /// let mask = positions.row_mask(0);
    /// assert_eq!(mask.len(), 2); // Two positions in row 0
    /// assert!(mask.contains(2));
    /// assert!(mask.contains(5));
    /// ```
    #[must_use]
    pub fn row_mask(&self, y: u8) -> HouseMask {
        let mut mask = HouseMask::new();
        for pos in *self & Self::ROW_POSITIONS[y] {
            mask.insert(pos.x());
        }
        mask
    }

    /// Returns a bitmask of positions in the specified column.
    ///
    /// The returned mask contains the row indices (0-8) where positions exist
    /// in the given column.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::{DigitPositions, Position};
    ///
    /// let mut positions = DigitPositions::new();
    /// positions.insert(Position::new(0, 1)); // Column 0, Row 1
    /// positions.insert(Position::new(0, 4)); // Column 0, Row 4
    /// positions.insert(Position::new(1, 2)); // Column 1, Row 2
    ///
    /// let mask = positions.col_mask(0);
    /// assert_eq!(mask.len(), 2); // Two positions in column 0
    /// assert!(mask.contains(1));
    /// assert!(mask.contains(4));
    /// ```
    #[must_use]
    pub fn col_mask(&self, x: u8) -> HouseMask {
        let mut mask = HouseMask::new();
        for pos in *self & Self::COLUMN_POSITIONS[x] {
            mask.insert(pos.y());
        }
        mask
    }

    /// Returns a bitmask of positions in the specified 3×3 box.
    ///
    /// The returned mask contains the cell indices (0-8) within the box where positions exist.
    /// Boxes are numbered 0-8 from left to right, top to bottom.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::{DigitPositions, Position};
    ///
    /// let mut positions = DigitPositions::new();
    /// positions.insert(Position::new(0, 0)); // Top-left corner of box 0
    /// positions.insert(Position::new(1, 1)); // Center area of box 0
    /// positions.insert(Position::new(3, 0)); // Box 1
    ///
    /// let mask = positions.box_mask(0);
    /// assert_eq!(mask.len(), 2); // Two positions in box 0
    /// ```
    #[must_use]
    pub fn box_mask(&self, box_index: u8) -> HouseMask {
        let mut mask = HouseMask::new();
        for pos in *self & Self::BOX_POSITIONS[box_index] {
            mask.insert(pos.box_cell_index());
        }
        mask
    }
}

/// A set of cell indices (0-8) within a house (row, column, or box).
///
/// A specialized [`BitSet9`] using [`CellIndexSemantics`], providing type-safe set operations
/// on cell indices through the [Semantics Pattern](crate#semantics-pattern-type-safe-indexing).
///
/// See the [crate-level documentation](crate#semantics-pattern-type-safe-indexing) for details.
pub type HouseMask = BitSet9<CellIndexSemantics>;

/// Candidate grid for sudoku solving.
///
/// Manages possible placements for each digit (1-9) across the entire 9x9 grid.
/// Internally stores 9 [`DigitPositions`] (one per digit), each tracking the 81 grid
/// positions where that digit can be placed.
///
/// The internal representation uses [`Array9`] with [`DigitSemantics`] to ensure
/// type-safe indexing through the [Semantics Pattern](crate#semantics-pattern-type-safe-indexing).
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
/// assert_eq!(grid.candidates_at(pos).len(), 9);
///
/// // Place digit 1 at (0, 0)
/// grid.place(pos, Digit::D1);
///
/// // Now (0, 0) only has digit 1
/// let candidates = grid.candidates_at(pos);
/// assert_eq!(candidates.len(), 1);
/// assert!(candidates.contains(Digit::D1));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGrid {
    /// `digit_positions[digit]` represents possible positions for that digit
    digit_positions: Array9<DigitPositions, DigitSemantics>,
}

impl CandidateGrid {
    /// Creates a `CandidateGrid` from a `DigitGrid`.
    ///
    /// Each digit in the input grid is placed into the corresponding position,
    /// removing other candidates from that cell.
    ///
    /// # Example
    ///
    /// ```
    /// use sudoku_core::{CandidateGrid, DigitGrid, Digit, Position};
    /// use std::str::FromStr;
    ///
    /// let digit_grid = DigitGrid::from_str("5________ _________ _________ _________ _________ _________ _________ _________ _________").unwrap();
    /// let grid = CandidateGrid::from_digit_grid(&digit_grid);
    ///
    /// // D5 is placed at (0, 0)
    /// let candidates = grid.candidates_at(Position::new(0, 0));
    /// assert_eq!(candidates.len(), 1);
    /// assert!(candidates.contains(Digit::D5));
    ///
    /// // D5 is still a candidate in the same row
    /// let candidates = grid.candidates_at(Position::new(1, 0));
    /// assert!(candidates.contains(Digit::D5));
    /// ```
    #[must_use]
    pub fn from_digit_grid(digit_grid: &crate::DigitGrid) -> Self {
        let mut grid = Self::new();
        for pos in Position::ALL {
            if let Some(digit) = digit_grid.get(pos) {
                grid.place(pos, digit);
            }
        }
        grid
    }
}

impl Default for CandidateGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// Error returned when a candidate grid is in an inconsistent state.
///
/// This error indicates that the grid violates Sudoku constraints:
/// - At least one cell has no remaining candidates (empty cell), or
/// - Duplicate decided digits (cells with exactly one candidate) exist in the same row, column, or box
///
/// An inconsistent grid cannot be solved and typically results from
/// incorrect placements or contradictory constraints.
///
/// This error distinguishes between two types of inconsistencies that can occur in a candidate grid:
/// - [`NoCandidates`]: A cell has no remaining candidates, making it impossible to place a digit
/// - [`DuplicatedDecidedDigits`]: Multiple cells in the same row, column, or box have the same decided digit
///
/// [`NoCandidates`]: ConsistencyError::NoCandidates
/// [`DuplicatedDecidedDigits`]: ConsistencyError::DuplicatedDecidedDigits
///
/// # Examples
///
/// ```
/// use sudoku_core::{CandidateGrid, Digit, Position};
///
/// let mut grid = CandidateGrid::new();
///
/// // Create a contradiction by removing all candidates from a cell
/// let pos = Position::new(0, 0);
/// for digit in Digit::ALL {
///     grid.remove_candidate(pos, digit);
/// }
///
/// // check_consistency will detect this as NoCandidates error
/// assert!(grid.check_consistency().is_err());
/// ```
#[derive(Debug, Clone, Copy, derive_more::Display, derive_more::Error)]
pub enum ConsistencyError {
    /// A cell has no remaining candidates.
    ///
    /// This occurs when candidate removal results in a cell with no possible digits,
    /// making the puzzle unsolvable.
    #[display("candidate grid has cells with no candidates")]
    NoCandidates,
    /// Multiple cells in the same constraint region have the same decided digit.
    ///
    /// This occurs when the same digit appears more than once in a row, column, or box,
    /// violating Sudoku rules.
    #[display("candidate grid has duplicated decided digits")]
    DuplicatedDecidedDigits,
}

impl CandidateGrid {
    /// Creates a new candidate grid with all positions available for all digits.
    #[must_use]
    pub fn new() -> Self {
        Self {
            digit_positions: Array9::from([DigitPositions::FULL; 9]),
        }
    }

    /// Converts the candidate grid to a digit grid containing only decided cells.
    ///
    /// A cell is considered "decided" when it has exactly one candidate remaining.
    /// Only these decided cells are populated in the returned grid; all other cells
    /// remain empty (`None`).
    ///
    /// This is useful for extracting the current solved state of the puzzle from
    /// the candidate grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::{CandidateGrid, Digit, Position};
    ///
    /// let mut grid = CandidateGrid::new();
    /// grid.place(Position::new(0, 0), Digit::D1);
    /// grid.place(Position::new(1, 0), Digit::D2);
    ///
    /// let digit_grid = grid.to_digit_grid();
    /// assert_eq!(digit_grid.get(Position::new(0, 0)), Some(Digit::D1));
    /// assert_eq!(digit_grid.get(Position::new(1, 0)), Some(Digit::D2));
    /// assert_eq!(digit_grid.get(Position::new(2, 0)), None); // Not decided yet
    /// ```
    #[must_use]
    pub fn to_digit_grid(&self) -> DigitGrid {
        let mut grid = DigitGrid::new();
        let [_empty_cells, decided_cells] = self.classify_cells();
        // For each digit, find cells where it's the only candidate and place it
        for digit in Digit::ALL {
            let digit_cells = &self.digit_positions[digit];
            for pos in *digit_cells & decided_cells {
                grid.set(pos, Some(digit));
            }
        }
        grid
    }

    /// Places a digit at a position by removing all other candidates from that position.
    ///
    /// This only affects the placed cell itself - other cells in the same row, column,
    /// or box are not modified.
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
        for (d, digits) in iter::zip(Digit::ALL, &mut self.digit_positions) {
            changed |= digits.set(pos, d == digit);
        }
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
        self.digit_positions[digit].remove(pos)
    }

    /// Removes a candidate digit from all positions specified by a mask.
    ///
    /// This is the batch version of [`remove_candidate`](Self::remove_candidate),
    /// allowing you to remove the same digit from multiple positions at once.
    ///
    /// Returns `true` if any candidate was removed, `false` if the digit was
    /// not a candidate at any of the masked positions.
    ///
    /// # Example
    ///
    /// ```
    /// use sudoku_core::{CandidateGrid, Digit, DigitPositions, Position};
    ///
    /// let mut grid = CandidateGrid::new();
    ///
    /// // Remove D5 from all positions in row 0
    /// let row_mask = DigitPositions::ROW_POSITIONS[0];
    /// let changed = grid.remove_candidate_with_mask(row_mask, Digit::D5);
    /// assert!(changed); // D5 was removed from 9 positions
    ///
    /// // Try removing again - nothing changes
    /// let changed = grid.remove_candidate_with_mask(row_mask, Digit::D5);
    /// assert!(!changed); // D5 is already removed from all positions
    /// ```
    pub fn remove_candidate_with_mask(&mut self, mask: DigitPositions, digit: Digit) -> bool {
        let before = self.digit_positions[digit];
        self.digit_positions[digit] &= !mask;
        before != self.digit_positions[digit]
    }

    /// Returns the set of all positions where the specified digit can be placed.
    #[must_use]
    pub fn digit_positions(&self, digit: Digit) -> DigitPositions {
        self.digit_positions[digit]
    }

    /// Returns the set of candidate digits that can be placed at a position.
    #[must_use]
    pub fn candidates_at(&self, pos: Position) -> DigitSet {
        let mut candidates = DigitSet::new();
        for (i, digit_pos) in (0..).zip(&self.digit_positions) {
            if digit_pos.contains(pos) {
                candidates.insert(DigitSemantics::from_index(Index9::new(i)));
            }
        }
        candidates
    }

    /// Returns a bitmask of candidate positions in the specified row for the digit.
    ///
    /// If the returned mask has only one bit set, a Hidden Single is detected.
    #[must_use]
    pub fn row_mask(&self, y: u8, digit: Digit) -> HouseMask {
        self.digit_positions[digit].row_mask(y)
    }

    /// Returns a bitmask of candidate positions in the specified column for the digit.
    ///
    /// If the returned mask has only one bit set, a Hidden Single is detected.
    #[must_use]
    pub fn col_mask(&self, x: u8, digit: Digit) -> HouseMask {
        self.digit_positions[digit].col_mask(x)
    }

    /// Returns a bitmask of candidate positions in the specified box for the digit.
    ///
    /// If the returned mask has only one bit set, a Hidden Single is detected.
    #[must_use]
    pub fn box_mask(&self, box_index: u8, digit: Digit) -> HouseMask {
        self.digit_positions[digit].box_mask(box_index)
    }

    /// Checks if the grid is in a consistent state.
    ///
    /// Returns `Ok(())` if the grid is consistent, or `Err(ConsistencyError)` if:
    /// - Any cell has no remaining candidates (empty cell), or
    /// - Duplicate decided digits (cells with exactly one candidate) exist in the same row, column, or box
    ///
    /// This method is useful during solving to detect contradictions early.
    /// Unlike [`is_solved`], this does NOT require all cells to be decided.
    ///
    /// # Errors
    ///
    /// Returns [`ConsistencyError`] if the grid violates Sudoku constraints.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::{CandidateGrid, Digit, Position};
    ///
    /// let mut grid = CandidateGrid::new();
    /// assert!(grid.check_consistency().is_ok());
    ///
    /// grid.place(Position::new(0, 0), Digit::D5);
    /// assert!(grid.check_consistency().is_ok()); // Still consistent after placing
    ///
    /// // Create a contradiction
    /// let pos = Position::new(1, 1);
    /// for digit in Digit::ALL {
    ///     grid.remove_candidate(pos, digit);
    /// }
    /// assert!(grid.check_consistency().is_err()); // Now inconsistent
    /// ```
    ///
    /// [`is_solved`]: CandidateGrid::is_solved
    pub fn check_consistency(&self) -> Result<(), ConsistencyError> {
        let [empty_cells, decided_cells] = self.classify_cells();
        if !empty_cells.is_empty() {
            return Err(ConsistencyError::NoCandidates);
        }
        if !self.placed_digits_are_unique(decided_cells) {
            return Err(ConsistencyError::DuplicatedDecidedDigits);
        }
        Ok(())
    }

    /// Checks if the puzzle is **solved** (complete and consistent).
    ///
    /// A grid is solved if:
    /// - All 81 positions have exactly one candidate (complete)
    /// - No position has zero candidates (no contradictions)
    /// - All definite digits satisfy sudoku uniqueness constraints (no duplicates)
    ///
    /// # Errors
    ///
    /// Returns [`ConsistencyError`] if the grid is inconsistent (has empty cells
    /// or duplicate digits).
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::CandidateGrid;
    ///
    /// let grid = CandidateGrid::new();
    /// assert!(!grid.is_solved().unwrap()); // Empty grid is not solved but is consistent
    /// ```
    pub fn is_solved(&self) -> Result<bool, ConsistencyError> {
        let [empty_cells, decided_cells] = self.classify_cells();
        if !empty_cells.is_empty() {
            return Err(ConsistencyError::NoCandidates);
        }
        if !self.placed_digits_are_unique(decided_cells) {
            return Err(ConsistencyError::DuplicatedDecidedDigits);
        }
        Ok(decided_cells.len() == 81)
    }

    /// Returns all positions that have exactly one candidate (decided cells).
    ///
    /// A cell is considered "decided" when it has only one possible candidate digit,
    /// either because it was explicitly placed or because all other candidates were
    /// eliminated.
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
    /// **Important**: Positions with `N` or more candidates are not included in any element.
    /// They are implicitly ignored by the algorithm.
    ///
    /// # Algorithm
    ///
    /// This method uses a **bitwise dynamic programming** approach to classify cells efficiently:
    ///
    /// 1. **Initialization**: Start with `cells[0] = FULL` (all 81 positions).
    ///    This represents the state "before processing any digits, all positions have 0 candidates".
    ///
    /// 2. **Digit Processing**: For each of the 9 digits (D1 through D9):
    ///    - For each candidate count `i` (in reverse order from `min(n, N-1)` down to 1):
    ///      - Remove positions where the current digit exists: `cells[i] &= !digit_pos`
    ///      - Add positions that transition from `i-1` to `i` candidates: `cells[i] |= cells[i-1] & digit_pos`
    ///    - Update `cells[0]` by removing positions with this digit: `cells[0] &= !digit_pos`
    ///
    /// 3. **Result**: After processing all 9 digits, `cells[i]` contains positions with exactly `i` candidates.
    ///
    /// This approach achieves **O(9 × N)** time complexity with efficient bitwise operations,
    /// where N is typically small (2-10), making it much faster than checking each of 81 cells individually.
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
    #[must_use]
    pub fn classify_cells<const N: usize>(&self) -> [DigitPositions; N] {
        let mut cells = [DigitPositions::EMPTY; N];

        // Initialize: all positions start with 0 candidates (before processing any digits)
        cells[0] = DigitPositions::FULL;

        // Process each digit (D1..D9) and update candidate counts
        for (n, digit_pos) in iter::zip(1.., self.digit_positions.iter().copied()) {
            let end = usize::min(n + 1, N);

            // Update each candidate count in reverse order to avoid overwriting
            // positions we need to read from in the same iteration
            for i in (1..end).rev() {
                // Remove positions where this digit exists (they already had i candidates)
                cells[i] &= !digit_pos;
                // Add positions transitioning from (i-1) to i candidates
                cells[i] |= cells[i - 1] & digit_pos;
            }

            // Remove positions with this digit from the "0 candidates" set
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
            let digit_cells = &self.digit_positions[digit];
            let decided_digit_cells = *digit_cells & decided_cells;
            for pos in decided_digit_cells {
                if decided_digit_cells.row_mask(pos.y()).len() != 1 {
                    return false;
                }
                if decided_digit_cells.col_mask(pos.x()).len() != 1 {
                    return false;
                }
                if decided_digit_cells.box_mask(pos.box_index()).len() != 1 {
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
    fn test_new_and_basic_operations() {
        let mut grid = CandidateGrid::new();

        // Initial state: all candidates available everywhere
        assert_eq!(grid.candidates_at(Position::new(0, 0)).len(), 9);
        assert_eq!(grid.digit_positions(D1).len(), 81);

        let pos = Position::new(4, 4);
        assert!(grid.place(pos, D5));

        // Place is idempotent
        assert!(!grid.place(pos, D5));

        // Placement reduces cell to single candidate and removes others
        let candidates = grid.candidates_at(pos);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(D5));
        assert!(!grid.digit_positions(D1).contains(pos));
    }

    #[test]
    fn test_remove_candidate() {
        let mut grid = CandidateGrid::new();
        let pos = Position::new(3, 3);

        assert!(grid.remove_candidate(pos, D5));

        // Remove is idempotent
        assert!(!grid.remove_candidate(pos, D5));

        let candidates = grid.candidates_at(pos);
        assert_eq!(candidates.len(), 8);
        assert!(!candidates.contains(D5));
    }

    #[test]
    fn test_remove_candidate_with_mask() {
        let mut grid = CandidateGrid::new();

        let cases = [
            (
                "row",
                DigitPositions::ROW_POSITIONS[0],
                D5,
                Position::new(3, 0),
            ),
            (
                "column",
                DigitPositions::COLUMN_POSITIONS[5],
                D7,
                Position::new(5, 4),
            ),
            (
                "box",
                DigitPositions::BOX_POSITIONS[4],
                D9,
                Position::new(4, 4),
            ),
        ];

        for (name, mask, digit, pos_in_mask) in cases {
            let mut grid = CandidateGrid::new();

            assert!(grid.remove_candidate_with_mask(mask, digit), "{name}");

            // Removal is idempotent
            assert!(!grid.remove_candidate_with_mask(mask, digit), "{name}");

            assert!(!grid.candidates_at(pos_in_mask).contains(digit), "{name}");
        }

        // Empty mask is no-op
        assert!(!grid.remove_candidate_with_mask(DigitPositions::EMPTY, D1));
    }

    #[test]
    fn test_decided_cells() {
        let mut grid = CandidateGrid::new();

        assert_eq!(grid.decided_cells().len(), 0);

        grid.place(Position::new(0, 0), D1);
        grid.place(Position::new(5, 5), D5);

        // Placed cells are detected as decided
        let decided = grid.decided_cells();
        assert_eq!(decided.len(), 2);
        assert!(decided.contains(Position::new(0, 0)));
        assert!(decided.contains(Position::new(5, 5)));
    }

    #[test]
    fn test_digit_positions_after_placement() {
        let mut grid = CandidateGrid::new();
        let pos = Position::new(4, 4);
        grid.place(pos, D5);

        let positions = grid.digit_positions(D5);

        // Placed digit remains candidate in same row/col/box (not eliminated globally)
        assert!(positions.contains(pos));
        assert!(positions.contains(Position::new(0, 4)));
        assert!(positions.contains(Position::new(4, 0)));
        assert!(positions.contains(Position::new(3, 3)));

        // Other digits eliminated from placed cell
        assert!(!grid.digit_positions(D1).contains(pos));
    }

    #[test]
    fn test_house_masks() {
        let mut grid = CandidateGrid::new();

        for pos in Position::ROWS[2] {
            if pos.x() != 1 && pos.x() != 3 && pos.x() != 5 {
                grid.remove_candidate(pos, D3);
            }
        }

        // Mask identifies positions in house where digit is still candidate
        let mask = grid.row_mask(2, D3);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(1));
        assert!(mask.contains(3));
        assert!(mask.contains(5));
    }

    #[test]
    fn test_hidden_single_detection() {
        let mut grid = CandidateGrid::new();
        for pos in Position::ROWS[0] {
            if pos.x() != 3 {
                grid.remove_candidate(pos, D5);
            }
        }
        // Mask detects hidden single (only one position remaining)
        assert_eq!(grid.row_mask(0, D5).len(), 1);

        let mut grid = CandidateGrid::new();
        for pos in Position::COLUMNS[5] {
            if pos.y() != 4 {
                grid.remove_candidate(pos, D7);
            }
        }
        assert_eq!(grid.col_mask(5, D7).len(), 1);

        let mut grid = CandidateGrid::new();
        for pos in Position::BOXES[4] {
            if pos.box_cell_index() != 4 {
                grid.remove_candidate(pos, D9);
            }
        }
        assert_eq!(grid.box_mask(4, D9).len(), 1);
    }

    #[test]
    fn test_check_consistency() {
        let mut grid = CandidateGrid::new();
        assert!(grid.check_consistency().is_ok());

        grid.place(Position::new(0, 0), D5);
        assert!(grid.check_consistency().is_ok());

        // Detects contradiction: cell with no candidates
        let mut grid = CandidateGrid::new();
        let pos = Position::new(4, 4);
        for digit in Digit::ALL {
            grid.remove_candidate(pos, digit);
        }
        assert!(grid.check_consistency().is_err());

        // Detects violation: duplicate digits in same house
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D5);
        grid.place(Position::new(0, 1), D5);
        assert!(grid.check_consistency().is_err());
    }

    #[test]
    fn test_is_solved() {
        let grid = CandidateGrid::new();
        assert!(!grid.is_solved().unwrap());

        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D1);
        assert!(!grid.is_solved().unwrap());

        // Returns error when grid is inconsistent
        let mut grid = CandidateGrid::new();
        for digit in Digit::ALL {
            grid.remove_candidate(Position::new(0, 0), digit);
        }
        assert!(grid.is_solved().is_err());
    }

    #[test]
    fn test_classify_cells() {
        let grid = CandidateGrid::new();
        let [empty, decided] = grid.classify_cells();
        assert_eq!(empty.len(), 0);
        assert_eq!(decided.len(), 0);

        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D5);
        let [empty, decided] = grid.classify_cells();
        assert_eq!(empty.len(), 0);
        assert_eq!(decided.len(), 1);
        assert!(decided.contains(Position::new(0, 0)));

        // Detects cells with zero candidates
        let mut grid = CandidateGrid::new();
        let pos = Position::new(4, 4);
        for digit in Digit::ALL {
            grid.remove_candidate(pos, digit);
        }
        let [empty] = grid.classify_cells();
        assert_eq!(empty.len(), 1);
        assert!(empty.contains(pos));
    }

    #[test]
    fn test_classify_cells_multiple_n() {
        let mut grid = CandidateGrid::new();

        let pos_one = Position::new(0, 0);
        for digit in [D2, D3, D4, D5, D6, D7, D8, D9] {
            grid.remove_candidate(pos_one, digit);
        }

        let pos_two = Position::new(1, 1);
        for digit in [D3, D4, D5, D6, D7, D8, D9] {
            grid.remove_candidate(pos_two, digit);
        }

        let pos_three = Position::new(2, 2);
        for digit in [D4, D5, D6, D7, D8, D9] {
            grid.remove_candidate(pos_three, digit);
        }

        // Correctly classifies cells by candidate count
        let [empty, one, two, three] = grid.classify_cells();
        assert_eq!(empty.len(), 0);
        assert_eq!(one.len(), 1);
        assert_eq!(two.len(), 1);
        assert_eq!(three.len(), 1);

        // Handles arbitrary N (empty grid: all 81 cells have 9 candidates)
        let grid = CandidateGrid::new();
        let [c0, _c1, _c2, _c3, _c4, _c5, _c6, _c7, _c8, c9]: [DigitPositions; 10] =
            grid.classify_cells();
        assert_eq!(c9.len(), 81);
        assert_eq!(c0.len(), 0);
    }

    #[test]
    fn test_digit_positions_mask_methods() {
        let mut positions = DigitPositions::new();
        positions.insert(Position::new(0, 0));
        positions.insert(Position::new(2, 0));
        positions.insert(Position::new(5, 0));
        positions.insert(Position::new(1, 1));

        let mask = positions.row_mask(0);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(0));
        assert!(mask.contains(2));
        assert!(mask.contains(5));

        let mask = positions.row_mask(1);
        assert_eq!(mask.len(), 1);
        assert!(mask.contains(1));

        let mask = positions.row_mask(2);
        assert_eq!(mask.len(), 0);
    }

    #[test]
    fn test_from_digit_grid() {
        use std::str::FromStr;
        let digit_grid = crate::DigitGrid::from_str(
            "5________ _________ _________ _________ _________ _________ _________ _________ _________",
        )
        .unwrap();

        let grid = CandidateGrid::from_digit_grid(&digit_grid);

        // Placed digits become single candidates
        let candidates = grid.candidates_at(Position::new(0, 0));
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(D5));

        // Placed digits not eliminated from same house (solver's responsibility)
        assert!(grid.candidates_at(Position::new(1, 0)).contains(D5));
        assert!(grid.candidates_at(Position::new(0, 1)).contains(D5));
        assert!(grid.candidates_at(Position::new(1, 1)).contains(D5));
    }

    #[test]
    fn test_to_digit_grid() {
        let grid = CandidateGrid::new();
        let digit_grid = grid.to_digit_grid();
        for pos in Position::ALL {
            assert_eq!(digit_grid.get(pos), None);
        }

        // Only decided cells (single candidate) appear in digit grid
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D1);
        grid.place(Position::new(4, 4), D5);
        let digit_grid = grid.to_digit_grid();
        assert_eq!(digit_grid.get(Position::new(0, 0)), Some(D1));
        assert_eq!(digit_grid.get(Position::new(4, 4)), Some(D5));
        assert_eq!(digit_grid.get(Position::new(1, 1)), None);

        // Cells with multiple candidates are omitted
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D1);
        grid.remove_candidate(Position::new(1, 1), D1);
        grid.remove_candidate(Position::new(1, 1), D2);
        let digit_grid = grid.to_digit_grid();
        assert_eq!(digit_grid.get(Position::new(0, 0)), Some(D1));
        assert_eq!(digit_grid.get(Position::new(1, 1)), None);
    }

    #[test]
    fn test_to_digit_grid_roundtrip() {
        use std::str::FromStr;

        let digit_grid_str = "\
            1........\n\
            .2.......\n\
            ..3......\n\
            ...4.....\n\
            ....5....\n\
            .....6...\n\
            ......7..\n\
            .......8.\n\
            ........9";
        let original = DigitGrid::from_str(digit_grid_str).unwrap();
        let candidate_grid = CandidateGrid::from(original);
        let result = candidate_grid.to_digit_grid();

        // DigitGrid → CandidateGrid → DigitGrid preserves placed digits
        assert_eq!(result.get(Position::new(0, 0)), Some(D1));
        assert_eq!(result.get(Position::new(1, 1)), Some(D2));
        assert_eq!(result.get(Position::new(2, 2)), Some(D3));
        assert_eq!(result.get(Position::new(8, 8)), Some(D9));
    }

    #[test]
    fn test_row_positions_constants() {
        // Each row contains exactly 9 positions
        for y in 0..9 {
            assert_eq!(DigitPositions::ROW_POSITIONS[y].len(), 9);
        }

        // Row masks are disjoint (no overlap between different rows)
        for y1 in 0..9 {
            for y2 in (y1 + 1)..9 {
                let intersection =
                    DigitPositions::ROW_POSITIONS[y1] & DigitPositions::ROW_POSITIONS[y2];
                assert_eq!(intersection.len(), 0);
            }
        }

        // Union of all rows covers all 81 positions
        let mut all = DigitPositions::EMPTY;
        for y in 0..9 {
            all |= DigitPositions::ROW_POSITIONS[y];
        }
        assert_eq!(all.len(), 81);
    }

    #[test]
    fn test_column_positions_constants() {
        // Each column contains exactly 9 positions
        for x in 0..9 {
            assert_eq!(DigitPositions::COLUMN_POSITIONS[x].len(), 9);
        }

        // Column masks are disjoint
        for x1 in 0..9 {
            for x2 in (x1 + 1)..9 {
                let intersection =
                    DigitPositions::COLUMN_POSITIONS[x1] & DigitPositions::COLUMN_POSITIONS[x2];
                assert_eq!(intersection.len(), 0);
            }
        }

        // Union of all columns covers all 81 positions
        let mut all = DigitPositions::EMPTY;
        for x in 0..9 {
            all |= DigitPositions::COLUMN_POSITIONS[x];
        }
        assert_eq!(all.len(), 81);
    }

    #[test]
    fn test_box_positions_constants() {
        // Each box contains exactly 9 positions
        for box_idx in 0..9 {
            assert_eq!(DigitPositions::BOX_POSITIONS[box_idx].len(), 9);
        }

        // Box masks are disjoint
        for b1 in 0..9 {
            for b2 in (b1 + 1)..9 {
                let intersection =
                    DigitPositions::BOX_POSITIONS[b1] & DigitPositions::BOX_POSITIONS[b2];
                assert_eq!(intersection.len(), 0);
            }
        }

        // Union of all boxes covers all 81 positions
        let mut all = DigitPositions::EMPTY;
        for box_idx in 0..9 {
            all |= DigitPositions::BOX_POSITIONS[box_idx];
        }
        assert_eq!(all.len(), 81);
    }
}
