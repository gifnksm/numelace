use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use super::{
    BoxedTechnique, BoxedTechniqueStep, ConditionCells, ConditionDigitCells, TechniqueApplication,
};
use crate::{
    SolverError, TechniqueGrid,
    technique::{Technique, TechniqueStep},
};

const NAME: &str = "Hidden Triple";

/// A technique that removes candidates using a hidden triple within a house.
///
/// A "hidden triple" occurs when three digits can only appear in the same three
/// cells of a row, column, or box. Other candidates in those three cells can be
/// removed.
///
/// # Examples
///
/// ```
/// use numelace_solver::{
///     TechniqueGrid,
///     technique::{HiddenTriple, Technique},
/// };
///
/// let mut grid = TechniqueGrid::new();
/// let technique = HiddenTriple::new();
///
/// // Apply the technique
/// let changed = technique.apply(&mut grid)?;
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct HiddenTriple {}

impl HiddenTriple {
    /// Creates a new `HiddenTriple` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone)]
pub struct HiddenTripleStep {
    positions: DigitPositions,
    digits: DigitSet,
    eliminate_digits: DigitSet,
}

impl HiddenTripleStep {
    pub fn new(positions: DigitPositions, digits: DigitSet, eliminate_digits: DigitSet) -> Self {
        Self {
            positions,
            digits,
            eliminate_digits,
        }
    }
}

impl TechniqueStep for HiddenTripleStep {
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

impl Technique for HiddenTriple {
    fn name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        for house in House::ALL {
            let house_positions = house.positions();
            for (d1, remaining_digits1) in DigitSet::FULL.pivots_with_following() {
                let d1_positions = grid.digit_positions(d1) & house_positions;
                if d1_positions.is_empty() || d1_positions.len() > 3 {
                    continue;
                }
                for (d2, remaining_digits2) in remaining_digits1.pivots_with_following() {
                    let d2_positions = grid.digit_positions(d2) & house_positions;
                    if d2_positions.is_empty() {
                        continue;
                    }
                    let pair_positions = d1_positions | d2_positions;
                    if pair_positions.len() > 3 {
                        continue;
                    }
                    for (d3, remaining_digits3) in remaining_digits2.pivots_with_following() {
                        let d3_positions = grid.digit_positions(d3) & house_positions;
                        if d3_positions.is_empty() {
                            continue;
                        }
                        let triple_positions = pair_positions | d3_positions;
                        if triple_positions.len() > 3 {
                            continue;
                        }
                        if triple_positions.len() < 3 {
                            return Err(ConsistencyError::CandidateConstraintViolation.into());
                        }

                        // Digits smaller than `d3` are checked in earlier combinations,
                        // so only the remaining digits need to be validated here.
                        for d in remaining_digits3 {
                            let other_positions = grid.digit_positions(d) & house_positions;
                            if !other_positions.is_empty()
                                && other_positions.is_subset(triple_positions)
                            {
                                return Err(ConsistencyError::CandidateConstraintViolation.into());
                            }
                        }

                        let eliminate_positions = triple_positions;
                        let mut eliminate_digits = DigitSet::EMPTY;
                        for d in DigitSet::FULL {
                            if d == d1 || d == d2 || d == d3 {
                                continue;
                            }
                            if grid.would_remove_candidate_with_mask_change(eliminate_positions, d)
                            {
                                eliminate_digits.insert(d);
                            }
                        }
                        if !eliminate_digits.is_empty() {
                            return Ok(Some(Box::new(HiddenTripleStep::new(
                                eliminate_positions,
                                DigitSet::from_iter([d1, d2, d3]),
                                eliminate_digits,
                            ))));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let mut changed = false;
        for house in House::ALL {
            let house_positions = house.positions();
            for (d1, remaining_digits1) in DigitSet::FULL.pivots_with_following() {
                let d1_positions = grid.digit_positions(d1) & house_positions;
                if d1_positions.is_empty() || d1_positions.len() > 3 {
                    continue;
                }
                for (d2, remaining_digits2) in remaining_digits1.pivots_with_following() {
                    let d2_positions = grid.digit_positions(d2) & house_positions;
                    if d2_positions.is_empty() {
                        continue;
                    }
                    let pair_positions = d1_positions | d2_positions;
                    if pair_positions.len() > 3 {
                        continue;
                    }
                    for (d3, remaining_digits3) in remaining_digits2.pivots_with_following() {
                        let d3_positions = grid.digit_positions(d3) & house_positions;
                        if d3_positions.is_empty() {
                            continue;
                        }
                        let triple_positions = pair_positions | d3_positions;
                        if triple_positions.len() > 3 {
                            continue;
                        }
                        if triple_positions.len() < 3 {
                            return Err(ConsistencyError::CandidateConstraintViolation.into());
                        }

                        // Digits smaller than `d3` are checked in earlier combinations,
                        // so only the remaining digits need to be validated here.
                        for d in remaining_digits3 {
                            let other_positions = grid.digit_positions(d) & house_positions;
                            if !other_positions.is_empty()
                                && other_positions.is_subset(triple_positions)
                            {
                                return Err(ConsistencyError::CandidateConstraintViolation.into());
                            }
                        }

                        let eliminate_positions = triple_positions;
                        for d in DigitSet::FULL {
                            if d == d1 || d == d2 || d == d3 {
                                continue;
                            }
                            changed |= grid.remove_candidate_with_mask(eliminate_positions, d);
                        }
                    }
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
    fn test_eliminates_hidden_triple_candidates_in_row() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);
        let pos3 = Position::new(6, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 && pos != pos3 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenTriple::new())
            .assert_removed_includes(pos1, [Digit::D4])
            .assert_removed_includes(pos2, [Digit::D4])
            .assert_removed_includes(pos3, [Digit::D4]);
    }

    #[test]
    fn test_find_step_returns_elimination() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);
        let pos3 = Position::new(6, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 && pos != pos3 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
            }
        }

        let grid = TechniqueGrid::from(grid);
        let step = HiddenTriple::new().find_step(&grid).unwrap();
        assert!(step.is_some());
    }

    #[test]
    fn test_no_change_when_no_hidden_triples() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&HiddenTriple::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }

    #[test]
    fn test_no_change_when_hidden_triple_has_no_eliminations() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);
        let pos3 = Position::new(6, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 && pos != pos3 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
            }
        }

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
                grid.remove_candidate(pos3, digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenTriple::new())
            .assert_no_change(Position::new(1, 0))
            .assert_no_change(Position::new(0, 1));
    }

    #[test]
    fn test_inconsistent_when_four_digits_share_three_positions() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);
        let pos3 = Position::new(6, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 && pos != pos3 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
                grid.remove_candidate(pos, Digit::D4);
            }
        }

        let mut grid = TechniqueGrid::from(grid);
        let result = HiddenTriple::new().apply(&mut grid);
        assert!(matches!(
            result,
            Err(SolverError::Inconsistent(
                ConsistencyError::CandidateConstraintViolation
            ))
        ));
    }
}
