use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use crate::{
    SolverError, TechniqueGrid,
    technique::{Technique, TechniqueStep},
};

use super::{
    BoxedTechnique, BoxedTechniqueStep, ConditionCells, ConditionDigitCells, TechniqueApplication,
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
    eliminate_digits: DigitSet,
}

impl HiddenPairStep {
    /// Creates a new `HiddenPairStep`.
    #[must_use]
    pub fn new(positions: DigitPositions, digits: DigitSet, eliminate_digits: DigitSet) -> Self {
        Self {
            positions,
            digits,
            eliminate_digits,
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
        vec![TechniqueApplication::CandidateElimination {
            positions: self.positions,
            digits: self.eliminate_digits,
        }]
    }
}

impl HiddenPair {
    /// Creates a new `HiddenPair` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
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
        for house in House::ALL {
            let house_positions = house.positions();

            let mut remaining_digits = DigitSet::FULL;
            while let Some(d1) = remaining_digits.pop_first() {
                let d1_positions = grid.digit_positions(d1) & house_positions;
                if d1_positions.len() != 2 {
                    continue;
                }

                let mut candidate_digits = remaining_digits
                    .into_iter()
                    .filter(|d2| grid.digit_positions(*d2) & house_positions == d1_positions);
                let Some(d2) = candidate_digits.next() else {
                    continue;
                };
                if candidate_digits.next().is_some() {
                    return Err(ConsistencyError::CandidateConstraintViolation.into());
                }

                let mut eliminate_digits = DigitSet::EMPTY;
                let eliminate_positions = d1_positions;
                for d in DigitSet::FULL {
                    if d == d1 || d == d2 {
                        continue;
                    }
                    if grid.would_remove_candidate_with_mask_change(eliminate_positions, d) {
                        eliminate_digits.insert(d);
                    }
                }
                if !eliminate_digits.is_empty() {
                    return Ok(Some(Box::new(HiddenPairStep::new(
                        eliminate_positions,
                        DigitSet::from_iter([d1, d2]),
                        eliminate_digits,
                    ))));
                }
            }
        }
        Ok(None)
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let mut changed = false;
        for house in House::ALL {
            let house_positions = house.positions();

            let mut remaining_digits = DigitSet::FULL;
            while let Some(d1) = remaining_digits.pop_first() {
                let d1_positions = grid.digit_positions(d1) & house_positions;
                if d1_positions.len() != 2 {
                    continue;
                }

                let mut candidate_digits = remaining_digits
                    .into_iter()
                    .filter(|d2| grid.digit_positions(*d2) & house_positions == d1_positions);
                let Some(d2) = candidate_digits.next() else {
                    continue;
                };
                if candidate_digits.next().is_some() {
                    return Err(ConsistencyError::CandidateConstraintViolation.into());
                }

                let eliminate_positions = d1_positions;
                for d in DigitSet::FULL {
                    if d == d1 || d == d2 {
                        continue;
                    }
                    changed |= grid.remove_candidate_with_mask(eliminate_positions, d);
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
