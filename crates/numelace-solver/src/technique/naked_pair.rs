use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use super::{
    BoxedTechnique, BoxedTechniqueStep, ConditionCells, ConditionDigitCells, TechniqueApplication,
};
use crate::{
    SolverError, TechniqueGrid,
    technique::{Technique, TechniqueStep},
};

const NAME: &str = "Naked Pair";

/// A technique that removes candidates using a naked pair within a house.
///
/// A "naked pair" occurs when two cells in a row, column, or box contain the
/// same two candidates. Those two digits can be eliminated from all other
/// cells in that house.
///
/// # Examples
///
/// ```
/// use numelace_solver::{
///     TechniqueGrid,
///     technique::{NakedPair, Technique},
/// };
///
/// let mut grid = TechniqueGrid::new();
/// let technique = NakedPair::new();
///
/// // Apply the technique
/// let changed = technique.apply(&mut grid)?;
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct NakedPair {}

#[derive(Debug, Clone)]
pub struct NakedPairStep {
    positions: DigitPositions,
    digits: DigitSet,
    application: TechniqueApplication,
}

impl NakedPairStep {
    pub fn new(
        positions: DigitPositions,
        digits: DigitSet,
        application: TechniqueApplication,
    ) -> Self {
        Self {
            positions,
            digits,
            application,
        }
    }
}

impl TechniqueStep for NakedPairStep {
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
        vec![self.application]
    }
}

impl NakedPair {
    /// Creates a new `NakedPair` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
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
        let pair_candidate_cells = grid.classify_cells::<3>()[2];
        if pair_candidate_cells.len() < 2 {
            return Ok(None);
        }

        for house in House::ALL {
            let pair_in_house = pair_candidate_cells & house.positions();
            if pair_in_house.len() < 2 {
                continue;
            }
            for (pos1, mut matching_pair_cells) in pair_in_house.pivots_with_following() {
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
                if let Some(app) =
                    grid.plan_remove_candidate_set_with_mask(eliminate_positions, pair_digits)
                {
                    return Ok(Some(Box::new(NakedPairStep::new(
                        DigitPositions::from_iter([pos1, pos2]),
                        pair_digits,
                        app,
                    ))));
                }
            }
        }
        Ok(None)
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let pair_candidate_cells = grid.classify_cells::<3>()[2];
        if pair_candidate_cells.len() < 2 {
            return Ok(false);
        }

        let mut changed = false;
        for house in House::ALL {
            let pair_in_house = pair_candidate_cells & house.positions();
            if pair_in_house.len() < 2 {
                continue;
            }
            for (pos1, mut matching_pair_cells) in pair_in_house.pivots_with_following() {
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
                    changed |= grid.remove_candidate_with_mask(eliminate_positions, digit);
                }
            }
        }
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
