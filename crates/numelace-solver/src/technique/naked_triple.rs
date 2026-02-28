use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
};

const ID: &str = "naked_triple";
const NAME: &str = "Naked Triple";

/// A technique that removes candidates using a naked triple within a house.
///
/// A "naked triple" occurs when three cells in a row, column, or box contain
/// only three candidates in total. Those three digits can be eliminated from
/// all other cells in that house.
#[derive(Debug, Default, Clone, Copy)]
pub struct NakedTriple {}

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
        let condition_cells = self.house.positions();
        let condition_digit_cells = vec![(self.positions, self.digits)];
        TechniqueStepData::from_diff(
            NAME,
            condition_cells,
            condition_digit_cells,
            before_grid,
            after_grid,
        )
    }
}

impl NakedTriple {
    /// Creates a new `NakedTriple` technique.
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
            for (pos1, remaining_pos1) in triple_in_house
                .pivots_with_following()
                .take(triple_in_house.len() - 2)
            {
                let digits1 = grid.candidates_at(pos1);
                for (pos2, remaining_pos2) in remaining_pos1
                    .pivots_with_following()
                    .take(remaining_pos1.len() - 1)
                {
                    let digits12 = digits1 | grid.candidates_at(pos2);
                    if digits12.len() > 3 {
                        continue;
                    }
                    for (pos3, remaining_pos3) in remaining_pos2.pivots_with_following() {
                        let digits123 = digits12 | grid.candidates_at(pos3);
                        if digits123.len() > 3 {
                            continue;
                        }
                        // If three cells only cover fewer than three digits, each cell would still
                        // require a placement while the digit set cannot host them all.
                        // This is a candidate constraint violation.
                        if digits123.len() < 3 {
                            return Err(ConsistencyError::CandidateConstraintViolation.into());
                        }

                        // Positions smaller than `pos3` are checked in earlier combinations,
                        // so only the remaining positions need to be validated here.
                        let has_fourth_cell = remaining_pos3
                            .iter()
                            .any(|pos| grid.candidates_at(pos).is_subset(digits123));
                        // If a fourth cell is also restricted to the same three digits,
                        // the house would require four placements from three digits.
                        // This is a candidate constraint violation.
                        if has_fourth_cell {
                            return Err(ConsistencyError::CandidateConstraintViolation.into());
                        }

                        let mut eliminate_positions = house.positions();
                        eliminate_positions.remove(pos1);
                        eliminate_positions.remove(pos2);
                        eliminate_positions.remove(pos3);
                        if grid.remove_candidate_set_with_mask(eliminate_positions, digits123)
                            && let ControlFlow::Break(step) = on_condition(
                                grid,
                                &Condition {
                                    house,
                                    digits: digits123,
                                    positions: DigitPositions::from_iter([pos1, pos2, pos3]),
                                },
                            )
                        {
                            return Ok(Some(step));
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}

impl Technique for NakedTriple {
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
            .apply_pass(&NakedTriple::new())
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
            .apply_pass(&NakedTriple::new())
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
            .apply_pass(&NakedTriple::new())
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
        let result = NakedTriple::new().apply_pass(&mut grid);
        assert!(matches!(
            result,
            Err(SolverError::Inconsistent(
                ConsistencyError::CandidateConstraintViolation
            ))
        ));
    }
}
