use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
};

const ID: &str = "naked_pair";
const NAME: &str = "Naked Pair";

/// A technique that removes candidates using a naked pair within a house.
///
/// A "naked pair" occurs when two cells in a row, column, or box contain the
/// same two candidates. Those two digits can be eliminated from all other
/// cells in that house.
#[derive(Debug, Default, Clone, Copy)]
pub struct NakedPair {}

struct Condition {
    house: House,
    digits: DigitSet,
    positions: DigitPositions,
}

impl Condition {
    fn build_step(
        &self,
        before_grid: &TechniqueGrid,
        after_grid: &TechniqueGrid,
    ) -> BoxedTechniqueStep {
        let condition_positions = self.house.positions();
        let condition_digit_positions = vec![(self.positions, self.digits)];
        TechniqueStepData::from_diff(
            NAME,
            condition_positions,
            condition_digit_positions,
            before_grid,
            after_grid,
        )
    }
}

impl NakedPair {
    /// Creates a new `NakedPair` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    fn apply_with_control_flow<T, F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Result<Option<T>, SolverError>
    where
        F: for<'a> FnMut(&'a mut TechniqueGrid, &'a Condition) -> ControlFlow<T>,
    {
        let bivalue_positions = grid.classify_positions::<3>()[2];
        if bivalue_positions.len() < 2 {
            return Ok(None);
        }

        for house in House::ALL {
            let pair_in_house = bivalue_positions & house.positions();
            if pair_in_house.len() < 2 {
                continue;
            }
            for (pos1, mut matching_pair_cells) in pair_in_house
                .pivots_with_following()
                .take(pair_in_house.len() - 1)
            {
                let pair_digits = grid.candidates_at(pos1);
                for d in pair_digits {
                    matching_pair_cells &= grid.digit_positions(d);
                }
                // If more than two cells share the same pair candidates, each of those cells
                // would still require a placement, but the pair only provides two slots.
                // This is a candidate constraint violation.
                if matching_pair_cells.len() > 1 {
                    return Err(ConsistencyError::CandidateConstraintViolation.into());
                }
                let Some(pos2) = matching_pair_cells.as_single() else {
                    continue;
                };

                let mut eliminate_positions = house.positions();
                eliminate_positions.remove(pos1);
                eliminate_positions.remove(pos2);
                if grid.remove_candidate_set_with_mask(eliminate_positions, pair_digits)
                    && let ControlFlow::Break(step) = on_condition(
                        grid,
                        &Condition {
                            house,
                            digits: pair_digits,
                            positions: DigitPositions::from_iter([pos1, pos2]),
                        },
                    )
                {
                    return Ok(Some(step));
                }
            }
        }
        Ok(None)
    }
}

impl Technique for NakedPair {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn tier(&self) -> TechniqueTier {
        TechniqueTier::Intermediate
    }

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        let mut after_grid = grid.clone();
        let step = Self::apply_with_control_flow(&mut after_grid, |after_grid, condition| {
            ControlFlow::Break(condition.build_step(grid, after_grid))
        })?;
        Ok(step)
    }

    fn apply_step(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let changed = Self::apply_with_control_flow(grid, |_, _| ControlFlow::Break(()))?.is_some();
        Ok(changed)
    }

    fn apply_pass(&self, grid: &mut TechniqueGrid) -> Result<usize, SolverError> {
        let mut changed = 0;
        Self::apply_with_control_flow(grid, |_, _| {
            changed += 1;
            ControlFlow::<()>::Continue(())
        })?;
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{CandidateGrid, Digit, Position};

    use super::*;
    use crate::testing;

    const TECHNIQUE: NakedPair = NakedPair::new();

    #[test]
    fn test_eliminates_pair_candidates_in_row() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::from_xy(0, 0);
        let pos2 = Position::from_xy(3, 0);
        let target = Position::from_xy(4, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
            }
        }

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t.assert_removed_includes(target, [Digit::D1, Digit::D2]);
        });
    }

    #[test]
    fn test_no_change_when_no_naked_pairs() {
        let grid = CandidateGrid::new();
        testing::test_technique_apply_pass_no_changes(grid, &TECHNIQUE);
    }

    #[test]
    fn test_no_change_when_pair_has_no_eliminations() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::from_xy(0, 0);
        let pos2 = Position::from_xy(1, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
            }
        }

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
            }
        }

        for pos in Position::BOXES[0] {
            if pos != pos1 && pos != pos2 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
            }
        }

        testing::test_technique_apply_pass_no_changes(grid, &TECHNIQUE);
    }

    #[test]
    fn test_inconsistent_when_three_cells_share_pair_candidates() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::from_xy(0, 0);
        let pos2 = Position::from_xy(3, 0);
        let pos3 = Position::from_xy(6, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
                grid.remove_candidate(pos3, digit);
            }
        }

        testing::test_technique_apply_pass_fail_with_constraint_violation(grid, &TECHNIQUE);
    }
}
