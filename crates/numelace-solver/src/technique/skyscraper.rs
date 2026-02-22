use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, Position};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
};

const NAME: &str = "Skyscraper";

/// A technique that removes candidates using a Skyscraper pattern.
///
/// A "Skyscraper" occurs when a digit appears exactly twice in each of two columns
/// (or rows), sharing one base row (or column) while the other candidates lie in
/// the same row-box (or column-box). The digit can then be eliminated from cells
/// that see both roofs.
#[derive(Debug, Default, Clone, Copy)]
pub struct Skyscraper {}

trait AxisOps {
    fn line_positions(index: u8) -> DigitPositions;
    fn cross_positions(index: u8) -> DigitPositions;
    fn cross_index(pos: Position) -> u8;
    fn make_pos(line: u8, cross: u8) -> Position;
}

#[derive(Debug, Clone, Copy)]
struct RowAxis;

#[derive(Debug, Clone, Copy)]
struct ColumnAxis;

impl AxisOps for RowAxis {
    #[inline]
    fn line_positions(index: u8) -> DigitPositions {
        DigitPositions::ROW_POSITIONS[index]
    }

    #[inline]
    fn cross_positions(index: u8) -> DigitPositions {
        DigitPositions::COLUMN_POSITIONS[index]
    }

    #[inline]
    fn cross_index(pos: Position) -> u8 {
        pos.x()
    }

    #[inline]
    fn make_pos(line: u8, cross: u8) -> Position {
        Position::new(cross, line)
    }
}

impl AxisOps for ColumnAxis {
    #[inline]
    fn line_positions(index: u8) -> DigitPositions {
        DigitPositions::COLUMN_POSITIONS[index]
    }

    #[inline]
    fn cross_positions(index: u8) -> DigitPositions {
        DigitPositions::ROW_POSITIONS[index]
    }

    #[inline]
    fn cross_index(pos: Position) -> u8 {
        pos.y()
    }

    #[inline]
    fn make_pos(line: u8, cross: u8) -> Position {
        Position::new(line, cross)
    }
}

impl Skyscraper {
    /// Creates a new `Skyscraper` instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline(always)]
    fn apply_axis_with_control_flow<A, F>(
        grid: &mut TechniqueGrid,
        digit: Digit,
        on_condition: &mut F,
    ) -> Option<BoxedTechniqueStep>
    where
        A: AxisOps,
        F: for<'a> FnMut(
            &'a mut TechniqueGrid,
            Digit,
            (Position, Position),
            (Position, Position),
        ) -> ControlFlow<BoxedTechniqueStep>,
    {
        const INVALID: u8 = u8::MAX;

        let digit_positions = grid.digit_positions(digit);

        let mut lines_with_two = [(INVALID, INVALID, INVALID); 9];
        let mut num_lines = 0usize;
        for line in 0..9u8 {
            let positions = digit_positions & A::line_positions(line);
            let Some((pos_a, pos_b)) = positions.as_double() else {
                continue;
            };
            let cross_a = A::cross_index(pos_a);
            let cross_b = A::cross_index(pos_b);
            if cross_a / 3 == cross_b / 3 {
                continue;
            }
            lines_with_two[num_lines] = (line, cross_a, cross_b);
            num_lines += 1;
        }
        let mut lines_with_two = lines_with_two[..num_lines].iter();
        while let Some(&(line1, line1_cross_a, line1_cross_b)) = lines_with_two.next() {
            for &(line2, line2_cross_a, line2_cross_b) in lines_with_two.as_slice() {
                if line1 / 3 == line2 / 3 {
                    continue;
                }
                if line1_cross_a / 3 != line2_cross_a / 3 || line1_cross_b / 3 != line2_cross_b / 3
                {
                    continue;
                }
                let (base_cross, line1_roof_cross, line2_roof_cross) =
                    if line1_cross_a == line2_cross_a && line1_cross_b != line2_cross_b {
                        (line1_cross_a, line1_cross_b, line2_cross_b)
                    } else if line1_cross_b == line2_cross_b && line1_cross_a != line2_cross_a {
                        (line1_cross_b, line1_cross_a, line2_cross_a)
                    } else {
                        continue;
                    };
                let line1_roof_box = A::make_pos(line1, line1_roof_cross).box_index();
                let line2_roof_box = A::make_pos(line2, line2_roof_cross).box_index();
                let eliminations = (A::cross_positions(line2_roof_cross)
                    & DigitPositions::BOX_POSITIONS[line1_roof_box])
                    | (A::cross_positions(line1_roof_cross)
                        & DigitPositions::BOX_POSITIONS[line2_roof_box]);
                if grid.remove_candidate_with_mask(eliminations, digit)
                    && let ControlFlow::Break(step) = on_condition(
                        grid,
                        digit,
                        (
                            A::make_pos(line1, base_cross),
                            A::make_pos(line2, base_cross),
                        ),
                        (
                            A::make_pos(line1, line1_roof_cross),
                            A::make_pos(line2, line2_roof_cross),
                        ),
                    )
                {
                    return Some(step);
                }
            }
        }
        None
    }

    #[inline]
    fn apply_with_control_flow<F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Option<BoxedTechniqueStep>
    where
        F: for<'a> FnMut(
            &'a mut TechniqueGrid,
            Digit,
            (Position, Position),
            (Position, Position),
        ) -> ControlFlow<BoxedTechniqueStep>,
    {
        for digit in Digit::ALL {
            if let Some(step) =
                Self::apply_axis_with_control_flow::<ColumnAxis, _>(grid, digit, &mut on_condition)
            {
                return Some(step);
            }
            if let Some(step) =
                Self::apply_axis_with_control_flow::<RowAxis, _>(grid, digit, &mut on_condition)
            {
                return Some(step);
            }
        }
        None
    }
}

impl Technique for Skyscraper {
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
            |after_grid, digit, (base_pos1, base_pos2), (ceil_pos1, ceil_pos2)| {
                ControlFlow::Break(Box::new(TechniqueStepData::from_diff(
                    NAME,
                    DigitPositions::from_iter([base_pos1, base_pos2, ceil_pos1, ceil_pos2]),
                    vec![(
                        DigitPositions::from_iter([base_pos1, base_pos2, ceil_pos1, ceil_pos2]),
                        DigitSet::from_elem(digit),
                    )],
                    grid,
                    after_grid,
                )))
            },
        );
        Ok(step)
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let mut changed = false;
        Self::apply_with_control_flow(grid, |_, _, _, _| {
            changed = true;
            ControlFlow::Continue(())
        });
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{CandidateGrid, Digit, Position};

    use super::*;
    use crate::testing::TechniqueTester;

    #[test]
    fn test_eliminates_skyscraper_candidates_in_columns() {
        let mut grid = CandidateGrid::new();
        let digit = Digit::D1;
        let col1 = 1;
        let col2 = 7;
        let base_row = 0;
        let col1_roof_row = 3;
        let col2_roof_row = 4;

        for row in 0..9u8 {
            if row != base_row && row != col1_roof_row {
                grid.remove_candidate(Position::new(col1, row), digit);
            }
        }
        for row in 0..9u8 {
            if row != base_row && row != col2_roof_row {
                grid.remove_candidate(Position::new(col2, row), digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&Skyscraper::new())
            .assert_removed_includes(Position::new(0, 4), [digit])
            .assert_removed_includes(Position::new(8, 3), [digit]);
    }

    #[test]
    fn test_eliminates_skyscraper_candidates_in_rows() {
        let mut grid = CandidateGrid::new();
        let digit = Digit::D1;
        let row1 = 0;
        let row2 = 4;
        let base_col = 0;
        let row1_roof_col = 3;
        let row2_roof_col = 4;

        for col in 0..9u8 {
            if col != base_col && col != row1_roof_col {
                grid.remove_candidate(Position::new(col, row1), digit);
            }
        }
        for col in 0..9u8 {
            if col != base_col && col != row2_roof_col {
                grid.remove_candidate(Position::new(col, row2), digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&Skyscraper::new())
            .assert_removed_includes(Position::new(4, 1), [digit])
            .assert_removed_includes(Position::new(3, 5), [digit]);
    }

    #[test]
    fn test_no_change_when_no_skyscraper() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&Skyscraper::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }
}
