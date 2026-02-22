use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
};

const NAME: &str = "Hidden Triple";

/// A technique that removes candidates using a hidden triple within a house.
///
/// A "hidden triple" occurs when three digits can only appear in the same three
/// cells of a row, column, or box. Other candidates in those three cells can be
/// removed.
#[derive(Debug, Default, Clone, Copy)]
pub struct HiddenTriple {}

impl HiddenTriple {
    /// Creates a new `HiddenTriple` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl HiddenTriple {
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
            for (d1, remaining_digits1) in DigitSet::FULL.pivots_with_following().take(7) {
                let d1_positions = grid.digit_positions(d1) & house_positions;
                if d1_positions.is_empty() || d1_positions.len() > 3 {
                    continue;
                }
                let digits1 = DigitSet::from_elem(d1);
                for (d2, remaining_digits2) in remaining_digits1
                    .pivots_with_following()
                    .take(remaining_digits1.len() - 1)
                {
                    let d2_positions = grid.digit_positions(d2) & house_positions;
                    if d2_positions.is_empty() {
                        continue;
                    }
                    let pair_positions = d1_positions | d2_positions;
                    if pair_positions.len() > 3 {
                        continue;
                    }
                    let digits12 = digits1 | DigitSet::from_elem(d2);
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

                        let digits123 = digits12 | DigitSet::from_elem(d3);
                        let eliminate_positions = triple_positions;
                        if grid.remove_candidate_set_with_mask(eliminate_positions, !digits123)
                            && let ControlFlow::Break(step) =
                                on_condition(grid, eliminate_positions, digits123)
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

impl Technique for HiddenTriple {
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
