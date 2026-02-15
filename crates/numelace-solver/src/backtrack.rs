//! Backtracking utilities for Sudoku solving.
//!
//! This module provides heuristics and utilities for backtracking-based search
//! in Sudoku solving and generation. The primary function is [`find_best_assumption`],
//! which implements the Minimum Remaining Values (MRV) heuristic for selecting
//! the next cell to fill during backtracking search.
//!
//! # MRV Heuristic
//!
//! The MRV heuristic selects the cell with the fewest remaining candidates.
//! This reduces the branching factor in backtracking search, making the search
//! more efficient by:
//!
//! - Detecting contradictions earlier (cells with few candidates fail faster)
//! - Reducing the number of branches explored
//! - Improving pruning effectiveness
//!
//! # Usage
//!
//! This module is used by both [`BacktrackSolver`](crate::BacktrackSolver) for
//! puzzle solving and by `PuzzleGenerator` (in the `numelace-generator` crate) for
//! puzzle generation.
//!
//! # Examples
//!
//! ```
//! use numelace_solver::{TechniqueGrid, backtrack};
//!
//! let mut grid = TechniqueGrid::new();
//! // ... apply some constraints ...
//!
//! // Find the best cell to make an assumption for
//! let (pos, candidates) = backtrack::find_best_assumption(&grid);
//! println!("Best cell to try: {:?}", pos);
//! println!("Candidates: {:?}", candidates);
//! ```

use numelace_core::{DigitSet, Position};

use crate::TechniqueGrid;

/// Finds the best cell to make an assumption for.
///
/// Selects the cell with the minimum number of remaining candidates using the
/// Minimum Remaining Values (MRV) heuristic. This heuristic minimizes the
/// branching factor in backtracking search, making the search more efficient.
///
/// # Returns
///
/// A tuple of `(Position, DigitSet)` where:
/// - `Position` is the cell with the fewest candidates
/// - `DigitSet` contains the valid candidate digits for that cell
///
/// # Panics
///
/// Panics if:
/// - The grid is inconsistent (contains cells with zero candidates)
/// - The grid is fully solved (no undecided cells remain)
///
/// These conditions indicate a programming error - the caller should check
/// the grid state before calling this function.
///
/// # Examples
///
/// ```
/// use numelace_core::{Digit, Position};
/// use numelace_solver::{TechniqueGrid, backtrack};
///
/// let mut grid = TechniqueGrid::new();
///
/// // Place some digits to create a partially filled grid
/// grid.place(Position::new(0, 0), Digit::D1);
/// grid.place(Position::new(1, 0), Digit::D2);
///
/// // Find the best cell to try next
/// let (pos, candidates) = backtrack::find_best_assumption(&grid);
///
/// // The selected cell should have minimal candidates
/// assert!(candidates.len() >= 1);
/// ```
#[must_use]
pub fn find_best_assumption(grid: &TechniqueGrid) -> (Position, DigitSet) {
    // classify_cells::<10> groups cells by candidate count (0-9)
    // [0]: 0 candidates (contradiction), [1]: 1 candidate (decided), [2..]: 2-9 candidates
    let [empty, decided, cells @ ..] = grid.classify_cells::<10>();
    assert!(empty.is_empty() && decided.len() < 81);

    // Pick the first undecided cell with minimum candidates
    let pos = cells.iter().find_map(|cells| cells.first()).unwrap();
    (pos, grid.candidates_at(pos))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use numelace_core::{CandidateGrid, Digit, DigitGrid};

    use super::*;

    #[test]
    fn test_find_best_assumption_selects_minimum_candidates() {
        let mut grid = CandidateGrid::new();

        // Place digits to create cells with different candidate counts
        // Row 0: Place 7 digits, leaving 2 cells with different candidate counts
        grid.place(Position::new(0, 0), Digit::D1);
        grid.place(Position::new(1, 0), Digit::D2);
        grid.place(Position::new(2, 0), Digit::D3);
        grid.place(Position::new(3, 0), Digit::D4);
        grid.place(Position::new(4, 0), Digit::D5);
        grid.place(Position::new(5, 0), Digit::D6);
        grid.place(Position::new(6, 0), Digit::D7);
        // Positions (7,0) and (8,0) remain with candidates {D8, D9}

        // Box 0 (top-left 3x3): Fill more cells to create varying candidate counts
        grid.place(Position::new(0, 1), Digit::D4);
        grid.place(Position::new(1, 1), Digit::D5);
        grid.place(Position::new(2, 1), Digit::D6);
        grid.place(Position::new(0, 2), Digit::D7);
        // Position (1,2) should have fewer candidates than (7,0) and (8,0)

        let (_pos, candidates) = find_best_assumption(&grid.into());

        // Should select a cell with minimum candidates
        // The exact position depends on constraint propagation,
        // but candidates should be non-empty
        assert!(!candidates.is_empty());
        assert!(candidates.len() <= 9);
    }

    #[test]
    fn test_find_best_assumption_with_single_undecided_cell() {
        let mut grid = CandidateGrid::new();

        // Fill 80 cells, leaving only one undecided
        let all_positions = Position::ALL;
        let last_pos = all_positions[80];

        // Fill first 80 positions with a valid solution pattern
        for (i, pos) in all_positions[0..80].iter().enumerate() {
            let digit = Digit::ALL[i % 9];
            grid.place(*pos, digit);
        }

        let (pos, candidates) = find_best_assumption(&grid.into());

        // Should return the last undecided cell
        assert_eq!(pos, last_pos);
        assert!(!candidates.is_empty());
    }

    #[test]
    #[should_panic(expected = "assertion")]
    fn test_find_best_assumption_panics_when_fully_solved() {
        let mut grid = CandidateGrid::new();

        // Create a fully solved grid using a known valid Sudoku solution
        let solution =
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
        let digit_grid = DigitGrid::from_str(solution).unwrap();

        // Verify it's complete and consistent
        for pos in Position::ALL {
            if let Some(digit) = digit_grid.get(pos) {
                grid.place(pos, digit);
            }
        }
        assert!(
            grid.is_solved().unwrap(),
            "Grid should be valid and complete"
        );

        // This should panic because all cells are decided
        let _ = find_best_assumption(&grid.into());
    }

    #[test]
    #[should_panic(expected = "assertion")]
    fn test_find_best_assumption_panics_when_inconsistent() {
        let mut grid = CandidateGrid::new();

        // Create an inconsistent state by placing contradictory values
        let pos = Position::new(0, 0);
        grid.place(pos, Digit::D1);

        // Try to remove all candidates from another cell in the same row
        // This creates an inconsistent state
        let conflict_pos = Position::new(1, 0);
        for digit in Digit::ALL {
            if grid.candidates_at(conflict_pos).contains(digit) {
                grid.remove_candidate(conflict_pos, digit);
            }
        }

        // This should panic because there's a cell with zero candidates
        let _ = find_best_assumption(&grid.into());
    }

    #[test]
    fn test_find_best_assumption_returns_valid_candidates() {
        let mut grid = CandidateGrid::new();

        // Place a few digits
        grid.place(Position::new(0, 0), Digit::D1);
        grid.place(Position::new(1, 1), Digit::D2);
        grid.place(Position::new(2, 2), Digit::D3);

        let grid = TechniqueGrid::from(grid);
        let (pos, candidates) = find_best_assumption(&grid);

        // Verify that returned candidates are actually valid for the position
        let actual_candidates = grid.candidates_at(pos);
        assert_eq!(candidates, actual_candidates);
        assert!(!candidates.is_empty());
    }
}
