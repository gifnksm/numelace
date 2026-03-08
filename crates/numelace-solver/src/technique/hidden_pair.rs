use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
};

const ID: &str = "hidden_pair";
const NAME: &str = "Hidden Pair";

/// A technique that removes candidates using a hidden pair within a house.
///
/// A "hidden pair" occurs when two digits can only appear in the same two cells
/// of a row, column, or box. Other candidates in those two cells can be removed.
#[derive(Debug, Default, Clone, Copy)]
pub struct HiddenPair {}

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

impl HiddenPair {
    /// Creates a new `HiddenPair` technique.
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
        for house in House::ALL {
            let house_positions = house.positions();
            for (d1, remaining_digits) in DigitSet::FULL.pivots_with_following().take(8) {
                let d1_positions = grid.digit_positions(d1) & house_positions;
                if d1_positions.len() != 2 {
                    continue;
                }
                let digits1 = DigitSet::from_elem(d1);
                let mut candidate_digits = remaining_digits
                    .into_iter()
                    .filter(|d2| grid.digit_positions(*d2) & house_positions == d1_positions);
                let Some(d2) = candidate_digits.next() else {
                    continue;
                };
                // If more than two digits share the same two positions, a placement is forced
                // in those cells while additional digits still require them. This is a
                // candidate constraint violation.
                if candidate_digits.next().is_some() {
                    return Err(ConsistencyError::CandidateConstraintViolation.into());
                }
                let digits12 = digits1 | DigitSet::from_elem(d2);
                let eliminate_positions = d1_positions;
                if grid.remove_candidate_set_with_mask(eliminate_positions, !digits12)
                    && let ControlFlow::Break(step) = on_condition(
                        grid,
                        &Condition {
                            house,
                            digits: digits12,
                            positions: d1_positions,
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

impl Technique for HiddenPair {
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

    const TECHNIQUE: HiddenPair = HiddenPair::new();

    #[test]
    fn test_eliminates_hidden_pair_candidates_in_row() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::from_xy(0, 0);
        let pos2 = Position::from_xy(3, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
            }
        }

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t.assert_removed_includes(pos1, [Digit::D3])
                .assert_removed_includes(pos2, [Digit::D3]);
        });
    }

    #[test]
    fn test_no_change_when_no_hidden_pairs() {
        let grid = CandidateGrid::new();
        testing::test_technique_apply_pass_no_changes(grid, &TECHNIQUE);
    }

    #[test]
    fn test_inconsistent_when_three_digits_share_two_positions() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::from_xy(0, 0);
        let pos2 = Position::from_xy(3, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
            }
        }

        testing::test_technique_apply_pass_fail_with_constraint_violation(grid, &TECHNIQUE);
    }
}
