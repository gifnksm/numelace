use std::ops::ControlFlow;

use numelace_core::{Digit, DigitSet, House, Position};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
};

const ID: &str = "locked_candidates";
const NAME: &str = "Locked Candidates";
const NAME_POINTING: &str = "Locked Candidates (Pointing)";
const NAME_CLAIMING: &str = "Locked Candidates (Claiming)";

/// A technique that removes candidates using locked candidates (pointing/claiming).
///
/// - **Pointing**: Within a box, all candidates of a digit lie in a single row/column,
///   so that digit can be removed from the rest of that row/column outside the box.
/// - **Claiming**: Within a row/column, all candidates of a digit lie in a single box,
///   so that digit can be removed from the rest of that box outside the row/column.
#[derive(Debug, Default, Clone, Copy)]
pub struct LockedCandidates {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LockedCandidatesKind {
    Pointing,
    Claiming,
}

struct Condition {
    kind: LockedCandidatesKind,
    digit: Digit,
    box_: House,
    line: House,
}

impl Condition {
    fn build_step(
        &self,
        before_grid: &TechniqueGrid,
        after_grid: &TechniqueGrid,
    ) -> BoxedTechniqueStep {
        let condition_positions = self.box_.positions() | self.line.positions();
        let condition_digit_positions = vec![(
            self.box_.positions() & self.line.positions(),
            DigitSet::from_elem(self.digit),
        )];
        TechniqueStepData::from_diff(
            match self.kind {
                LockedCandidatesKind::Pointing => NAME_POINTING,
                LockedCandidatesKind::Claiming => NAME_CLAIMING,
            },
            condition_positions,
            condition_digit_positions,
            before_grid,
            after_grid,
        )
    }
}

impl LockedCandidates {
    /// Creates a new `LockedCandidates` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    fn apply_with_control_flow<T, F>(grid: &mut TechniqueGrid, mut on_condition: F) -> Option<T>
    where
        F: for<'a> FnMut(&'a mut TechniqueGrid, &'a Condition) -> ControlFlow<T>,
    {
        let univalue_positions = grid.univalue_positions();
        for box_index in 0..9 {
            let box_ = House::Box { index: box_index };
            let origin = Position::box_origin(box_index);
            let lines = [
                House::Row { y: origin.y() },
                House::Row { y: origin.y() + 1 },
                House::Row { y: origin.y() + 2 },
                House::Column { x: origin.x() },
                House::Column { x: origin.x() + 1 },
                House::Column { x: origin.x() + 2 },
            ];

            for line in lines {
                let intersection = box_.positions() & line.positions();
                if (intersection & !univalue_positions).is_empty() {
                    continue;
                }

                let rest_in_box = box_.positions() & !intersection;
                let rest_in_line = line.positions() & !intersection;
                for digit in Digit::ALL {
                    let undecided_positions = grid.digit_positions(digit) & !univalue_positions;
                    if (undecided_positions & intersection).is_empty() {
                        continue;
                    }
                    if (undecided_positions & rest_in_box).is_empty() {
                        // Pointing
                        let eliminations = undecided_positions & rest_in_line;
                        if grid.remove_candidate_with_mask(eliminations, digit)
                            && let ControlFlow::Break(value) = on_condition(
                                grid,
                                &Condition {
                                    kind: LockedCandidatesKind::Pointing,
                                    digit,
                                    box_,
                                    line,
                                },
                            )
                        {
                            return Some(value);
                        }
                    } else if (undecided_positions & rest_in_line).is_empty() {
                        // Claiming
                        let eliminations = undecided_positions & rest_in_box;
                        if grid.remove_candidate_with_mask(eliminations, digit)
                            && let ControlFlow::Break(value) = on_condition(
                                grid,
                                &Condition {
                                    kind: LockedCandidatesKind::Claiming,
                                    digit,
                                    box_,
                                    line,
                                },
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
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn tier(&self) -> TechniqueTier {
        TechniqueTier::Basic
    }

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        let mut after_grid = grid.clone();
        let step = Self::apply_with_control_flow(&mut after_grid, |after_grid, condition| {
            ControlFlow::Break(condition.build_step(grid, after_grid))
        });
        Ok(step)
    }

    fn apply_step(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let changed = Self::apply_with_control_flow(grid, |_, _| ControlFlow::Break(())).is_some();
        Ok(changed)
    }

    fn apply_pass(&self, grid: &mut TechniqueGrid) -> Result<usize, SolverError> {
        let mut changed = 0;
        Self::apply_with_control_flow(grid, |_, _| {
            changed += 1;
            ControlFlow::<()>::Continue(())
        });
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{CandidateGrid, Digit, Position};

    use super::*;
    use crate::testing;

    const TECHNIQUE: LockedCandidates = LockedCandidates::new();

    #[test]
    fn test_pointing_eliminates_from_row() {
        // Box 0 (rows 0-2, cols 0-2): limit D5 candidates to row 0 inside the box.
        let mut grid = CandidateGrid::new();
        for pos in Position::BOXES[0] {
            if pos.y() != 0 {
                grid.remove_candidate(pos, Digit::D5);
            }
        }

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t
                // D5 removed from the rest of row 0 outside the box.
                .assert_removed_includes(Position::new(3, 0), [Digit::D5])
                .assert_removed_includes(Position::new(8, 0), [Digit::D5]);
        });
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

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t
                // D7 removed from the rest of box 0 outside row 0.
                .assert_removed_includes(Position::new(0, 1), [Digit::D7])
                .assert_removed_includes(Position::new(2, 2), [Digit::D7]);
        });
    }

    #[test]
    fn test_no_change_when_no_locked_candidates() {
        // A fresh grid has no locked candidate eliminations.
        let grid = CandidateGrid::new();
        testing::test_technique_apply_pass_no_changes(grid, &TECHNIQUE);
    }
}
