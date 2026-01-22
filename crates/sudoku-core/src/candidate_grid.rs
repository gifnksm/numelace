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
/// // Place digit 1 at (0, 0) - removes candidates from row, col, box
/// grid.place(pos, Digit::D1);
///
/// // Now (0, 0) only has digit 1
/// let candidates = grid.candidates_at(pos);
/// assert_eq!(candidates.len(), 1);
/// assert!(candidates.contains(Digit::D1));
///
/// // Other cells in the row no longer have digit 1 as candidate
/// let row_mask = grid.row_mask(0, Digit::D1);
/// assert_eq!(row_mask.len(), 1); // Only at (0, 0)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGrid {
    /// `digit_positions[digit]` represents possible positions for that digit
    digit_positions: Array9<DigitPositions, DigitSemantics>,
}

impl CandidateGrid {
    /// Creates a `CandidateGrid` from a `DigitGrid` with constraint propagation.
    ///
    /// Places digits from the grid using [`place`](Self::place), which removes
    /// candidates from related cells according to sudoku rules. This is the
    /// standard way to create a candidate grid from placed digits.
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
    /// // D5 is not a candidate in the same row as (0, 0)
    /// let candidates = grid.candidates_at(Position::new(1, 0));
    /// assert!(!candidates.contains(Digit::D5));
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

    /// Creates a `CandidateGrid` from a `DigitGrid` without constraint propagation.
    ///
    /// Places digits from the grid using [`place_no_propagation`](Self::place_no_propagation),
    /// which does not remove candidates from related cells. This results in a grid with
    /// redundant candidates, useful for testing scenarios.
    ///
    /// For normal use cases with full constraint propagation, use
    /// [`from_digit_grid`](Self::from_digit_grid) instead.
    ///
    /// # Example
    ///
    /// ```
    /// use sudoku_core::{CandidateGrid, DigitGrid, Digit, Position};
    /// use std::str::FromStr;
    ///
    /// let digit_grid = DigitGrid::from_str("5________ _________ _________ _________ _________ _________ _________ _________ _________").unwrap();
    /// let grid = CandidateGrid::from_digit_grid_no_propagation(&digit_grid);
    ///
    /// // D5 is placed at (0, 0)
    /// let candidates = grid.candidates_at(Position::new(0, 0));
    /// assert!(candidates.contains(Digit::D5));
    ///
    /// // But D5 is still a candidate in the same row (redundant)
    /// let candidates = grid.candidates_at(Position::new(1, 0));
    /// assert!(candidates.contains(Digit::D5));
    /// ```
    #[must_use]
    pub fn from_digit_grid_no_propagation(digit_grid: &crate::DigitGrid) -> Self {
        let mut grid = Self::new();
        for pos in Position::ALL {
            if let Some(digit) = digit_grid.get(pos) {
                grid.place_no_propagation(pos, digit);
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
/// // check_consistency will detect this
/// assert!(grid.check_consistency().is_err());
/// ```
#[derive(Debug, Clone, Copy, derive_more::Display, derive_more::Error)]
#[display("candidate grid has cells with no candidates")]
pub struct ConsistencyError;

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
    ///
    /// For placing a digit without constraint propagation, see
    /// [`place_no_propagation`](Self::place_no_propagation).
    pub fn place(&mut self, pos: Position, digit: Digit) -> bool {
        let mut changed = false;

        // remove all digits around the pos
        for (d, digits) in iter::zip(Digit::ALL, &mut self.digit_positions) {
            if d != digit {
                changed |= digits.remove(pos);
            }
        }

        let mut affected_pos = DigitPositions::ROW_POSITIONS[pos.y()]
            | DigitPositions::COLUMN_POSITIONS[pos.x()]
            | DigitPositions::BOX_POSITIONS[pos.box_index()];
        affected_pos.remove(pos);

        let digit_pos = &mut self.digit_positions[digit];
        if !(*digit_pos & affected_pos).is_empty() {
            changed = true;
            *digit_pos &= !affected_pos;
        }

        changed |= digit_pos.insert(pos);
        changed
    }

    /// Places a digit at a position without propagating constraints.
    ///
    /// This only updates the specified position to have the given digit as its
    /// sole candidate. It does NOT remove candidates from other cells in the
    /// same row, column, or box, resulting in a grid with redundant candidates.
    ///
    /// This is primarily useful for constructing specific test scenarios where
    /// you want to control exactly which candidates remain.
    ///
    /// For normal placement with full constraint propagation, use
    /// [`place`](Self::place) instead.
    ///
    /// # Returns
    ///
    /// Returns `true` if the grid was modified, `false` if the operation
    /// had no effect.
    ///
    /// # Example
    ///
    /// ```
    /// use sudoku_core::{CandidateGrid, Digit, Position};
    ///
    /// let mut grid = CandidateGrid::new();
    /// grid.place_no_propagation(Position::new(0, 0), Digit::D5);
    ///
    /// // D5 is placed at (0, 0)
    /// let candidates = grid.candidates_at(Position::new(0, 0));
    /// assert_eq!(candidates.len(), 1);
    /// assert!(candidates.contains(Digit::D5));
    ///
    /// // But D5 is still a candidate in the same row (redundant)
    /// let candidates = grid.candidates_at(Position::new(1, 0));
    /// assert!(candidates.contains(Digit::D5));
    /// ```
    pub fn place_no_propagation(&mut self, pos: Position, digit: Digit) -> bool {
        let mut changed = false;

        // Remove all other digits from this position
        for (d, digits) in iter::zip(Digit::ALL, &mut self.digit_positions) {
            if d != digit {
                changed |= digits.remove(pos);
            }
        }

        // Ensure digit is at position
        changed |= self.digit_positions[digit].insert(pos);
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
        if !empty_cells.is_empty() || !self.placed_digits_are_unique(decided_cells) {
            return Err(ConsistencyError);
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
        if !empty_cells.is_empty() || !self.placed_digits_are_unique(decided_cells) {
            return Err(ConsistencyError);
        }
        Ok(decided_cells.len() == 81)
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
    fn test_new_grid_has_all_candidates() {
        let grid = CandidateGrid::new();

        // All positions should have all 9 digits as candidates initially
        for pos in Position::ALL {
            let candidates = grid.candidates_at(pos);
            assert_eq!(candidates.len(), 9);
            for digit in Digit::ALL {
                assert!(candidates.contains(digit));
            }
        }
    }

    #[test]
    fn test_place_digit() {
        let mut grid = CandidateGrid::new();

        // Manually set up some candidates
        let pos = Position::new(4, 4); // center
        for digit in &mut grid.digit_positions {
            digit.insert(pos);
        }

        // Place digit 5 at center
        let changed = grid.place(pos, D5);
        assert!(changed, "Placing a digit should return true");

        // The position should only have digit 5
        let candidates = grid.candidates_at(pos);
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
        for pos in Position::ROWS[0] {
            grid.digit_positions[D4].insert(pos);
        }

        // Place digit 5 at (0, 0)
        grid.place(Position::new(0, 0), D5);

        // Digit 5 should be removed from rest of row 0
        for x in 1..9 {
            let row_mask = grid.row_mask(0, D5);
            assert!(
                !row_mask.contains(x),
                "Position ({x}, 0) should not have digit 5"
            );
        }

        // But (5, 3) should still have it
        assert!(grid.candidates_at(Position::new(5, 3)).contains(D3));
    }

    #[test]
    fn test_place_removes_column_candidates() {
        let mut grid = CandidateGrid::new();

        // Set digit 3 as candidate for entire column 5
        for pos in Position::COLUMNS[5] {
            grid.digit_positions[D2].insert(pos);
        }

        // Place digit 3 at (5, 3)
        grid.place(Position::new(5, 3), D3);

        // Digit 3 should be removed from rest of column 5
        for pos in Position::COLUMNS[5] {
            if pos.y() == 3 {
                continue;
            }
            let col_mask = grid.col_mask(5, D3);
            assert!(
                !col_mask.contains(pos.y()),
                "Position (5, {}) should not have digit 3",
                pos.y()
            );
        }
    }

    #[test]
    fn test_place_removes_box_candidates() {
        let mut grid = CandidateGrid::new();

        // Set digit 7 as candidate for entire box 4 (center box)
        for pos in Position::BOXES[4] {
            grid.digit_positions[D6].insert(pos);
        }

        // Place digit 7 at center of center box
        grid.place(Position::new(4, 4), D7);

        // Digit 7 should only be at (4, 4) in box 4
        let box_mask = grid.box_mask(4, D7);
        assert_eq!(box_mask.len(), 1, "Only one position should remain in box");
        assert!(box_mask.contains(4), "Center cell should remain");
    }

    #[test]
    fn test_place_removes_all_candidates_at_position() {
        let mut grid = CandidateGrid::new();

        let pos = Position::new(2, 2);

        // Add all digits as candidates at position
        for digit in &mut grid.digit_positions {
            digit.insert(pos);
        }

        // Place digit 1 there
        grid.place(pos, D1);

        // Now only digit 1 should be there
        let candidates = grid.candidates_at(pos);
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

        let candidates = grid.candidates_at(pos);
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

        // Set single candidate at position without propagating constraints
        grid.place_no_propagation(pos, D7);

        // Position should be considered decided (only one candidate)
        let decided = grid.decided_cells();
        assert_eq!(decided.len(), 1);
        assert!(decided.contains(pos));
    }

    #[test]
    fn test_digit_positions_full_grid() {
        let grid = CandidateGrid::new();

        // Initially all 81 positions are candidates for any digit
        for digit in Digit::ALL {
            let positions = grid.digit_positions(digit);
            assert_eq!(positions.len(), 81);
        }
    }

    #[test]
    fn test_digit_positions_after_placement() {
        let mut grid = CandidateGrid::new();

        // Place digit 5 at (4, 4)
        let pos = Position::new(4, 4);
        grid.place(pos, D5);

        let positions = grid.digit_positions(D5);

        // D5 should be at the placed position
        assert!(positions.contains(pos));

        // D5 should be removed from same row, column, and box
        assert!(!positions.contains(Position::new(0, 4))); // Same row
        assert!(!positions.contains(Position::new(4, 0))); // Same column
        assert!(!positions.contains(Position::new(3, 3))); // Same box

        // But D5 can still be placed in other rows/columns/boxes
        assert!(positions.contains(Position::new(0, 0))); // Different row, column, and box

        // Other digits should be removed from the placed cell
        let positions_d1 = grid.digit_positions(D1);
        assert!(!positions_d1.contains(pos)); // Cell itself removed
        assert!(positions_d1.contains(Position::new(0, 4))); // Same row is OK
        assert!(positions_d1.contains(Position::new(4, 0))); // Same column is OK
        assert!(positions_d1.contains(Position::new(3, 3))); // Same box is OK
    }

    #[test]
    fn test_candidates_at_full_position() {
        let grid = CandidateGrid::new();
        let candidates = grid.candidates_at(Position::new(4, 4));
        assert_eq!(candidates.len(), 9);
    }

    #[test]
    fn test_candidates_at_with_removed_digits() {
        let mut board = CandidateGrid::new();
        let pos = Position::new(5, 5);

        // Remove digits 1, 3, 5, 7, 9 (keep 2, 4, 6, 8)
        for digit in [D1, D3, D5, D7, D9] {
            board.remove_candidate(pos, digit);
        }

        let candidates = board.candidates_at(pos);
        assert_eq!(candidates.len(), 4);
        assert!(candidates.contains(D2));
        assert!(candidates.contains(D4));
        assert!(candidates.contains(D6));
        assert!(candidates.contains(D8));
    }

    #[test]
    fn test_row_mask_full() {
        let grid = CandidateGrid::new();
        let mask = grid.row_mask(0, D1);
        assert_eq!(mask.len(), 9);
    }

    #[test]
    fn test_row_mask_with_candidates() {
        let mut board = CandidateGrid::new();

        // Remove digit 3 from all positions in row 2 except (1, 2), (3, 2), (5, 2)
        for pos in Position::ROWS[2] {
            if pos.x() != 1 && pos.x() != 3 && pos.x() != 5 {
                board.remove_candidate(pos, D3);
            }
        }

        let mask = board.row_mask(2, D3);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(1));
        assert!(mask.contains(3));
        assert!(mask.contains(5));
    }

    #[test]
    fn test_col_mask_full() {
        let grid = CandidateGrid::new();
        let mask = grid.col_mask(0, D1);
        assert_eq!(mask.len(), 9);
    }

    #[test]
    fn test_col_mask_with_candidates() {
        let mut board = CandidateGrid::new();

        // Remove digit 9 from all positions in column 4 except (4, 0), (4, 4), (4, 8)
        for pos in Position::COLUMNS[4] {
            if pos.y() != 0 && pos.y() != 4 && pos.y() != 8 {
                board.remove_candidate(pos, D9);
            }
        }

        let mask = board.col_mask(4, D9);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(0));
        assert!(mask.contains(4));
        assert!(mask.contains(8));
    }

    #[test]
    fn test_box_mask_full() {
        let grid = CandidateGrid::new();
        let mask = grid.box_mask(4, D1);
        assert_eq!(mask.len(), 9);
    }

    #[test]
    fn test_box_mask_with_candidates() {
        let mut board = CandidateGrid::new();

        // Remove digit 6 from all positions in box 8 except cells 0, 4, 8
        for pos in Position::BOXES[8] {
            let cell_idx = pos.box_cell_index();
            if cell_idx != 0 && cell_idx != 4 && cell_idx != 8 {
                board.remove_candidate(pos, D6);
            }
        }

        let mask = board.box_mask(8, D6);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(0));
        assert!(mask.contains(4));
        assert!(mask.contains(8));
    }

    #[test]
    fn test_hidden_single_in_row() {
        let mut grid = CandidateGrid::new();

        // Set up: digit 5 can only go in position (3, 0) in row 0
        for pos in Position::ROWS[0] {
            if pos.x() != 3 {
                grid.remove_candidate(pos, D5);
            }
        }

        let mask = grid.row_mask(0, D5);
        assert_eq!(mask.len(), 1, "Hidden single detected: only one candidate");
        assert!(mask.contains(3)); // x=3 is the only position
    }

    #[test]
    fn test_hidden_single_in_column() {
        let mut grid = CandidateGrid::new();

        // Set up: digit 7 can only go in position (5, 4) in column 5
        for pos in Position::COLUMNS[5] {
            if pos.y() != 4 {
                grid.remove_candidate(pos, D7);
            }
        }

        let mask = grid.col_mask(5, D7);
        assert_eq!(mask.len(), 1, "Hidden single detected: only one candidate");
        assert!(mask.contains(4)); // y=4 is the only position
    }

    #[test]
    fn test_hidden_single_in_box() {
        let mut grid = CandidateGrid::new();

        // Set up: digit 9 can only go in position (4, 4) (center of box 4)
        for pos in Position::BOXES[4] {
            if pos.box_cell_index() != 4 {
                grid.remove_candidate(pos, D9);
            }
        }

        let mask = grid.box_mask(4, D9);
        assert_eq!(mask.len(), 1, "Hidden single detected: only one candidate");
        assert!(mask.contains(4)); // cell_index=4 is the center of the box
    }

    #[test]
    fn test_board_clone() {
        let mut board1 = CandidateGrid::new();
        board1.digit_positions[D1].insert(Position::new(0, 0));

        let board2 = board1.clone();

        assert_eq!(board1, board2);
    }

    #[test]
    fn test_board_default() {
        let board = CandidateGrid::default();

        // Default should be same as new() - all candidates available
        for pos in Position::ALL {
            assert_eq!(board.candidates_at(pos).len(), 9);
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
    fn test_check_consistency_empty_grid() {
        let grid = CandidateGrid::new();
        assert!(grid.check_consistency().is_ok());
    }

    #[test]
    fn test_check_consistency_after_placement() {
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D5);
        assert!(grid.check_consistency().is_ok());
    }

    #[test]
    fn test_check_consistency_with_empty_cell() {
        let mut grid = CandidateGrid::new();
        let pos = Position::new(4, 4);
        // Remove all candidates from a position
        for digit in Digit::ALL {
            grid.remove_candidate(pos, digit);
        }
        assert!(grid.check_consistency().is_err());
    }

    #[test]
    fn test_check_consistency_with_duplicate() {
        let mut grid = CandidateGrid::new();
        // Place the same digit twice in the same row using place_no_propagation
        grid.place_no_propagation(Position::new(0, 0), D5);
        grid.place_no_propagation(Position::new(0, 1), D5);
        assert!(grid.check_consistency().is_err());
    }

    #[test]
    fn test_is_solved_empty_grid() {
        let grid = CandidateGrid::new();
        assert!(!grid.is_solved().unwrap());
    }

    #[test]
    fn test_is_solved_partially_filled() {
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D1);
        grid.place(Position::new(1, 1), D2);
        assert!(!grid.is_solved().unwrap());
    }

    #[test]
    fn test_is_solved_returns_error_on_contradiction() {
        let mut grid = CandidateGrid::new();
        let pos = Position::new(0, 0);
        // Remove all candidates to create a contradiction
        for digit in Digit::ALL {
            grid.remove_candidate(pos, digit);
        }
        assert!(grid.is_solved().is_err());
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
    fn test_place_no_propagation() {
        let mut grid = CandidateGrid::new();
        let pos = Position::new(4, 4);

        // Place digit 5 at center without propagation
        let changed = grid.place_no_propagation(pos, D5);
        assert!(changed, "Placing should change the grid");

        // The position should only have digit 5
        let candidates = grid.candidates_at(pos);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(D5));

        // But D5 should still be a candidate in the same row (redundant)
        let candidates = grid.candidates_at(Position::new(0, 4));
        assert_eq!(candidates.len(), 9, "All candidates should remain");
        assert!(candidates.contains(D5));

        // And in the same column
        let candidates = grid.candidates_at(Position::new(4, 0));
        assert_eq!(candidates.len(), 9, "All candidates should remain");
        assert!(candidates.contains(D5));

        // And in the same box
        let candidates = grid.candidates_at(Position::new(3, 3));
        assert_eq!(candidates.len(), 9, "All candidates should remain");
        assert!(candidates.contains(D5));

        // And other digits should still be candidates at the same position? No, we remove them
        // Actually, we DO remove other digits from the placed position
        let candidates = grid.candidates_at(pos);
        assert!(!candidates.contains(D1));
        assert!(!candidates.contains(D2));
    }

    #[test]
    fn test_place_no_propagation_removes_other_digits_at_position() {
        let mut grid = CandidateGrid::new();
        let pos = Position::new(2, 2);

        // Place digit 7 without propagation
        grid.place_no_propagation(pos, D7);

        // Only D7 should be at the position
        let candidates = grid.candidates_at(pos);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(D7));

        // All other digits should be removed from this position
        for digit in Digit::ALL {
            if digit != D7 {
                assert!(!candidates.contains(digit));
            }
        }
    }

    #[test]
    fn test_place_vs_place_no_propagation() {
        let mut grid_propagated = CandidateGrid::new();
        let mut grid_not_propagated = CandidateGrid::new();
        let pos = Position::new(0, 0);

        grid_propagated.place(pos, D3);
        grid_not_propagated.place_no_propagation(pos, D3);

        // Both should have D3 at (0, 0)
        assert_eq!(grid_propagated.candidates_at(pos).len(), 1);
        assert_eq!(grid_not_propagated.candidates_at(pos).len(), 1);

        // With propagation, D3 is removed from same row
        let candidates = grid_propagated.candidates_at(Position::new(1, 0));
        assert!(!candidates.contains(D3));

        // Without propagation, D3 remains in same row
        let candidates = grid_not_propagated.candidates_at(Position::new(1, 0));
        assert!(candidates.contains(D3));
    }

    #[test]
    fn test_from_digit_grid() {
        use std::str::FromStr;
        let digit_grid = crate::DigitGrid::from_str(
            "5________ _________ _________ _________ _________ _________ _________ _________ _________",
        )
        .unwrap();

        let grid = CandidateGrid::from_digit_grid(&digit_grid);

        // D5 is placed at (0, 0)
        let candidates = grid.candidates_at(Position::new(0, 0));
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(D5));

        // D5 is NOT a candidate in the same row (propagated)
        let candidates = grid.candidates_at(Position::new(1, 0));
        assert!(!candidates.contains(D5));

        // D5 is NOT a candidate in the same column (propagated)
        let candidates = grid.candidates_at(Position::new(0, 1));
        assert!(!candidates.contains(D5));

        // D5 is NOT a candidate in the same box (propagated)
        let candidates = grid.candidates_at(Position::new(1, 1));
        assert!(!candidates.contains(D5));
    }

    #[test]
    fn test_from_digit_grid_no_propagation() {
        use std::str::FromStr;
        let digit_grid = crate::DigitGrid::from_str(
            "5________ _________ _________ _________ _________ _________ _________ _________ _________",
        )
        .unwrap();

        let grid = CandidateGrid::from_digit_grid_no_propagation(&digit_grid);

        // D5 is placed at (0, 0)
        let candidates = grid.candidates_at(Position::new(0, 0));
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(D5));

        // D5 is still a candidate in the same row (not propagated)
        let candidates = grid.candidates_at(Position::new(1, 0));
        assert!(candidates.contains(D5));

        // D5 is still a candidate in the same column (not propagated)
        let candidates = grid.candidates_at(Position::new(0, 1));
        assert!(candidates.contains(D5));

        // D5 is still a candidate in the same box (not propagated)
        let candidates = grid.candidates_at(Position::new(1, 1));
        assert!(candidates.contains(D5));
    }

    #[test]
    fn test_from_digit_grid_with_multiple_digits() {
        use std::str::FromStr;
        let digit_grid = crate::DigitGrid::from_str(
            "53_______ 6________ _________ _________ _________ _________ _________ _________ _________",
        )
        .unwrap();

        let grid = CandidateGrid::from_digit_grid(&digit_grid);

        // D5 is placed at (0, 0)
        let candidates = grid.candidates_at(Position::new(0, 0));
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(D5));

        // D3 is placed at (1, 0)
        let candidates = grid.candidates_at(Position::new(1, 0));
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(D3));

        // D6 is placed at (0, 1)
        let candidates = grid.candidates_at(Position::new(0, 1));
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(D6));

        // At (2, 0): D5 and D3 removed (same row), D6 removed (same box)
        let candidates = grid.candidates_at(Position::new(2, 0));
        assert!(!candidates.contains(D5));
        assert!(!candidates.contains(D3));
        assert!(!candidates.contains(D6));

        // At (3, 0): D5 and D3 removed (same row), but D6 remains (different box)
        let candidates = grid.candidates_at(Position::new(3, 0));
        assert!(!candidates.contains(D5));
        assert!(!candidates.contains(D3));
        assert!(candidates.contains(D6));
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
    fn test_digit_positions_row_mask() {
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
        assert!(!mask.contains(1));

        let mask = positions.row_mask(1);
        assert_eq!(mask.len(), 1);
        assert!(mask.contains(1));

        let mask = positions.row_mask(2);
        assert_eq!(mask.len(), 0);
    }

    #[test]
    fn test_digit_positions_col_mask() {
        let mut positions = DigitPositions::new();
        positions.insert(Position::new(0, 0));
        positions.insert(Position::new(0, 2));
        positions.insert(Position::new(0, 5));
        positions.insert(Position::new(1, 1));

        let mask = positions.col_mask(0);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(0));
        assert!(mask.contains(2));
        assert!(mask.contains(5));
        assert!(!mask.contains(1));

        let mask = positions.col_mask(1);
        assert_eq!(mask.len(), 1);
        assert!(mask.contains(1));

        let mask = positions.col_mask(2);
        assert_eq!(mask.len(), 0);
    }

    #[test]
    fn test_digit_positions_box_mask() {
        let mut positions = DigitPositions::new();
        // Box 0 (top-left 3×3)
        positions.insert(Position::new(0, 0)); // Box index 0
        positions.insert(Position::new(1, 1)); // Box index 4
        positions.insert(Position::new(2, 2)); // Box index 8
        // Box 1 (top-center 3×3)
        positions.insert(Position::new(3, 0)); // Box index 0

        let mask = positions.box_mask(0);
        assert_eq!(mask.len(), 3);
        assert!(mask.contains(0));
        assert!(mask.contains(4));
        assert!(mask.contains(8));

        let mask = positions.box_mask(1);
        assert_eq!(mask.len(), 1);
        assert!(mask.contains(0));

        let mask = positions.box_mask(2);
        assert_eq!(mask.len(), 0);
    }

    #[test]
    fn test_digit_positions_row_masks_constant() {
        // Test that ROW_MASKS contains the correct positions for each row
        for y in 0..9 {
            let row_mask = DigitPositions::ROW_POSITIONS[y];
            assert_eq!(row_mask.len(), 9, "Row {y} should have 9 positions");

            // Check that all positions in the row are present
            for x in 0..9 {
                let pos = Position::new(x, y);
                assert!(
                    row_mask.contains(pos),
                    "Row {y} mask should contain position ({x}, {y})"
                );
            }

            // Check that positions from other rows are not present
            for other_y in 0..9 {
                if other_y != y {
                    for x in 0..9 {
                        let pos = Position::new(x, other_y);
                        assert!(
                            !row_mask.contains(pos),
                            "Row {y} mask should not contain position ({x}, {other_y})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_digit_positions_col_masks_constant() {
        // Test that COL_MASKS contains the correct positions for each column
        for x in 0..9 {
            let col_mask = DigitPositions::COLUMN_POSITIONS[x];
            assert_eq!(col_mask.len(), 9, "Column {x} should have 9 positions");

            // Check that all positions in the column are present
            for y in 0..9 {
                let pos = Position::new(x, y);
                assert!(
                    col_mask.contains(pos),
                    "Column {x} mask should contain position ({x}, {y})"
                );
            }

            // Check that positions from other columns are not present
            for other_x in 0..9 {
                if other_x != x {
                    for y in 0..9 {
                        let pos = Position::new(other_x, y);
                        assert!(
                            !col_mask.contains(pos),
                            "Column {x} mask should not contain position ({other_x}, {y})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_digit_positions_box_masks_constant() {
        // Test that BOX_MASKS contains the correct positions for each box
        for box_index in 0..9 {
            let box_mask = DigitPositions::BOX_POSITIONS[box_index];
            assert_eq!(box_mask.len(), 9, "Box {box_index} should have 9 positions");

            // Check that all positions in the box are present
            for cell_index in 0..9 {
                let pos = Position::from_box(box_index, cell_index);
                assert!(
                    box_mask.contains(pos),
                    "Box {box_index} mask should contain position at cell index {cell_index}"
                );
            }

            // Check that positions from other boxes are not present
            for other_box in 0..9 {
                if other_box != box_index {
                    for cell_index in 0..9 {
                        let pos = Position::from_box(other_box, cell_index);
                        assert!(
                            !box_mask.contains(pos),
                            "Box {box_index} mask should not contain position from box {other_box}"
                        );
                    }
                }
            }
        }
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

    #[test]
    fn test_to_digit_grid_empty() {
        let grid = CandidateGrid::new();
        let digit_grid = grid.to_digit_grid();

        // Empty candidate grid should produce empty digit grid
        for pos in Position::ALL {
            assert_eq!(digit_grid.get(pos), None);
        }
    }

    #[test]
    fn test_to_digit_grid_single_placement() {
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D1);

        let digit_grid = grid.to_digit_grid();

        // Only the placed digit should appear in the digit grid
        assert_eq!(digit_grid.get(Position::new(0, 0)), Some(D1));

        // All other cells should be empty
        for pos in Position::ALL {
            if pos != Position::new(0, 0) {
                assert_eq!(digit_grid.get(pos), None);
            }
        }
    }

    #[test]
    fn test_to_digit_grid_multiple_placements() {
        let mut grid = CandidateGrid::new();
        grid.place(Position::new(0, 0), D1);
        grid.place(Position::new(1, 0), D2);
        grid.place(Position::new(2, 1), D3);
        grid.place(Position::new(4, 4), D5);

        let digit_grid = grid.to_digit_grid();

        // Check all placed digits appear
        assert_eq!(digit_grid.get(Position::new(0, 0)), Some(D1));
        assert_eq!(digit_grid.get(Position::new(1, 0)), Some(D2));
        assert_eq!(digit_grid.get(Position::new(2, 1)), Some(D3));
        assert_eq!(digit_grid.get(Position::new(4, 4)), Some(D5));

        // Check some unplaced cells are empty
        assert_eq!(digit_grid.get(Position::new(3, 0)), None);
        assert_eq!(digit_grid.get(Position::new(8, 8)), None);
    }

    #[test]
    fn test_to_digit_grid_with_undecided_cells() {
        let mut grid = CandidateGrid::new();
        // Place some digits
        grid.place(Position::new(0, 0), D1);

        // Manually remove some candidates to create cells with multiple candidates
        // but don't reduce to a single candidate
        grid.remove_candidate(Position::new(1, 1), D1);
        grid.remove_candidate(Position::new(1, 1), D2);
        // Position (1, 1) now has 7 candidates, not decided

        let digit_grid = grid.to_digit_grid();

        // Only the fully decided cell should appear
        assert_eq!(digit_grid.get(Position::new(0, 0)), Some(D1));
        // The cell with multiple candidates should be empty in digit grid
        assert_eq!(digit_grid.get(Position::new(1, 1)), None);
    }

    #[test]
    fn test_to_digit_grid_roundtrip_compatibility() {
        use std::str::FromStr;

        // Start with a digit grid with some placements
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
        let original_digit_grid = DigitGrid::from_str(digit_grid_str).unwrap();

        // Convert to candidate grid
        let candidate_grid = CandidateGrid::from(original_digit_grid);

        // Convert back to digit grid
        let result_digit_grid = candidate_grid.to_digit_grid();

        // Should match the original
        assert_eq!(result_digit_grid.get(Position::new(0, 0)), Some(D1));
        assert_eq!(result_digit_grid.get(Position::new(1, 1)), Some(D2));
        assert_eq!(result_digit_grid.get(Position::new(2, 2)), Some(D3));
        assert_eq!(result_digit_grid.get(Position::new(3, 3)), Some(D4));
        assert_eq!(result_digit_grid.get(Position::new(4, 4)), Some(D5));
        assert_eq!(result_digit_grid.get(Position::new(5, 5)), Some(D6));
        assert_eq!(result_digit_grid.get(Position::new(6, 6)), Some(D7));
        assert_eq!(result_digit_grid.get(Position::new(7, 7)), Some(D8));
        assert_eq!(result_digit_grid.get(Position::new(8, 8)), Some(D9));
    }
}
