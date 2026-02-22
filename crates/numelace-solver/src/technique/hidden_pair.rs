use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
};

const NAME: &str = "Hidden Pair";

/// A technique that removes candidates using a hidden pair within a house.
///
/// A "hidden pair" occurs when two digits can only appear in the same two cells
/// of a row, column, or box. Other candidates in those two cells can be removed.
#[derive(Debug, Default, Clone, Copy)]
pub struct HiddenPair {}

impl HiddenPair {
    /// Creates a new `HiddenPair` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    fn apply_with_control_flow<F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Result<Option<BoxedTechniqueStep>, SolverError>
    where
        F: for<'a> FnMut(
            &'a mut TechniqueGrid,
            DigitPositions,
            DigitSet,
        ) -> ControlFlow<BoxedTechniqueStep>,
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
                if candidate_digits.next().is_some() {
                    return Err(ConsistencyError::CandidateConstraintViolation.into());
                }
                let digits12 = digits1 | DigitSet::from_elem(d2);
                let eliminate_positions = d1_positions;
                if grid.remove_candidate_set_with_mask(eliminate_positions, !digits12)
                    && let ControlFlow::Break(step) =
                        on_condition(grid, eliminate_positions, digits12)
                {
                    return Ok(Some(step));
                }
            }
        }
        Ok(None)
    }
}

impl Technique for HiddenPair {
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
        let step =
            Self::apply_with_control_flow(&mut after_grid, |after_grid, positions, digits| {
                ControlFlow::Break(Box::new(TechniqueStepData::from_diff(
                    NAME,
                    positions,
                    vec![(positions, digits)],
                    grid,
                    after_grid,
                )))
            })?;
        Ok(step)
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let mut changed = false;
        Self::apply_with_control_flow(grid, |_, _, _| {
            changed = true;
            ControlFlow::Continue(())
        })?;
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{CandidateGrid, ConsistencyError, Digit, Position};

    use super::*;
    use crate::{SolverError, TechniqueGrid, testing::TechniqueTester};

    #[test]
    fn test_eliminates_hidden_pair_candidates_in_row() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenPair::new())
            .assert_removed_includes(pos1, [Digit::D3])
            .assert_removed_includes(pos2, [Digit::D3]);
    }

    #[test]
    fn test_no_change_when_no_hidden_pairs() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&HiddenPair::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }

    #[test]
    fn test_inconsistent_when_three_digits_share_two_positions() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
            }
        }

        let mut grid = TechniqueGrid::from(grid);
        let result = HiddenPair::new().apply(&mut grid);
        assert!(matches!(
            result,
            Err(SolverError::Inconsistent(
                ConsistencyError::CandidateConstraintViolation
            ))
        ));
    }
}
