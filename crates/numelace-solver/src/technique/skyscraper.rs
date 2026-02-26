use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, House, Position};
use tinyvec::array_vec;

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
    axis::{AxisOps, ColumnAxis, RowAxis},
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

struct Condition {
    digit: Digit,
    lines: [House; 2],
    base_positions: [Position; 2],
    roof_positions: [Position; 2],
}

impl Condition {
    fn build_step(
        &self,
        before_grid: &TechniqueGrid,
        after_grid: &TechniqueGrid,
    ) -> BoxedTechniqueStep {
        let condition_cells = self.lines.into_iter().map(House::positions).sum();
        let condition_digit_cells = vec![(
            DigitPositions::from_iter(self.base_positions)
                | DigitPositions::from_iter(self.roof_positions),
            DigitSet::from_elem(self.digit),
        )];
        TechniqueStepData::from_diff(
            NAME,
            condition_cells,
            condition_digit_cells,
            before_grid,
            after_grid,
        )
    }
}

impl Skyscraper {
    /// Creates a new `Skyscraper` instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    fn apply_axis_with_control_flow<A, T, F>(
        grid: &mut TechniqueGrid,
        digit: Digit,
        on_condition: &mut F,
    ) -> Option<T>
    where
        A: AxisOps,
        F: for<'a> FnMut(&'a mut TechniqueGrid, &'a Condition) -> ControlFlow<T>,
    {
        let digit_positions = grid.digit_positions(digit);

        let mut lines_with_two = array_vec!([(u8, u8, u8); 9]);
        for line in 0..9 {
            let positions = digit_positions & A::LINE_POSITIONS[line];
            let Some([pos_a, pos_b]) = positions.as_double() else {
                continue;
            };
            let cross_a = A::cross_index(pos_a);
            let cross_b = A::cross_index(pos_b);
            if cross_a / 3 == cross_b / 3 {
                continue;
            }
            lines_with_two.push((line, cross_a, cross_b));
        }
        let mut lines_with_two = lines_with_two.iter();
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
                let eliminations = (A::CROSS_POSITIONS[line2_roof_cross]
                    & DigitPositions::BOX_POSITIONS[line1_roof_box])
                    | (A::CROSS_POSITIONS[line1_roof_cross]
                        & DigitPositions::BOX_POSITIONS[line2_roof_box]);
                if grid.remove_candidate_with_mask(eliminations, digit)
                    && let ControlFlow::Break(step) = on_condition(
                        grid,
                        &Condition {
                            digit,
                            lines: [A::LINE_HOUSES[line1], A::LINE_HOUSES[line2]],
                            base_positions: [
                                A::make_pos(line1, base_cross),
                                A::make_pos(line2, base_cross),
                            ],
                            roof_positions: [
                                A::make_pos(line1, line1_roof_cross),
                                A::make_pos(line2, line2_roof_cross),
                            ],
                        },
                    )
                {
                    return Some(step);
                }
            }
        }
        None
    }

    #[inline]
    fn apply_with_control_flow<T, F>(grid: &mut TechniqueGrid, mut on_condition: F) -> Option<T>
    where
        F: for<'a> FnMut(&'a mut TechniqueGrid, &'a Condition) -> ControlFlow<T>,
    {
        for digit in Digit::ALL {
            if let Some(step) = Self::apply_axis_with_control_flow::<ColumnAxis, T, _>(
                grid,
                digit,
                &mut on_condition,
            ) {
                return Some(step);
            }
            if let Some(step) =
                Self::apply_axis_with_control_flow::<RowAxis, T, _>(grid, digit, &mut on_condition)
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
        let step = Self::apply_with_control_flow(&mut after_grid, |after_grid, condition| {
            ControlFlow::Break(condition.build_step(grid, after_grid))
        });
        Ok(step)
    }

    fn apply_step(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let changed = Self::apply_with_control_flow(grid, |_, _| ControlFlow::Break(())).is_some();
        Ok(changed)
    }

    fn apply_pass(&self, grid: &mut TechniqueGrid) -> Result<usize, SolverError> {
        let mut changed = 0;
        Self::apply_with_control_flow(grid, |_, _| {
            changed += 1;
            ControlFlow::<()>::Continue(())
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

        for row in 0..9 {
            if row != base_row && row != col1_roof_row {
                grid.remove_candidate(Position::new(col1, row), digit);
            }
        }
        for row in 0..9 {
            if row != base_row && row != col2_roof_row {
                grid.remove_candidate(Position::new(col2, row), digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_pass(&Skyscraper::new())
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

        for col in 0..9 {
            if col != base_col && col != row1_roof_col {
                grid.remove_candidate(Position::new(col, row1), digit);
            }
        }
        for col in 0..9 {
            if col != base_col && col != row2_roof_col {
                grid.remove_candidate(Position::new(col, row2), digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_pass(&Skyscraper::new())
            .assert_removed_includes(Position::new(4, 1), [digit])
            .assert_removed_includes(Position::new(3, 5), [digit]);
    }

    #[test]
    fn test_no_change_when_no_skyscraper() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_pass(&Skyscraper::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }
}
