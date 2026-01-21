use sudoku_core::{CandidateGrid, Digit};

use super::BoxedTechnique;
use crate::{SolverError, technique::Technique};

#[derive(Debug, Default, Clone, Copy)]
pub struct NakedSingle;

impl NakedSingle {
    pub const fn new() -> Self {
        NakedSingle
    }
}

impl Technique for NakedSingle {
    fn name(&self) -> &'static str {
        "naked singles"
    }

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn apply(&self, grid: &mut CandidateGrid) -> Result<bool, SolverError> {
        let mut changed = false;

        let decided_cells = grid.decided_cells();
        for digit in Digit::ALL {
            let decided_cells = grid.get_positions(digit) & decided_cells;
            for pos in decided_cells {
                changed |= grid.place(pos, digit);
            }
        }

        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use sudoku_core::{CandidateGrid, Digit, Position};

    use super::*;
    use crate::testing::TechniqueTester;

    #[test]
    fn test_places_naked_single() {
        // When a cell has only one candidate, placing it removes that digit
        // from all cells in the same row, column, and box
        let mut grid = CandidateGrid::new();

        // Make (0, 0) have only D5 as candidate
        for digit in Digit::ALL {
            if digit != Digit::D5 {
                grid.remove_candidate(Position::new(0, 0), digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&NakedSingle::new())
            // D5 removed from same row
            .assert_removed_exact(Position::new(1, 0), [Digit::D5])
            // D5 removed from same column
            .assert_removed_exact(Position::new(0, 1), [Digit::D5])
            // D5 removed from same box
            .assert_removed_exact(Position::new(1, 1), [Digit::D5]);
    }

    #[test]
    fn test_places_multiple_naked_singles() {
        // Multiple naked singles in different regions are all placed
        let mut grid = CandidateGrid::new();

        // Create naked single at (0, 0) with D3
        for digit in Digit::ALL {
            if digit != Digit::D3 {
                grid.remove_candidate(Position::new(0, 0), digit);
            }
        }

        // Create naked single at (5, 5) with D7
        for digit in Digit::ALL {
            if digit != Digit::D7 {
                grid.remove_candidate(Position::new(5, 5), digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&NakedSingle::new())
            // D3 removed from a cell in same row as (0, 0)
            .assert_removed_exact(Position::new(1, 0), [Digit::D3])
            // D7 removed from a cell in same column as (5, 5)
            .assert_removed_exact(Position::new(5, 4), [Digit::D7]);
    }

    #[test]
    fn test_no_change_when_no_naked_singles() {
        // When no cells have a single candidate, nothing changes
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&NakedSingle::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }

    #[test]
    fn test_real_puzzle() {
        // Test with an actual puzzle to verify it works with constraint propagation
        TechniqueTester::from_str(
            "
            53_ _7_ ___
            6__ 195 ___
            _98 ___ _6_
            8__ _6_ __3
            4__ 8_3 __1
            7__ _2_ __6
            _6_ ___ 28_
            ___ 419 __5
            ___ _8_ _79
        ",
        )
        .apply_until_stuck(&NakedSingle::new())
        // After constraint propagation from the initial digits,
        // naked singles should be found and placed.
        // Verify at least one placement occurred by checking candidate removal.
        .assert_removed_includes(Position::new(1, 1), [Digit::D4]);
    }
}
