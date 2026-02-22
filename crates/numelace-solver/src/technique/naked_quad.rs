use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
};

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

impl NakedQuad {
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
            for (pos1, remaining_pos1) in quad_in_house
                .pivots_with_following()
                .take(quad_in_house.len() - 3)
            {
                let digits1 = grid.candidates_at(pos1);
                for (pos2, remaining_pos2) in remaining_pos1
                    .pivots_with_following()
                    .take(remaining_pos1.len() - 2)
                {
                    let digits12 = digits1 | grid.candidates_at(pos2);
                    if digits12.len() > 4 {
                        continue;
                    }
                    for (pos3, remaining_pos3) in remaining_pos2
                        .pivots_with_following()
                        .take(remaining_pos2.len() - 1)
                    {
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
                            if grid.remove_candidate_set_with_mask(eliminate_positions, digits1234)
                                && let ControlFlow::Break(step) = on_condition(
                                    grid,
                                    DigitPositions::from_iter([pos1, pos2, pos3, pos4]),
                                    digits1234,
                                )
                            {
                                return Ok(Some(step));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
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
