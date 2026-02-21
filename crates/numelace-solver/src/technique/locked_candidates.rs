use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, House, Position};

use super::{BoxedTechnique, BoxedTechniqueStep, TechniqueGrid};
use crate::{
    SolverError,
    technique::{Technique, TechniqueStepData},
};

const NAME: &str = "Locked Candidates";
const NAME_POINTING: &str = "Locked Candidates (Pointing)";
const NAME_CLAIMING: &str = "Locked Candidates (Claiming)";

/// A technique that removes candidates using locked candidates (pointing/claiming).
///
/// - **Pointing**: Within a box, all candidates of a digit lie in a single row/column,
///   so that digit can be removed from the rest of that row/column outside the box.
/// - **Claiming**: Within a row/column, all candidates of a digit lie in a single box,
///   so that digit can be removed from the rest of that box outside the row/column.
///
/// # Examples
///
/// ```
/// use numelace_solver::{
///     TechniqueGrid,
///     technique::{LockedCandidates, Technique},
/// };
///
/// let mut grid = TechniqueGrid::new();
/// let technique = LockedCandidates::new();
///
/// // Apply the technique
/// let changed = technique.apply(&mut grid)?;
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct LockedCandidates {}

impl LockedCandidates {
    /// Creates a new `LockedCandidates` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LockedCandidatesKind {
    Pointing,
    Claiming,
}

impl LockedCandidates {
    #[inline]
    fn apply_with_control_flow<F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Option<BoxedTechniqueStep>
    where
        F: for<'a> FnMut(
            &'a mut TechniqueGrid,
            LockedCandidatesKind,
            Digit,
            u8,
            House,
            DigitPositions,
        ) -> ControlFlow<BoxedTechniqueStep>,
    {
        let decided_cells = grid.decided_cells();
        for box_index in 0..9 {
            let box_ = House::Box { index: box_index };
            let origin = Position::box_origin(box_index);
            let row_or_cols = [
                House::Row { y: origin.y() },
                House::Row { y: origin.y() + 1 },
                House::Row { y: origin.y() + 2 },
                House::Column { x: origin.x() },
                House::Column { x: origin.x() + 1 },
                House::Column { x: origin.x() + 2 },
            ];

            for row_or_col in row_or_cols {
                let intersection = box_.positions() & row_or_col.positions();
                if (intersection & !decided_cells).is_empty() {
                    continue;
                }

                let rest_in_box = box_.positions() & !intersection;
                let rest_in_row_or_col = row_or_col.positions() & !intersection;
                for digit in Digit::ALL {
                    let undecided_positions = grid.digit_positions(digit) & !decided_cells;
                    if (undecided_positions & intersection).is_empty() {
                        continue;
                    }
                    if (undecided_positions & rest_in_box).is_empty() {
                        // Pointing
                        let eliminations = undecided_positions & rest_in_row_or_col;
                        if grid.remove_candidate_with_mask(eliminations, digit)
                            && let ControlFlow::Break(value) = on_condition(
                                grid,
                                LockedCandidatesKind::Pointing,
                                digit,
                                box_index,
                                row_or_col,
                                undecided_positions & intersection,
                            )
                        {
                            return Some(value);
                        }
                    } else if (undecided_positions & rest_in_row_or_col).is_empty() {
                        // Claiming
                        let eliminations = undecided_positions & rest_in_box;
                        if grid.remove_candidate_with_mask(eliminations, digit)
                            && let ControlFlow::Break(value) = on_condition(
                                grid,
                                LockedCandidatesKind::Claiming,
                                digit,
                                box_index,
                                row_or_col,
                                undecided_positions & intersection,
                            )
                        {
                            return Some(value);
                        }
                    }
                }
            }
        }
        None
    }
}

impl Technique for LockedCandidates {
    fn name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        let mut after_grid = grid.clone();
        let step = Self::apply_with_control_flow(
            &mut after_grid,
            |after_grid, kind, digit, box_index, row_or_col, intersection| {
                ControlFlow::Break(Box::new(TechniqueStepData::from_diff(
                    match kind {
                        LockedCandidatesKind::Pointing => NAME_POINTING,
                        LockedCandidatesKind::Claiming => NAME_CLAIMING,
                    },
                    House::Box { index: box_index }.positions() | row_or_col.positions(),
                    vec![(intersection, DigitSet::from_elem(digit))],
                    grid,
                    after_grid,
                )))
            },
        );
        Ok(step)
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let mut changed = false;
        Self::apply_with_control_flow(grid, |_, _, _, _, _, _| {
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
    fn test_pointing_eliminates_from_row() {
        // Box 0 (rows 0-2, cols 0-2): limit D5 candidates to row 0 inside the box.
        let mut grid = CandidateGrid::new();
        for pos in Position::BOXES[0] {
            if pos.y() != 0 {
                grid.remove_candidate(pos, Digit::D5);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&LockedCandidates::new())
            // D5 removed from the rest of row 0 outside the box.
            .assert_removed_includes(Position::new(3, 0), [Digit::D5])
            .assert_removed_includes(Position::new(8, 0), [Digit::D5]);
    }

    #[test]
    fn test_claiming_eliminates_from_box() {
        // Row 0: limit D7 candidates to box 0 cells in row 0.
        let mut grid = CandidateGrid::new();
        for pos in Position::ROWS[0] {
            if pos.x() > 2 {
                grid.remove_candidate(pos, Digit::D7);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&LockedCandidates::new())
            // D7 removed from the rest of box 0 outside row 0.
            .assert_removed_includes(Position::new(0, 1), [Digit::D7])
            .assert_removed_includes(Position::new(2, 2), [Digit::D7]);
    }

    #[test]
    fn test_no_change_when_no_locked_candidates() {
        // A fresh grid has no locked candidate eliminations.
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&LockedCandidates::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }
}
