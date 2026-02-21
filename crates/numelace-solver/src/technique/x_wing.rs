use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, Digit, DigitPositions, DigitSet, Position};

use crate::{
    SolverError, TechniqueGrid,
    technique::{Technique, TechniqueStepData},
};

use super::BoxedTechniqueStep;

const NAME: &str = "X-Wing";

/// A technique that removes candidates using an X-Wing pattern.
///
/// An "X-Wing" occurs when a digit appears exactly twice in each of two rows
/// (or columns) and those candidate positions align in the same two columns
/// (or rows). The digit can then be eliminated from the other cells in the
/// intersecting columns (or rows).
///
/// # Examples
///
/// ```
/// use numelace_solver::{
///     TechniqueGrid,
///     technique::{Technique, XWing},
/// };
///
/// let mut grid = TechniqueGrid::new();
/// let technique = XWing::new();
///
/// // Apply the technique
/// let changed = technique.apply(&mut grid)?;
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct XWing {}

impl XWing {
    /// Creates a new `XWing` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl XWing {
    #[inline]
    fn apply_with_control_flow<F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Result<Option<BoxedTechniqueStep>, SolverError>
    where
        F: for<'a> FnMut(
            &'a mut TechniqueGrid,
            Digit,
            (u8, u8),
            (u8, u8),
        ) -> ControlFlow<BoxedTechniqueStep>,
    {
        const INVALID: u8 = u8::MAX;

        for digit in Digit::ALL {
            let mut row_count = 0;
            let mut row_masks = [(INVALID, (INVALID, INVALID)); 9];
            for y in 0..9 {
                let Some(xs) = grid.row_mask(y, digit).as_double() else {
                    continue;
                };
                row_masks[row_count] = (y, xs);
                row_count += 1;
            }
            if row_count >= 2 {
                for (i, (y1, xs1 @ (x1, x2))) in
                    row_masks[0..row_count - 1].iter().copied().enumerate()
                {
                    for (y2, xs2) in row_masks[i + 1..].iter().copied() {
                        if xs1 != xs2 {
                            continue;
                        }
                        // If all four corners land in one box, each row would require a placement
                        // while the box allows only one. This is a candidate constraint violation.
                        if y1 / 3 == y2 / 3 && x1 / 3 == x2 / 3 {
                            return Err(ConsistencyError::CandidateConstraintViolation.into());
                        }
                        let mut eliminations = DigitPositions::COLUMN_POSITIONS[x1]
                            | DigitPositions::COLUMN_POSITIONS[x2];
                        eliminations &= !(DigitPositions::ROW_POSITIONS[y1]
                            | DigitPositions::ROW_POSITIONS[y2]);
                        if grid.remove_candidate_with_mask(eliminations, digit)
                            && let ControlFlow::Break(value) =
                                on_condition(grid, digit, (x1, x2), (y1, y2))
                        {
                            return Ok(Some(value));
                        }
                    }
                }
            }

            let mut col_count = 0;
            let mut col_masks = [(INVALID, (INVALID, INVALID)); 9];
            for x in 0..9 {
                let Some(ys) = grid.col_mask(x, digit).as_double() else {
                    continue;
                };
                col_masks[col_count] = (x, ys);
                col_count += 1;
            }
            if col_count >= 2 {
                for (i, (x1, ys1 @ (y1, y2))) in
                    col_masks[0..col_count - 1].iter().copied().enumerate()
                {
                    for (x2, ys2) in col_masks[i + 1..].iter().copied() {
                        if ys1 != ys2 {
                            continue;
                        }
                        // If all four corners land in one box, each column would require a placement
                        // while the box allows only one. This is a candidate constraint violation.
                        if x1 / 3 == x2 / 3 && y1 / 3 == y2 / 3 {
                            return Err(ConsistencyError::CandidateConstraintViolation.into());
                        }
                        let mut eliminations =
                            DigitPositions::ROW_POSITIONS[y1] | DigitPositions::ROW_POSITIONS[y2];
                        eliminations &= !(DigitPositions::COLUMN_POSITIONS[x1]
                            | DigitPositions::COLUMN_POSITIONS[x2]);
                        if grid.remove_candidate_with_mask(eliminations, digit)
                            && let ControlFlow::Break(value) =
                                on_condition(grid, digit, (x1, x2), (y1, y2))
                        {
                            return Ok(Some(value));
                        }
                    }
                }
            }
        }

        Ok(None)
    }
}

impl Technique for XWing {
    fn name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> super::BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        let mut after_grid = grid.clone();
        let step = Self::apply_with_control_flow(
            &mut after_grid,
            |after_grid, digit, (x1, x2), (y1, y2)| {
                let positions = DigitPositions::from_iter([
                    Position::new(x1, y1),
                    Position::new(x2, y1),
                    Position::new(x1, y2),
                    Position::new(x2, y2),
                ]);
                ControlFlow::Break(Box::new(TechniqueStepData::from_diff(
                    NAME,
                    positions,
                    vec![(positions, DigitSet::from_elem(digit))],
                    grid,
                    after_grid,
                )))
            },
        )?;
        Ok(step)
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let mut changed = false;
        Self::apply_with_control_flow(grid, |_, _, _, _| {
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
    fn test_eliminates_x_wing_candidates_in_columns() {
        let mut grid = CandidateGrid::new();
        let x1 = 1;
        let x2 = 7;
        let y1 = 0;
        let y2 = 4;

        for x in 0..9 {
            if x != x1 && x != x2 {
                grid.remove_candidate(Position::new(x, y1), Digit::D1);
                grid.remove_candidate(Position::new(x, y2), Digit::D1);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&XWing::new())
            .assert_removed_includes(Position::new(x1, 2), [Digit::D1])
            .assert_removed_includes(Position::new(x2, 6), [Digit::D1]);
    }

    #[test]
    fn test_no_change_when_no_x_wing() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&XWing::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }

    #[test]
    fn test_inconsistent_when_x_wing_in_same_box() {
        let mut grid = CandidateGrid::new();
        let x1 = 0;
        let x2 = 1;
        let y1 = 0;
        let y2 = 1;

        for x in 0..9 {
            if x != x1 && x != x2 {
                grid.remove_candidate(Position::new(x, y1), Digit::D1);
                grid.remove_candidate(Position::new(x, y2), Digit::D1);
            }
        }

        let mut grid = TechniqueGrid::from(grid);
        let result = XWing::new().apply(&mut grid);
        assert!(matches!(
            result,
            Err(SolverError::Inconsistent(
                ConsistencyError::CandidateConstraintViolation
            ))
        ));
    }
}
