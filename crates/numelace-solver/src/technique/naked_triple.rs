use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use super::{BoxedTechniqueStep, ConditionCells, ConditionDigitCells, TechniqueApplication};
use crate::technique::{Technique, TechniqueStep};

const NAME: &str = "Naked Triple";

/// A technique that removes candidates using a naked triple within a house.
///
/// A "naked triple" occurs when three cells in a row, column, or box contain
/// only three candidates in total. Those three digits can be eliminated from
/// all other cells in that house.
#[derive(Debug, Default, Clone, Copy)]
pub struct NakedTriple {}

impl NakedTriple {
    /// Creates a new `NakedTriple` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

/// A step describing a naked triple and its candidate eliminations.
#[derive(Debug, Clone)]
pub struct NakedTripleStep {
    positions: DigitPositions,
    digits: DigitSet,
    eliminate_positions: DigitPositions,
}

impl NakedTripleStep {
    /// Creates a new `NakedTripleStep`.
    #[must_use]
    pub fn new(
        positions: DigitPositions,
        digits: DigitSet,
        eliminate_positions: DigitPositions,
    ) -> Self {
        Self {
            positions,
            digits,
            eliminate_positions,
        }
    }
}

impl TechniqueStep for NakedTripleStep {
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
            positions: self.eliminate_positions,
            digits: self.digits,
        }]
    }
}

impl Technique for NakedTriple {
    fn name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> super::BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(
        &self,
        grid: &crate::TechniqueGrid,
    ) -> Result<Option<BoxedTechniqueStep>, crate::SolverError> {
        let classes = grid.classify_cells::<4>();
        let triple_candidate_cells = classes[2] | classes[3];
        if triple_candidate_cells.len() < 3 {
            return Ok(None);
        }

        for house in House::ALL {
            let triple_in_house = triple_candidate_cells & house.positions();
            if triple_in_house.len() < 3 {
                continue;
            }

            for (pos1, remaining_pos1) in triple_in_house.pivots_with_following() {
                let digits1 = grid.candidates_at(pos1);
                for (pos2, remaining_pos2) in remaining_pos1.pivots_with_following() {
                    let digits12 = digits1 | grid.candidates_at(pos2);
                    if digits12.len() > 3 {
                        continue;
                    }
                    for (pos3, remaining_pos3) in remaining_pos2.pivots_with_following() {
                        let digits123 = digits12 | grid.candidates_at(pos3);
                        if digits123.len() > 3 {
                            continue;
                        }
                        if digits123.len() < 3 {
                            return Err(ConsistencyError::CandidateConstraintViolation.into());
                        }

                        // Positions smaller than `pos3` are checked in earlier combinations,
                        // so only the remaining positions need to be validated here.
                        let has_fourth_cell = remaining_pos3
                            .iter()
                            .any(|pos| grid.candidates_at(pos).is_subset(digits123));
                        if has_fourth_cell {
                            return Err(ConsistencyError::CandidateConstraintViolation.into());
                        }

                        let mut eliminate_positions = house.positions();
                        eliminate_positions.remove(pos1);
                        eliminate_positions.remove(pos2);
                        eliminate_positions.remove(pos3);
                        for digit in digits123 {
                            if grid
                                .would_remove_candidate_with_mask_change(eliminate_positions, digit)
                            {
                                return Ok(Some(Box::new(NakedTripleStep::new(
                                    DigitPositions::from_iter([pos1, pos2, pos3]),
                                    digits123,
                                    eliminate_positions,
                                ))));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn apply(&self, grid: &mut crate::TechniqueGrid) -> Result<bool, crate::SolverError> {
        let classes = grid.classify_cells::<4>();
        let triple_candidate_cells = classes[2] | classes[3];
        if triple_candidate_cells.len() < 3 {
            return Ok(false);
        }

        let mut changed = false;
        for house in House::ALL {
            let triple_in_house = triple_candidate_cells & house.positions();
            if triple_in_house.len() < 3 {
                continue;
            }

            for (pos1, remaining_pos1) in triple_in_house.pivots_with_following() {
                let digits1 = grid.candidates_at(pos1);
                for (pos2, remaining_pos2) in remaining_pos1.pivots_with_following() {
                    let digits12 = digits1 | grid.candidates_at(pos2);
                    if digits12.len() > 3 {
                        continue;
                    }
                    for (pos3, remaining_pos3) in remaining_pos2.pivots_with_following() {
                        let digits123 = digits12 | grid.candidates_at(pos3);
                        if digits123.len() > 3 {
                            continue;
                        }
                        if digits123.len() < 3 {
                            return Err(ConsistencyError::CandidateConstraintViolation.into());
                        }

                        let has_fourth_cell = remaining_pos3
                            .iter()
                            .any(|pos| grid.candidates_at(pos).is_subset(digits123));
                        if has_fourth_cell {
                            return Err(ConsistencyError::CandidateConstraintViolation.into());
                        }

                        let mut eliminate_positions = house.positions();
                        eliminate_positions.remove(pos1);
                        eliminate_positions.remove(pos2);
                        eliminate_positions.remove(pos3);
                        for digit in digits123 {
                            changed |= grid.remove_candidate_with_mask(eliminate_positions, digit);
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
    fn test_eliminates_triple_candidates_in_row() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);
        let pos3 = Position::new(6, 0);
        let target = Position::new(4, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
                grid.remove_candidate(pos3, digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&NakedTriple::new())
            .assert_removed_includes(target, [Digit::D1, Digit::D2, Digit::D3]);
    }

    #[test]
    fn test_find_step_returns_elimination() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);
        let pos3 = Position::new(6, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
                grid.remove_candidate(pos3, digit);
            }
        }

        let grid = TechniqueGrid::from(grid);
        let step = NakedTriple::new().find_step(&grid).unwrap();
        assert!(step.is_some());
    }

    #[test]
    fn test_no_change_when_no_naked_triples() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&NakedTriple::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }

    #[test]
    fn test_no_change_when_triple_has_no_eliminations() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);
        let pos3 = Position::new(6, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
                grid.remove_candidate(pos3, digit);
            }
        }

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 && pos != pos3 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&NakedTriple::new())
            .assert_no_change(Position::new(1, 0))
            .assert_no_change(Position::new(0, 1));
    }

    #[test]
    fn test_inconsistent_when_four_cells_share_triple_candidates() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 0);
        let pos3 = Position::new(6, 0);
        let pos4 = Position::new(8, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
                grid.remove_candidate(pos3, digit);
                grid.remove_candidate(pos4, digit);
            }
        }

        let mut grid = TechniqueGrid::from(grid);
        let result = NakedTriple::new().apply(&mut grid);
        assert!(matches!(
            result,
            Err(SolverError::Inconsistent(
                ConsistencyError::CandidateConstraintViolation
            ))
        ));
    }
}
