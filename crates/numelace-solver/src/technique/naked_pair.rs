use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House, Position};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
};

const NAME: &str = "Naked Pair";

/// A technique that removes candidates using a naked pair within a house.
///
/// A "naked pair" occurs when two cells in a row, column, or box contain the
/// same two candidates. Those two digits can be eliminated from all other
/// cells in that house.
#[derive(Debug, Default, Clone, Copy)]
pub struct NakedPair {}

impl NakedPair {
    /// Creates a new `NakedPair` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl NakedPair {
    fn apply_with_control_flow<F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Result<Option<BoxedTechniqueStep>, SolverError>
    where
        F: for<'a> FnMut(
            &'a mut TechniqueGrid,
            [Position; 2],
            DigitSet,
        ) -> ControlFlow<BoxedTechniqueStep>,
    {
        let pair_candidate_cells = grid.classify_cells::<3>()[2];
        if pair_candidate_cells.len() < 2 {
            return Ok(None);
        }

        for house in House::ALL {
            let pair_in_house = pair_candidate_cells & house.positions();
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
                if matching_pair_cells.len() > 1 {
                    return Err(ConsistencyError::CandidateConstraintViolation.into());
                }
                let Some(pos2) = matching_pair_cells.as_single() else {
                    continue;
                };

                let mut eliminate_positions = house.positions();
                eliminate_positions.remove(pos1);
                eliminate_positions.remove(pos2);
                for digit in pair_digits {
                    if grid.remove_candidate_with_mask(eliminate_positions, digit)
                        && let ControlFlow::Break(step) =
                            on_condition(grid, [pos1, pos2], pair_digits)
                    {
                        return Ok(Some(step));
                    }
                }
            }
        }
        Ok(None)
    }
}

impl Technique for NakedPair {
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
            |after_grid, [pos1, pos2], pair_digits| {
                ControlFlow::Break(Box::new(TechniqueStepData::from_diff(
                    NAME,
                    DigitPositions::from_iter([pos1, pos2]),
                    vec![(DigitPositions::from_iter([pos1, pos2]), pair_digits)],
                    grid,
                    after_grid,
                )))
            },
        )?;
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
    fn test_eliminates_pair_candidates_in_row() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);
        let target = Position::new(4, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&NakedPair::new())
            .assert_removed_includes(target, [Digit::D1, Digit::D2]);
    }

    #[test]
    fn test_no_change_when_no_naked_pairs() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&NakedPair::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }

    #[test]
    fn test_no_change_when_pair_has_no_eliminations() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(1, 0);

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

        TechniqueTester::new(grid)
            .apply_once(&NakedPair::new())
            .assert_no_change(Position::new(2, 0))
            .assert_no_change(Position::new(0, 1));
    }

    #[test]
    fn test_inconsistent_when_three_cells_share_pair_candidates() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);
        let pos3 = Position::new(6, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
                grid.remove_candidate(pos3, digit);
            }
        }

        let mut grid = TechniqueGrid::from(grid);
        let result = NakedPair::new().apply(&mut grid);
        assert!(matches!(
            result,
            Err(SolverError::Inconsistent(
                ConsistencyError::CandidateConstraintViolation
            ))
        ));
    }
}
