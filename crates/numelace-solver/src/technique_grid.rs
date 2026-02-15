use numelace_core::{
    CandidateGrid, ConsistencyError, Digit, DigitGrid, DigitPositions, DigitSet, House, HouseMask,
    Position,
};

/// Solver state for technique-based solving.
///
/// This type wraps a [`CandidateGrid`] and exposes a solver-oriented API for
/// applying techniques without leaking direct candidate access. It is designed
/// to keep technique logic focused on domain operations (place/remove/query)
/// while centralizing solver bookkeeping such as propagation tracking.
///
/// # Design Notes
///
/// - `TechniqueGrid` is the only surface used by techniques to mutate candidates.
/// - Tracking fields (like `decided_propagated`) live here to keep technique logic
///   simple and consistent.
/// - Conversions from [`DigitGrid`] and [`CandidateGrid`] exist to support solver
///   entry points and test setups.
///
/// # Examples
///
/// ```
/// use numelace_solver::TechniqueGrid;
///
/// let mut grid = TechniqueGrid::new();
/// // grid methods are used by techniques to place digits or remove candidates.
/// # let _ = grid;
/// ```
#[derive(Debug, Clone)]
pub struct TechniqueGrid {
    /// Underlying candidate state.
    candidates: CandidateGrid,
    /// Decided cells that have already had their peer eliminations applied.
    decided_propagated: DigitPositions,
}

impl From<DigitGrid> for TechniqueGrid {
    fn from(grid: DigitGrid) -> Self {
        CandidateGrid::from(grid).into()
    }
}

impl From<CandidateGrid> for TechniqueGrid {
    fn from(candidates: CandidateGrid) -> Self {
        Self {
            candidates,
            decided_propagated: DigitPositions::EMPTY,
        }
    }
}

impl Default for TechniqueGrid {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TechniqueGrid {
    /// Creates an empty technique grid with all candidates available.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::from(CandidateGrid::new())
    }

    /// Builds a technique grid from a digit grid.
    ///
    /// This is a convenience wrapper around
    /// [`CandidateGrid::from_digit_grid`].
    #[inline]
    #[must_use]
    pub fn from_digit_grid(grid: &DigitGrid) -> Self {
        Self::from(CandidateGrid::from_digit_grid(grid))
    }

    /// Consumes the wrapper and returns the underlying candidate grid.
    ///
    /// This is intended for interoperability with APIs that operate directly
    /// on [`CandidateGrid`].
    #[inline]
    #[must_use]
    pub fn into_candidates(self) -> CandidateGrid {
        self.candidates
    }

    /// Returns a digit grid containing only decided cells.
    ///
    /// Undecided cells are left empty in the returned grid.
    ///
    /// This mirrors [`CandidateGrid::to_digit_grid`].
    #[inline]
    #[must_use]
    pub fn to_digit_grid(&self) -> DigitGrid {
        self.candidates.to_digit_grid()
    }

    /// Places a digit at a position by removing all other candidates at that cell.
    ///
    /// This does not propagate eliminations to peers.
    ///
    /// This mirrors [`CandidateGrid::place`].
    #[inline]
    pub fn place(&mut self, pos: Position, digit: Digit) -> bool {
        self.candidates.place(pos, digit)
    }

    /// Returns `true` if placing the digit would change the grid.
    ///
    /// This mirrors [`CandidateGrid::would_place_change`].
    #[inline]
    #[must_use]
    pub fn would_place_change(&self, pos: Position, digit: Digit) -> bool {
        self.candidates.would_place_change(pos, digit)
    }

    /// Removes a specific digit as a candidate at a position.
    ///
    /// Returns `true` if the candidate was removed.
    ///
    /// This mirrors [`CandidateGrid::remove_candidate`].
    #[inline]
    pub fn remove_candidate(&mut self, pos: Position, digit: Digit) -> bool {
        self.candidates.remove_candidate(pos, digit)
    }

    /// Returns `true` if removing the candidate would change the grid.
    ///
    /// This mirrors [`CandidateGrid::would_remove_candidate_change`].
    #[inline]
    #[must_use]
    pub fn would_remove_candidate_change(&self, pos: Position, digit: Digit) -> bool {
        self.candidates.would_remove_candidate_change(pos, digit)
    }

    /// Removes a candidate digit from all positions specified by a mask.
    ///
    /// Returns `true` if any candidate was removed.
    ///
    /// This mirrors [`CandidateGrid::remove_candidate_with_mask`].
    #[inline]
    pub fn remove_candidate_with_mask(&mut self, mask: DigitPositions, digit: Digit) -> bool {
        self.candidates.remove_candidate_with_mask(mask, digit)
    }

    /// Returns `true` if removing the digit from the masked positions would change the grid.
    ///
    /// This mirrors [`CandidateGrid::would_remove_candidate_with_mask_change`].
    #[inline]
    #[must_use]
    pub fn would_remove_candidate_with_mask_change(
        &self,
        mask: DigitPositions,
        digit: Digit,
    ) -> bool {
        self.candidates
            .would_remove_candidate_with_mask_change(mask, digit)
    }

    /// Returns the set of all positions where the specified digit can be placed.
    ///
    /// This mirrors [`CandidateGrid::digit_positions`].
    #[inline]
    #[must_use]
    pub fn digit_positions(&self, digit: Digit) -> DigitPositions {
        self.candidates.digit_positions(digit)
    }

    /// Returns the set of candidate digits that can be placed at a position.
    ///
    /// This mirrors [`CandidateGrid::candidates_at`].
    #[inline]
    #[must_use]
    pub fn candidates_at(&self, pos: Position) -> DigitSet {
        self.candidates.candidates_at(pos)
    }

    /// Returns a bitmask of candidate positions in the specified house for the digit.
    ///
    /// This mirrors [`CandidateGrid::house_mask`].
    #[inline]
    #[must_use]
    pub fn house_mask(&self, house: House, digit: Digit) -> HouseMask {
        self.candidates.house_mask(house, digit)
    }

    /// Returns a bitmask of candidate positions in the specified row for the digit.
    ///
    /// This mirrors [`CandidateGrid::row_mask`].
    #[inline]
    #[must_use]
    pub fn row_mask(&self, y: u8, digit: Digit) -> HouseMask {
        self.candidates.row_mask(y, digit)
    }

    /// Returns a bitmask of candidate positions in the specified column for the digit.
    ///
    /// This mirrors [`CandidateGrid::col_mask`].
    #[inline]
    #[must_use]
    pub fn col_mask(&self, x: u8, digit: Digit) -> HouseMask {
        self.candidates.col_mask(x, digit)
    }

    /// Returns a bitmask of candidate positions in the specified box for the digit.
    ///
    /// This mirrors [`CandidateGrid::box_mask`].
    #[inline]
    #[must_use]
    pub fn box_mask(&self, box_index: u8, digit: Digit) -> HouseMask {
        self.candidates.box_mask(box_index, digit)
    }

    /// Checks whether the candidate grid is consistent.
    ///
    /// This mirrors [`CandidateGrid::check_consistency`].
    ///
    /// # Errors
    ///
    /// Returns [`ConsistencyError`] if the grid contains contradictions.
    #[inline]
    pub fn check_consistency(&self) -> Result<(), ConsistencyError> {
        self.candidates.check_consistency()
    }

    /// Returns whether the candidate grid is fully solved.
    ///
    /// This mirrors [`CandidateGrid::is_solved`].
    ///
    /// # Errors
    ///
    /// Returns [`ConsistencyError`] if the grid contains contradictions.
    #[inline]
    pub fn is_solved(&self) -> Result<bool, ConsistencyError> {
        self.candidates.is_solved()
    }

    /// Returns all positions that have exactly one candidate (decided cells).
    ///
    /// This mirrors [`CandidateGrid::decided_cells`].
    #[inline]
    #[must_use]
    pub fn decided_cells(&self) -> DigitPositions {
        self.candidates.decided_cells()
    }

    /// Classifies all grid positions by candidate count.
    ///
    /// This mirrors [`CandidateGrid::classify_cells`].
    #[inline]
    #[must_use]
    pub fn classify_cells<const N: usize>(&self) -> [DigitPositions; N] {
        self.candidates.classify_cells()
    }

    /// Returns the set of decided cells that have already been propagated.
    #[inline]
    #[must_use]
    pub fn decided_propagated(&self) -> DigitPositions {
        self.decided_propagated
    }

    /// Marks a decided cell as having its peer eliminations applied.
    #[inline]
    pub fn insert_decided_propagated(&mut self, pos: Position) {
        self.decided_propagated.insert(pos);
    }
}
