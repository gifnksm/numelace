use sudoku_core::{CandidateGrid, Digit, Position};

use crate::{SolverError, technique::Technique};

use super::BoxedTechnique;

#[derive(Debug, Default, Clone, Copy)]
pub struct HiddenSingle;

impl HiddenSingle {
    pub const fn new() -> Self {
        HiddenSingle
    }
}

impl Technique for HiddenSingle {
    fn name(&self) -> &'static str {
        "hidden singles"
    }

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn apply(&self, grid: &mut CandidateGrid) -> Result<bool, SolverError> {
        let mut changed = false;

        for digit in Digit::ALL {
            for y in 0..9 {
                let row = grid.get_row(y, digit);
                if row.len() == 1 {
                    let x = row.first().unwrap();
                    changed |= grid.place(Position::new(x, y), digit);
                }
            }

            for x in 0..9 {
                let col = grid.get_col(x, digit);
                if col.len() == 1 {
                    let y = col.first().unwrap();
                    changed |= grid.place(Position::new(x, y), digit);
                }
            }

            for box_index in 0..9 {
                let block = grid.get_box(box_index, digit);
                if block.len() == 1 {
                    let i = block.first().unwrap();
                    changed |= grid.place(Position::from_box(box_index, i), digit);
                }
            }
        }

        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use sudoku_core::{CandidateGrid, Digit, Position};

    use crate::testing::TechniqueTester;

    use super::*;

    #[test]
    fn test_hidden_single_in_row() {
        // When a digit can only go in one position in a row, it's a hidden single
        let mut grid = CandidateGrid::new();

        // Remove D5 from all cells in row 0 except (3, 0)
        for x in 0..9 {
            if x != 3 {
                grid.remove_candidate(Position::new(x, 0), Digit::D5);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenSingle::new())
            // D5 should be placed at (3, 0), removing D5 from:
            .assert_removed_exact(Position::new(3, 1), [Digit::D5]) // same column
            .assert_removed_exact(Position::new(3, 8), [Digit::D5]) // same column, far cell
            .assert_removed_exact(Position::new(4, 1), [Digit::D5]); // same box
    }

    #[test]
    fn test_hidden_single_in_column() {
        // When a digit can only go in one position in a column, it's a hidden single
        let mut grid = CandidateGrid::new();

        // Remove D7 from all cells in column 5 except (5, 4)
        for y in 0..9 {
            if y != 4 {
                grid.remove_candidate(Position::new(5, y), Digit::D7);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenSingle::new())
            // D7 should be placed at (5, 4), removing D7 from:
            .assert_removed_exact(Position::new(4, 4), [Digit::D7]) // same row
            .assert_removed_exact(Position::new(0, 4), [Digit::D7]) // same row, far cell
            .assert_removed_exact(Position::new(3, 3), [Digit::D7]); // same box
    }

    #[test]
    fn test_hidden_single_in_box() {
        // When a digit can only go in one position in a box, it's a hidden single
        let mut grid = CandidateGrid::new();

        // Box 4 is the center box (rows 3-5, columns 3-5)
        // Remove D9 from all cells in box 4 except the center cell (4, 4)
        for i in 0..9 {
            if i != 4 {
                grid.remove_candidate(Position::from_box(4, i), Digit::D9);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenSingle::new())
            // D9 should be placed at (4, 4), removing D9 from:
            .assert_removed_exact(Position::new(4, 0), [Digit::D9]) // same column, different box
            .assert_removed_exact(Position::new(0, 4), [Digit::D9]) // same row, different box
            .assert_removed_exact(Position::new(4, 8), [Digit::D9]); // same column, different box
    }

    #[test]
    fn test_multiple_hidden_singles() {
        // Multiple hidden singles in different regions are all placed
        let mut grid = CandidateGrid::new();

        // Create hidden single in row 0: D3 can only go at (2, 0)
        for x in 0..9 {
            if x != 2 {
                grid.remove_candidate(Position::new(x, 0), Digit::D3);
            }
        }

        // Create hidden single in column 7: D8 can only go at (7, 6)
        for y in 0..9 {
            if y != 6 {
                grid.remove_candidate(Position::new(7, y), Digit::D8);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenSingle::new())
            // D3 placed at (2, 0)
            .assert_removed_exact(Position::new(2, 1), [Digit::D3]) // same column
            // D8 placed at (7, 6)
            .assert_removed_exact(Position::new(6, 6), [Digit::D8]); // same row
    }

    #[test]
    fn test_no_change_when_no_hidden_singles() {
        // When every digit has multiple candidates in each house, nothing changes
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&HiddenSingle::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }
}
