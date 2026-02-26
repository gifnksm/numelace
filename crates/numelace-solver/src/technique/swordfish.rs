use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, Digit, DigitPositions, DigitSet, House, HouseMask};
use tinyvec::array_vec;

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
    axis::{AxisOps, ColumnAxis, RowAxis},
};

const NAME: &str = "Swordfish";

/// A technique that removes candidates using a Swordfish pattern.
///
/// A "Swordfish" occurs when a digit appears in 2-3 cells across each of three rows
/// (or columns) and those candidates align within the same three columns (or rows).
/// The digit can then be eliminated from the other cells in the cover columns (or rows).
#[derive(Debug, Default, Clone, Copy)]
pub struct Swordfish {}

#[derive(Debug, Clone, Copy)]
struct Condition {
    digit: Digit,
    base_houses: [House; 3],
    cover_houses: [House; 3],
}

impl Condition {
    fn build_step(
        &self,
        before_grid: &TechniqueGrid,
        after_grid: &TechniqueGrid,
    ) -> BoxedTechniqueStep {
        let condition_cells = self.base_houses.into_iter().map(House::positions).sum();
        let crosses = self.cover_houses.into_iter().map(House::positions).sum();
        let cross_cells = self
            .base_houses
            .into_iter()
            .map(|house| house.positions() & crosses)
            .sum::<DigitPositions>();
        let condition_digit_cells = vec![(
            cross_cells & before_grid.digit_positions(self.digit),
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

impl Swordfish {
    /// Creates a new `Swordfish` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    fn apply_axis_with_control_flow<A, T, F>(
        grid: &mut TechniqueGrid,
        digit: Digit,
        mut on_condition: F,
    ) -> Result<Option<T>, SolverError>
    where
        A: AxisOps,
        F: for<'a> FnMut(&'a mut TechniqueGrid, &'a Condition) -> ControlFlow<T>,
    {
        let mut line_masks = array_vec!([(u8, HouseMask); 9]);
        for line in 0..9 {
            let mask = A::line_mask(grid, line, digit);
            if !(1..=3).contains(&mask.len()) {
                continue;
            }
            line_masks.push((line, mask));
        }
        let mut line_masks = line_masks.iter();
        while let Some(&(line1, crosses1)) = line_masks.next() {
            let mut line_masks2 = line_masks.as_slice().iter();
            while let Some(&(line2, crosses2)) = line_masks2.next() {
                if (crosses1 | crosses2).len() > 3 {
                    continue;
                }
                for &(line3, crosses3) in line_masks2.as_slice() {
                    let crosses = crosses1 | crosses2 | crosses3;
                    if crosses.len() > 3 {
                        continue;
                    }
                    let Some([cross1, cross2, cross3]) = crosses.as_triple() else {
                        return Err(ConsistencyError::CandidateConstraintViolation.into());
                    };
                    if line1 / 3 == line2 / 3
                        && line2 / 3 == line3 / 3
                        && cross1 / 3 == cross2 / 3
                        && cross2 / 3 == cross3 / 3
                    {
                        return Err(ConsistencyError::CandidateConstraintViolation.into());
                    }
                    let eliminations = (A::CROSS_POSITIONS[cross1]
                        | A::CROSS_POSITIONS[cross2]
                        | A::CROSS_POSITIONS[cross3])
                        & !(A::LINE_POSITIONS[line1]
                            | A::LINE_POSITIONS[line2]
                            | A::LINE_POSITIONS[line3]);
                    if grid.remove_candidate_with_mask(eliminations, digit)
                        && let ControlFlow::Break(value) = on_condition(
                            grid,
                            &Condition {
                                digit,
                                base_houses: [
                                    A::LINE_HOUSES[line1],
                                    A::LINE_HOUSES[line2],
                                    A::LINE_HOUSES[line3],
                                ],
                                cover_houses: [
                                    A::CROSS_HOUSES[cross1],
                                    A::CROSS_HOUSES[cross2],
                                    A::CROSS_HOUSES[cross3],
                                ],
                            },
                        )
                    {
                        return Ok(Some(value));
                    }
                }
            }
        }
        Ok(None)
    }

    #[inline]
    fn apply_with_control_flow<T, F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Result<Option<T>, SolverError>
    where
        F: for<'a> FnMut(&'a mut TechniqueGrid, &'a Condition) -> ControlFlow<T>,
    {
        for digit in Digit::ALL {
            if let Some(result) = Self::apply_axis_with_control_flow::<ColumnAxis, _, _>(
                grid,
                digit,
                &mut on_condition,
            )? {
                return Ok(Some(result));
            }
            if let Some(result) =
                Self::apply_axis_with_control_flow::<RowAxis, _, _>(grid, digit, &mut on_condition)?
            {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }
}

impl Technique for Swordfish {
    fn name(&self) -> &'static str {
        NAME
    }

    fn tier(&self) -> TechniqueTier {
        TechniqueTier::Advanced
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
    fn test_eliminates_swordfish_candidates_in_rows() {
        let mut grid = CandidateGrid::new();
        let digit = Digit::D1;
        let rows = [0u8, 4, 8];
        let cols = [1u8, 4, 7];

        for &row in &rows {
            for x in 0..9u8 {
                if !cols.contains(&x) {
                    grid.remove_candidate(Position::new(x, row), digit);
                }
            }
        }

        TechniqueTester::new(grid)
            .apply_pass(&Swordfish::new())
            .assert_removed_includes(Position::new(cols[0], 2), [digit])
            .assert_removed_includes(Position::new(cols[2], 6), [digit]);
    }

    #[test]
    fn test_no_change_when_no_swordfish() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_pass(&Swordfish::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }

    #[test]
    fn test_inconsistent_when_only_two_cover_columns() {
        let mut grid = CandidateGrid::new();
        let digit = Digit::D1;
        let rows = [0u8, 3, 6];
        let cols = [1u8, 5];

        for &row in &rows {
            for x in 0..9u8 {
                if !cols.contains(&x) {
                    grid.remove_candidate(Position::new(x, row), digit);
                }
            }
        }

        let mut grid = TechniqueGrid::from(grid);
        let result = Swordfish::new().apply_pass(&mut grid);
        assert!(matches!(
            result,
            Err(SolverError::Inconsistent(
                ConsistencyError::CandidateConstraintViolation
            ))
        ));
    }
}
