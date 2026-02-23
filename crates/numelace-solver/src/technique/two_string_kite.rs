use std::{iter, ops::ControlFlow};

use numelace_core::{
    CellIndexIndexedArray, Digit, DigitPositions, DigitSet, Position, containers::Array9,
};
use tinyvec::ArrayVec;

use crate::{
    BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData, TechniqueTier,
    axis::{AxisOps, ColumnAxis, RowAxis},
};

const NAME: &str = "2-String Kite";

/// A technique that removes candidates using a 2-String Kite pattern.
///
/// A "2-String Kite" occurs when a digit appears exactly twice in a row and
/// exactly twice in a column, and one candidate from each lies in the same
/// 3x3 box. The digit can then be eliminated from the cell at the intersection
/// of the other row candidate and the other column candidate.
#[derive(Debug, Default, Clone, Copy)]
pub struct TwoStringKite {}

type LinePair = (u8, (u8, u8));
type LinePairs = ArrayVec<[LinePair; 3]>;

impl TwoStringKite {
    /// Creates a new `TwoStringKite` instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    fn collect_line_pairs_by_box<A: AxisOps>(
        grid: &TechniqueGrid,
        digit: Digit,
    ) -> (CellIndexIndexedArray<LinePairs>, u8) {
        let digit_positions = grid.digit_positions(digit);
        let mut line_pairs = Array9::from_array([LinePairs::new(); 9]);
        let mut found_lines = 0;
        for line in 0..9 {
            let positions = digit_positions & A::LINE_POSITIONS[line];
            let Some((pos_a, pos_b)) = positions.as_double() else {
                continue;
            };
            let cross_a = A::cross_index(pos_a);
            let cross_b = A::cross_index(pos_b);
            if cross_a / 3 == cross_b / 3 {
                continue;
            }
            found_lines += 1;
            line_pairs[pos_a.box_index()].push((line, (cross_a, cross_b)));
            line_pairs[pos_b.box_index()].push((line, (cross_b, cross_a)));
        }
        (line_pairs, found_lines)
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
            LinePair,
            LinePair,
        ) -> ControlFlow<BoxedTechniqueStep>,
    {
        for digit in Digit::ALL {
            let (box_rows, found_rows) = Self::collect_line_pairs_by_box::<RowAxis>(grid, digit);
            if found_rows == 0 {
                continue;
            }
            let (box_cols, found_cols) = Self::collect_line_pairs_by_box::<ColumnAxis>(grid, digit);
            if found_cols == 0 {
                continue;
            }
            for (rows, cols) in iter::zip(box_rows, box_cols) {
                for (row, (row_box_col, row_other_col)) in rows {
                    for (col, (col_box_row, col_other_row)) in cols {
                        if row == col_box_row && row_box_col == col {
                            continue;
                        }
                        let eliminate_pos = Position::new(row_other_col, col_other_row);
                        if grid.remove_candidate(eliminate_pos, digit)
                            && let ControlFlow::Break(step) = on_condition(
                                grid,
                                digit,
                                (row, (row_box_col, row_other_col)),
                                (col, (col_box_row, col_other_row)),
                            )
                        {
                            return Some(step);
                        }
                    }
                }
            }
        }
        None
    }
}

impl Technique for TwoStringKite {
    fn name(&self) -> &'static str {
        NAME
    }

    fn tier(&self) -> TechniqueTier {
        TechniqueTier::UpperIntermediate
    }

    fn clone_box(&self) -> Box<dyn Technique> {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        let mut after_grid = grid.clone();
        let step = Self::apply_with_control_flow(
            &mut after_grid,
            |after_grid,
             digit,
             (row, (row_box_col, row_other_col)),
             (col, (col_box_row, col_other_row))| {
                let condition_cells = DigitPositions::from_iter([
                    Position::new(row_box_col, row),
                    Position::new(row_other_col, row),
                    Position::new(col, col_box_row),
                    Position::new(col, col_other_row),
                ]);
                ControlFlow::Break(Box::new(TechniqueStepData::from_diff(
                    NAME,
                    condition_cells,
                    vec![(condition_cells, DigitSet::from_elem(digit))],
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
    fn test_eliminates_two_string_kite_candidates() {
        let mut grid = CandidateGrid::new();
        let digit = Digit::D1;
        let row = 0;
        let row_box_col = 1;
        let row_other_col = 4;
        let col = 2;
        let col_box_row = 1;
        let col_other_row = 4;

        for x in 0..9u8 {
            if x != row_box_col && x != row_other_col {
                grid.remove_candidate(Position::new(x, row), digit);
            }
        }
        for y in 0..9u8 {
            if y != col_box_row && y != col_other_row {
                grid.remove_candidate(Position::new(col, y), digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&TwoStringKite::new())
            .assert_removed_includes(Position::new(row_other_col, col_other_row), [digit]);
    }

    #[test]
    fn test_shared_cell_skips_elimination() {
        let mut grid = CandidateGrid::new();
        let digit = Digit::D1;
        let row = 0;
        let row_box_col = 1;
        let row_other_col = 4;
        let col = 1;
        let col_box_row = 0;
        let col_other_row = 4;

        for x in 0..9u8 {
            if x != row_box_col && x != row_other_col {
                grid.remove_candidate(Position::new(x, row), digit);
            }
        }
        for y in 0..9u8 {
            if y != col_box_row && y != col_other_row {
                grid.remove_candidate(Position::new(col, y), digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&TwoStringKite::new())
            .assert_no_change(Position::new(row_other_col, col_other_row));
    }

    #[test]
    fn test_no_change_when_no_two_string_kite() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&TwoStringKite::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }
}
