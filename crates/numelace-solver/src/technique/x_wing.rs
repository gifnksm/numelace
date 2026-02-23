use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, Digit, DigitPositions, DigitSet, Position};
use tinyvec::array_vec;

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
    axis::{AxisOps, ColumnAxis, RowAxis},
};

const NAME: &str = "X-Wing";

/// A technique that removes candidates using an X-Wing pattern.
///
/// An "X-Wing" occurs when a digit appears exactly twice in each of two rows
/// (or columns) and those candidate positions align in the same two columns
/// (or rows). The digit can then be eliminated from the other cells in the
/// intersecting columns (or rows).
#[derive(Debug, Default, Clone, Copy)]
pub struct XWing {}

impl XWing {
    /// Creates a new `XWing` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    fn apply_axis_with_control_flow<A, F>(
        grid: &mut TechniqueGrid,
        digit: Digit,
        mut on_condition: F,
    ) -> Result<Option<BoxedTechniqueStep>, SolverError>
    where
        A: AxisOps,
        F: for<'a> FnMut(
            &'a mut TechniqueGrid,
            Digit,
            (u8, u8),
            (u8, u8),
        ) -> ControlFlow<BoxedTechniqueStep>,
    {
        let mut line_masks = array_vec!([(u8, (u8, u8)); 9]);
        for line in 0..9 {
            let Some(crosses) = A::line_mask(grid, line, digit).as_double() else {
                continue;
            };
            line_masks.push((line, crosses));
        }
        let mut line_masks = line_masks.iter();
        while let Some(&(line1, crosses1 @ (cross1, cross2))) = line_masks.next() {
            for &(line2, crosses2) in line_masks.as_slice() {
                if crosses1 != crosses2 {
                    continue;
                }
                // If all four corners land in one box, each row would require a placement
                // while the box allows only one. This is a candidate constraint violation.
                if line1 / 3 == line2 / 3 && cross1 / 3 == cross2 / 3 {
                    return Err(ConsistencyError::CandidateConstraintViolation.into());
                }
                let eliminations = (A::CROSS_POSITIONS[cross1] | A::CROSS_POSITIONS[cross2])
                    & !(A::LINE_POSITIONS[line1] | A::LINE_POSITIONS[line2]);
                if grid.remove_candidate_with_mask(eliminations, digit)
                    && let ControlFlow::Break(value) = {
                        let pos1 = A::make_pos(line1, cross1);
                        let pos2 = A::make_pos(line2, cross2);
                        on_condition(grid, digit, (pos1.x(), pos2.x()), (pos1.y(), pos2.y()))
                    }
                {
                    return Ok(Some(value));
                }
            }
        }
        Ok(None)
    }

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
        for digit in Digit::ALL {
            if let Some(step) =
                Self::apply_axis_with_control_flow::<ColumnAxis, _>(grid, digit, &mut on_condition)?
            {
                return Ok(Some(step));
            }
            if let Some(step) =
                Self::apply_axis_with_control_flow::<RowAxis, _>(grid, digit, &mut on_condition)?
            {
                return Ok(Some(step));
            }
        }
        Ok(None)
    }
}

impl Technique for XWing {
    fn name(&self) -> &'static str {
        NAME
    }

    fn tier(&self) -> TechniqueTier {
        TechniqueTier::UpperIntermediate
    }

    fn clone_box(&self) -> BoxedTechnique {
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

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<usize, SolverError> {
        let mut changed = 0;
        Self::apply_with_control_flow(grid, |_, _, _, _| {
            changed += 1;
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
