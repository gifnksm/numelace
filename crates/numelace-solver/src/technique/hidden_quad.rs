use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, DigitPositions, DigitSet, House};

use crate::{
    SolverError, TechniqueGrid,
    technique::{BoxedTechnique, BoxedTechniqueStep, Technique, TechniqueStepData},
};

const NAME: &str = "Hidden Quad";

/// A technique that removes candidates using a hidden quad within a house.
///
/// A "hidden quad" occurs when four digits can only appear in the same four
/// cells of a row, column, or box. Other candidates in those four cells can be
/// removed.
///
/// # Examples
///
/// ```
/// use numelace_solver::{
///     TechniqueGrid,
///     technique::{HiddenQuad, Technique},
/// };
///
/// let mut grid = TechniqueGrid::new();
/// let technique = HiddenQuad::new();
///
/// // Apply the technique
/// let changed = technique.apply(&mut grid)?;
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct HiddenQuad {}

impl HiddenQuad {
    /// Creates a new `HiddenQuad` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl HiddenQuad {
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
            for (d1, remaining_digits1) in DigitSet::FULL.pivots_with_following().take(6) {
                let d1_positions = grid.digit_positions(d1) & house_positions;
                if d1_positions.is_empty() || d1_positions.len() > 4 {
                    continue;
                }
                let digits1 = DigitSet::from_elem(d1);
                for (d2, remaining_digits2) in remaining_digits1
                    .pivots_with_following()
                    .take(remaining_digits1.len() - 2)
                {
                    let d2_positions = grid.digit_positions(d2) & house_positions;
                    if d2_positions.is_empty() {
                        continue;
                    }
                    let pair_positions = d1_positions | d2_positions;
                    if pair_positions.len() > 4 {
                        continue;
                    }
                    let digits12 = digits1 | DigitSet::from_elem(d2);
                    for (d3, remaining_digits3) in remaining_digits2
                        .pivots_with_following()
                        .take(remaining_digits2.len() - 1)
                    {
                        let d3_positions = grid.digit_positions(d3) & house_positions;
                        if d3_positions.is_empty() {
                            continue;
                        }
                        let triple_positions = d1_positions | d2_positions | d3_positions;
                        if triple_positions.len() > 4 {
                            continue;
                        }
                        let digits123 = digits12 | DigitSet::from_elem(d3);
                        for (d4, remaining_digits4) in remaining_digits3.pivots_with_following() {
                            let d4_positions = grid.digit_positions(d4) & house_positions;
                            if d4_positions.is_empty() {
                                continue;
                            }
                            let quad_positions = triple_positions | d4_positions;
                            if quad_positions.len() > 4 {
                                continue;
                            }
                            if quad_positions.len() < 4 {
                                return Err(ConsistencyError::CandidateConstraintViolation.into());
                            }

                            // Digits smaller than `d4` are checked in earlier combinations,
                            // so only the remaining digits need to be validated here.
                            for d in remaining_digits4 {
                                let other_positions = grid.digit_positions(d) & house_positions;
                                if !other_positions.is_empty()
                                    && other_positions.is_subset(quad_positions)
                                {
                                    return Err(
                                        ConsistencyError::CandidateConstraintViolation.into()
                                    );
                                }
                            }

                            let digits1234 = digits123 | DigitSet::from_elem(d4);
                            let eliminate_positions = quad_positions;
                            if grid.remove_candidate_set_with_mask(eliminate_positions, !digits1234)
                                && let ControlFlow::Break(step) =
                                    on_condition(grid, eliminate_positions, digits1234)
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

impl Technique for HiddenQuad {
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
    fn test_eliminates_hidden_quad_candidates_in_row() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(2, 0);
        let pos3 = Position::new(4, 0);
        let pos4 = Position::new(6, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 && pos != pos3 && pos != pos4 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
                grid.remove_candidate(pos, Digit::D4);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&HiddenQuad::new())
            .assert_removed_includes(pos1, [Digit::D5])
            .assert_removed_includes(pos2, [Digit::D5])
            .assert_removed_includes(pos3, [Digit::D5])
            .assert_removed_includes(pos4, [Digit::D5]);
    }

    #[test]
    fn test_find_step_returns_elimination() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(2, 0);
        let pos3 = Position::new(4, 0);
        let pos4 = Position::new(6, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 && pos != pos3 && pos != pos4 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
                grid.remove_candidate(pos, Digit::D4);
            }
        }

        let grid = TechniqueGrid::from(grid);
        let step = HiddenQuad::new().find_step(&grid).unwrap();
        assert!(step.is_some());
    }

    #[test]
    fn test_no_change_when_no_hidden_quads() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&HiddenQuad::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }

    #[test]
    fn test_no_change_when_hidden_quad_has_no_eliminations() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(2, 0);
        let pos3 = Position::new(4, 0);
        let pos4 = Position::new(6, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 && pos != pos3 && pos != pos4 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
                grid.remove_candidate(pos, Digit::D4);
            }
        }

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
            .apply_once(&HiddenQuad::new())
            .assert_no_change(Position::new(1, 0))
            .assert_no_change(Position::new(0, 1));
    }

    #[test]
    fn test_inconsistent_when_five_digits_share_four_positions() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(2, 0);
        let pos3 = Position::new(4, 0);
        let pos4 = Position::new(6, 0);

        for pos in Position::ROWS[0] {
            if pos != pos1 && pos != pos2 && pos != pos3 && pos != pos4 {
                grid.remove_candidate(pos, Digit::D1);
                grid.remove_candidate(pos, Digit::D2);
                grid.remove_candidate(pos, Digit::D3);
                grid.remove_candidate(pos, Digit::D4);
                grid.remove_candidate(pos, Digit::D5);
            }
        }

        let mut grid = TechniqueGrid::from(grid);
        let result = HiddenQuad::new().apply(&mut grid);
        assert!(matches!(
            result,
            Err(SolverError::Inconsistent(
                ConsistencyError::CandidateConstraintViolation
            ))
        ));
    }
}
