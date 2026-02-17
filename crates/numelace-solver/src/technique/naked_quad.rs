use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use crate::{
    SolverError, TechniqueGrid,
    technique::{
        BoxedTechniqueStep, ConditionCells, ConditionDigitCells, Technique, TechniqueApplication,
        TechniqueStep,
    },
};

use super::BoxedTechnique;

const NAME: &str = "Naked Quad";

/// A technique that removes candidates using a naked quad within a house.
///
/// A "naked quad" occurs when four cells in a row, column, or box contain
/// only four candidates in total. Those four digits can be eliminated from
/// all other cells in that house.
#[derive(Debug, Default, Clone, Copy)]
pub struct NakedQuad {}

impl NakedQuad {
    /// Creates a new `NakedQuad` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

/// A step describing a naked quad and its candidate eliminations.
#[derive(Debug, Clone)]
pub struct NakedQuadStep {
    positions: DigitPositions,
    digits: DigitSet,
    eliminate_positions: DigitPositions,
}

impl NakedQuadStep {
    /// Creates a new `NakedQuadStep`.
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

impl TechniqueStep for NakedQuadStep {
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

impl Technique for NakedQuad {
    fn name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        let classes = grid.classify_cells::<5>();
        let quad_candidate_cells = classes[2] | classes[3] | classes[4];
        if quad_candidate_cells.len() < 4 {
            return Ok(None);
        }
        for house in House::ALL {
            let quad_in_house = quad_candidate_cells & house.positions();
            if quad_in_house.len() < 4 {
                continue;
            }
            for (pos1, remaining_pos1) in quad_in_house.pivots_with_following() {
                let digits1 = grid.candidates_at(pos1);
                for (pos2, remaining_pos2) in remaining_pos1.pivots_with_following() {
                    let digits12 = digits1 | grid.candidates_at(pos2);
                    if digits12.len() > 4 {
                        continue;
                    }
                    for (pos3, remaining_pos3) in remaining_pos2.pivots_with_following() {
                        let digits123 = digits12 | grid.candidates_at(pos3);
                        if digits123.len() > 4 {
                            continue;
                        }
                        for (pos4, remaining_pos4) in remaining_pos3.pivots_with_following() {
                            let digits1234 = digits123 | grid.candidates_at(pos4);
                            if digits1234.len() > 4 {
                                continue;
                            }
                            if digits1234.len() < 4 {
                                return Err(ConsistencyError::CandidateConstraintViolation.into());
                            }

                            // Positions smaller than `pos4` are checked in earlier combinations,
                            // so only the remaining positions need to be validated here.
                            let has_fifth_cell = remaining_pos4
                                .iter()
                                .any(|pos| grid.candidates_at(pos).is_subset(digits1234));
                            if has_fifth_cell {
                                return Err(ConsistencyError::CandidateConstraintViolation.into());
                            }

                            let mut eliminate_positions = house.positions();
                            eliminate_positions.remove(pos1);
                            eliminate_positions.remove(pos2);
                            eliminate_positions.remove(pos3);
                            eliminate_positions.remove(pos4);
                            if grid.would_remove_candidate_set_with_mask_change(
                                eliminate_positions,
                                digits1234,
                            ) {
                                return Ok(Some(Box::new(NakedQuadStep::new(
                                    DigitPositions::from_iter([pos1, pos2, pos3, pos4]),
                                    digits1234,
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

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let classes = grid.classify_cells::<5>();
        let quad_candidate_cells = classes[2] | classes[3] | classes[4];
        if quad_candidate_cells.len() < 4 {
            return Ok(false);
        }
        let mut changed = false;
        for house in House::ALL {
            let quad_in_house = quad_candidate_cells & house.positions();
            if quad_in_house.len() < 4 {
                continue;
            }
            for (pos1, remaining_pos1) in quad_in_house.pivots_with_following() {
                let digits1 = grid.candidates_at(pos1);
                for (pos2, remaining_pos2) in remaining_pos1.pivots_with_following() {
                    let digits12 = digits1 | grid.candidates_at(pos2);
                    if digits12.len() > 4 {
                        continue;
                    }
                    for (pos3, remaining_pos3) in remaining_pos2.pivots_with_following() {
                        let digits123 = digits12 | grid.candidates_at(pos3);
                        if digits123.len() > 4 {
                            continue;
                        }
                        for (pos4, remaining_pos4) in remaining_pos3.pivots_with_following() {
                            let digits1234 = digits123 | grid.candidates_at(pos4);
                            if digits1234.len() > 4 {
                                continue;
                            }
                            if digits1234.len() < 4 {
                                return Err(ConsistencyError::CandidateConstraintViolation.into());
                            }

                            // Positions smaller than `pos4` are checked in earlier combinations,
                            // so only the remaining positions need to be validated here.
                            let has_fifth_cell = remaining_pos4
                                .iter()
                                .any(|pos| grid.candidates_at(pos).is_subset(digits1234));
                            if has_fifth_cell {
                                return Err(ConsistencyError::CandidateConstraintViolation.into());
                            }

                            let mut eliminate_positions = house.positions();
                            eliminate_positions.remove(pos1);
                            eliminate_positions.remove(pos2);
                            eliminate_positions.remove(pos3);
                            eliminate_positions.remove(pos4);
                            changed |= grid
                                .remove_candidate_set_with_mask(eliminate_positions, digits1234);
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
    fn test_eliminates_quad_candidates_in_row() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(2, 0);
        let pos3 = Position::new(4, 0);
        let pos4 = Position::new(6, 0);
        let target = Position::new(8, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 && digit != Digit::D4
            {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
                grid.remove_candidate(pos3, digit);
                grid.remove_candidate(pos4, digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&NakedQuad::new())
            .assert_removed_includes(target, [Digit::D1, Digit::D2, Digit::D3, Digit::D4]);
    }

    #[test]
    fn test_find_step_returns_elimination() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(2, 0);
        let pos3 = Position::new(4, 0);
        let pos4 = Position::new(6, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 && digit != Digit::D4
            {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
                grid.remove_candidate(pos3, digit);
                grid.remove_candidate(pos4, digit);
            }
        }

        let grid = TechniqueGrid::from(grid);
        let step = NakedQuad::new().find_step(&grid).unwrap();
        assert!(step.is_some());
    }

    #[test]
    fn test_no_change_when_no_naked_quads() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&NakedQuad::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }

    #[test]
    fn test_no_change_when_quad_has_no_eliminations() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(2, 0);
        let pos3 = Position::new(4, 0);
        let pos4 = Position::new(6, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 && digit != Digit::D4
            {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
                grid.remove_candidate(pos3, digit);
                grid.remove_candidate(pos4, digit);
            }
        }

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 && pos != pos3 && pos != pos4 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
                grid.remove_candidate(pos, Digit::D4);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&NakedQuad::new())
            .assert_no_change(Position::new(1, 0))
            .assert_no_change(Position::new(0, 1));
    }

    #[test]
    fn test_inconsistent_when_five_cells_share_quad_candidates() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(2, 0);
        let pos3 = Position::new(4, 0);
        let pos4 = Position::new(6, 0);
        let pos5 = Position::new(8, 0);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 && digit != Digit::D4
            {
                grid.remove_candidate(pos1, digit);
                grid.remove_candidate(pos2, digit);
                grid.remove_candidate(pos3, digit);
                grid.remove_candidate(pos4, digit);
                grid.remove_candidate(pos5, digit);
            }
        }

        let mut grid = TechniqueGrid::from(grid);
        let result = NakedQuad::new().apply(&mut grid);
        assert!(matches!(
            result,
            Err(SolverError::Inconsistent(
                ConsistencyError::CandidateConstraintViolation
            ))
        ));
    }
}
