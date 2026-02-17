use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use super::{
    BoxedTechnique, BoxedTechniqueStep, ConditionCells, ConditionDigitCells, TechniqueApplication,
};
use crate::{
    SolverError, TechniqueGrid,
    technique::{Technique, TechniqueStep},
};

const NAME: &str = "Hidden Pair";

/// A technique that removes candidates using a hidden pair within a house.
///
/// A "hidden pair" occurs when two digits can only appear in the same two cells
/// of a row, column, or box. Other candidates in those two cells can be removed.
///
/// # Examples
///
/// ```
/// use numelace_solver::{
///     TechniqueGrid,
///     technique::{HiddenPair, Technique},
/// };
///
/// let mut grid = TechniqueGrid::new();
/// let technique = HiddenPair::new();
///
/// // Apply the technique
/// let changed = technique.apply(&mut grid)?;
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct HiddenPair {}

#[derive(Debug, Clone)]
pub struct HiddenPairStep {
    positions: DigitPositions,
    digits: DigitSet,
    application: Vec<TechniqueApplication>,
}

impl HiddenPairStep {
    /// Creates a new `HiddenPairStep`.
    #[must_use]
    pub fn new(
        positions: DigitPositions,
        digits: DigitSet,
        application: Vec<TechniqueApplication>,
    ) -> Self {
        Self {
            positions,
            digits,
            application,
        }
    }
}

impl TechniqueStep for HiddenPairStep {
    fn technique_name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> BoxedTechniqueStep {
        Box::new(self.clone())
    }

    fn condition_cells(&self) -> ConditionCells {
        self.positions
    }

    fn condition_digit_cells(&self) -> ConditionDigitCells {
        vec![(self.positions, self.digits)]
    }

    fn application(&self) -> Vec<TechniqueApplication> {
        self.application.clone()
    }
}

impl HiddenPair {
    /// Creates a new `HiddenPair` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl HiddenPair {
    fn apply_with_control_flow<F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Result<Option<HiddenPairStep>, SolverError>
    where
        F: for<'a> FnMut(
            &'a mut TechniqueGrid,
            DigitPositions,
            DigitSet,
        ) -> ControlFlow<HiddenPairStep>,
    {
        for house in House::ALL {
            let house_positions = house.positions();
            for (d1, remaining_digits) in DigitSet::FULL.pivots_with_following() {
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

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        let mut after_grid = grid.clone();
        let step =
            Self::apply_with_control_flow(&mut after_grid, |after_grid, positions, digits| {
                let app = super::collect_applications_from_diff(grid, after_grid);
                ControlFlow::Break(HiddenPairStep::new(positions, digits, app))
            })?;
        Ok(step.map(|step| step.clone_box()))
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
