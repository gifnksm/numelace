use std::ops::ControlFlow;

use numelace_core::{Digit, DigitSet, House, Position};

use crate::{
    SolverError,
    technique::{
        BoxedTechnique, BoxedTechniqueStep, ConditionCells, ConditionDigitCells, Technique,
        TechniqueApplication, TechniqueGrid, TechniqueStep,
    },
};

const NAME: &str = "Hidden Single";

/// A technique that finds digits that can only go in one position within a house.
///
/// A "hidden single" occurs when a digit has only one possible position in a row,
/// column, or box, even though that cell may have multiple candidates. This is
/// slightly more complex than a naked single as it requires checking all positions
/// for a specific digit within a house.
///
/// # Examples
///
/// ```
/// use numelace_solver::{
///     TechniqueGrid,
///     technique::{HiddenSingle, Technique},
/// };
///
/// let mut grid = TechniqueGrid::new();
/// let technique = HiddenSingle::new();
///
/// // Apply the technique
/// let changed = technique.apply(&mut grid)?;
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct HiddenSingle {}

impl HiddenSingle {
    /// Creates a new `HiddenSingle` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone)]
pub struct HiddenSingleStep {
    house: House,
    digit: Digit,
    application: Vec<TechniqueApplication>,
}

impl HiddenSingleStep {
    fn new(house: House, digit: Digit, application: Vec<TechniqueApplication>) -> Self {
        HiddenSingleStep {
            house,
            digit,
            application,
        }
    }
}

impl TechniqueStep for HiddenSingleStep {
    fn technique_name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> BoxedTechniqueStep {
        Box::new(self.clone())
    }

    fn condition_cells(&self) -> ConditionCells {
        self.house.positions()
    }

    fn condition_digit_cells(&self) -> ConditionDigitCells {
        vec![(self.house.positions(), DigitSet::from_elem(self.digit))]
    }

    fn application(&self) -> Vec<TechniqueApplication> {
        self.application.clone()
    }
}

impl HiddenSingle {
    #[inline]
    fn apply_with_control_flow<F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Option<HiddenSingleStep>
    where
        F: for<'a> FnMut(
            &'a mut TechniqueGrid,
            House,
            Position,
            Digit,
        ) -> ControlFlow<HiddenSingleStep>,
    {
        let decided_cells = grid.decided_cells();
        for digit in Digit::ALL {
            let undecided_digit_positions = grid.digit_positions(digit) & !decided_cells;
            for house in House::ALL {
                let house_mask = undecided_digit_positions.house_mask(house);
                if let Some(x) = house_mask.as_single() {
                    let pos = house.position_from_cell_index(x);
                    if grid.place(pos, digit)
                        && let ControlFlow::Break(value) = on_condition(grid, house, pos, digit)
                    {
                        return Some(value);
                    }
                }
            }
        }
        None
    }
}

impl Technique for HiddenSingle {
    fn name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        let mut after_grid = grid.clone();
        let step =
            Self::apply_with_control_flow(&mut after_grid, |after_grid, house, pos, digit| {
                let mut app = super::collect_applications_from_diff(grid, after_grid);
                app.push(TechniqueApplication::Placement {
                    position: pos,
                    digit,
                });
                ControlFlow::Break(HiddenSingleStep::new(house, digit, app))
            });
        Ok(step.map(|step| step.clone_box()))
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let mut changed = false;
        Self::apply_with_control_flow(grid, |_, _, _, _| {
            changed = true;
            ControlFlow::Continue(())
        });
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{CandidateGrid, Digit, Position};

    use super::*;
    use crate::testing::TechniqueTester;

    #[test]
    fn test_hidden_single_in_row() {
        // When a digit can only go in one position in a row, it's a hidden single
        let mut grid = CandidateGrid::new();

        // Remove D5 from all cells in row 0 except (3, 0)
        for pos in Position::ROWS[0] {
            if pos.x() != 3 {
                grid.remove_candidate(pos, Digit::D5);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenSingle::new())
            // D5 should be placed at (3, 0)
            .assert_placed(Position::new(3, 0), Digit::D5);
    }

    #[test]
    fn test_hidden_single_in_column() {
        // When a digit can only go in one position in a column, it's a hidden single
        let mut grid = CandidateGrid::new();

        // Remove D7 from all cells in column 5 except (5, 4)
        for pos in Position::COLUMNS[5] {
            if pos.y() != 4 {
                grid.remove_candidate(pos, Digit::D7);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenSingle::new())
            // D7 should be placed at (5, 4)
            .assert_placed(Position::new(5, 4), Digit::D7);
    }

    #[test]
    fn test_hidden_single_in_box() {
        // When a digit can only go in one position in a box, it's a hidden single
        let mut grid = CandidateGrid::new();

        // Box 4 is the center box (rows 3-5, columns 3-5)
        // Remove D9 from all cells in box 4 except the center cell (4, 4)
        for pos in Position::BOXES[4] {
            if pos.box_cell_index() != 4 {
                grid.remove_candidate(pos, Digit::D9);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenSingle::new())
            // D9 should be placed at (4, 4)
            .assert_placed(Position::new(4, 4), Digit::D9);
    }

    #[test]
    fn test_multiple_hidden_singles() {
        // Multiple hidden singles in different regions are all placed
        let mut grid = CandidateGrid::new();

        // Create hidden single in row 0: D3 can only go at (2, 0)
        for pos in Position::ROWS[0] {
            if pos.x() != 2 {
                grid.remove_candidate(pos, Digit::D3);
            }
        }

        // Create hidden single in column 7: D8 can only go at (7, 6)
        for pos in Position::COLUMNS[7] {
            if pos.y() != 6 {
                grid.remove_candidate(pos, Digit::D8);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenSingle::new())
            // D3 placed at (2, 0)
            .assert_placed(Position::new(2, 0), Digit::D3)
            // D8 placed at (7, 6)
            .assert_placed(Position::new(7, 6), Digit::D8);
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
