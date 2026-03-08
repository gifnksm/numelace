use std::ops::ControlFlow;

use numelace_core::{ConsistencyError, Digit, DigitPositions, DigitSet, House, HouseMask};
use tinyvec::array_vec;

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
    axis::{AxisOps, ColumnAxis, RowAxis},
};

const ID: &str = "jellyfish";
const NAME: &str = "Jellyfish";

/// A technique that removes candidates using a Jellyfish pattern.
///
/// A "Jellyfish" occurs when a digit appears in 2-4 cells across each of four rows
/// (or columns) and those candidates align within the same four columns (or rows).
/// The digit can then be eliminated from the other cells in the cover columns (or rows).
#[derive(Debug, Default, Clone, Copy)]
pub struct Jellyfish {}

#[derive(Debug, Clone, Copy)]
struct Condition {
    digit: Digit,
    base_houses: [House; 4],
    cover_houses: [House; 4],
}

impl Condition {
    fn build_step(
        &self,
        before_grid: &TechniqueGrid,
        after_grid: &TechniqueGrid,
    ) -> BoxedTechniqueStep {
        let condition_positions = self.base_houses.into_iter().map(House::positions).sum();
        let crosses = self.cover_houses.into_iter().map(House::positions).sum();
        let cross_positions = self
            .base_houses
            .into_iter()
            .map(|house| house.positions() & crosses)
            .sum::<DigitPositions>();
        let condition_digit_positions = vec![(
            cross_positions & before_grid.digit_positions(self.digit),
            DigitSet::from_elem(self.digit),
        )];
        TechniqueStepData::from_diff(
            NAME,
            condition_positions,
            condition_digit_positions,
            before_grid,
            after_grid,
        )
    }
}

impl Jellyfish {
    /// Creates a new `Jellyfish` technique.
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
            if !(1..=4).contains(&mask.len()) {
                continue;
            }
            line_masks.push((line, mask));
        }
        let mut line_masks = line_masks.iter();
        while let Some(&(line1, crosses1)) = line_masks.next() {
            let mut line_masks2 = line_masks.as_slice().iter();
            while let Some(&(line2, crosses2)) = line_masks2.next() {
                let crosses12 = crosses1 | crosses2;
                if crosses12.len() > 4 {
                    continue;
                }
                let mut line_masks3 = line_masks2.as_slice().iter();
                while let Some(&(line3, crosses3)) = line_masks3.next() {
                    let crosses123 = crosses12 | crosses3;
                    if crosses123.len() > 4 {
                        continue;
                    }
                    for &(line4, crosses4) in line_masks3.as_slice() {
                        let crosses1234 = crosses123 | crosses4;
                        if crosses1234.len() > 4 {
                            continue;
                        }
                        // If four base houses only cover fewer than four crosses, each base house
                        // would still require a placement while the cover set cannot host them all.
                        // This is a candidate constraint violation.
                        let Some([cross1, cross2, cross3, cross4]) = crosses1234.as_quad() else {
                            return Err(ConsistencyError::CandidateConstraintViolation.into());
                        };
                        let eliminations = (A::CROSS_POSITIONS[cross1]
                            | A::CROSS_POSITIONS[cross2]
                            | A::CROSS_POSITIONS[cross3]
                            | A::CROSS_POSITIONS[cross4])
                            & !(A::LINE_POSITIONS[line1]
                                | A::LINE_POSITIONS[line2]
                                | A::LINE_POSITIONS[line3]
                                | A::LINE_POSITIONS[line4]);
                        if grid.remove_candidate_with_mask(eliminations, digit)
                            && let ControlFlow::Break(value) = on_condition(
                                grid,
                                &Condition {
                                    digit,
                                    base_houses: [
                                        A::LINE_HOUSES[line1],
                                        A::LINE_HOUSES[line2],
                                        A::LINE_HOUSES[line3],
                                        A::LINE_HOUSES[line4],
                                    ],
                                    cover_houses: [
                                        A::CROSS_HOUSES[cross1],
                                        A::CROSS_HOUSES[cross2],
                                        A::CROSS_HOUSES[cross3],
                                        A::CROSS_HOUSES[cross4],
                                    ],
                                },
                            )
                        {
                            return Ok(Some(value));
                        }
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

impl Technique for Jellyfish {
    fn id(&self) -> &'static str {
        ID
    }

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
    use numelace_core::{CandidateGrid, Digit, Position};

    use super::*;
    use crate::testing;

    const TECHNIQUE: Jellyfish = Jellyfish::new();

    #[test]
    fn test_eliminates_jellyfish_candidates_in_rows() {
        let mut grid = CandidateGrid::new();
        let digit = Digit::D1;
        let rows = [0u8, 2, 5, 8];
        let cols = [1u8, 4, 6, 8];

        for &row in &rows {
            for x in 0..9u8 {
                if !cols.contains(&x) {
                    grid.remove_candidate(Position::from_xy(x, row), digit);
                }
            }
        }

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t.assert_removed_includes(Position::from_xy(cols[0], 1), [digit])
                .assert_removed_includes(Position::from_xy(cols[2], 7), [digit]);
        });
    }

    #[test]
    fn test_no_change_when_no_jellyfish() {
        let grid = CandidateGrid::new();
        testing::test_technique_apply_pass_no_changes(grid, &TECHNIQUE);
    }

    #[test]
    fn test_inconsistent_when_only_three_cover_columns() {
        let mut grid = CandidateGrid::new();
        let digit = Digit::D1;
        let rows = [0u8, 2, 5, 8];
        let cols = [1u8, 5, 7];

        for &row in &rows {
            for x in 0..9u8 {
                if !cols.contains(&x) {
                    grid.remove_candidate(Position::from_xy(x, row), digit);
                }
            }
        }
        testing::test_technique_apply_pass_fail_with_constraint_violation(grid, &TECHNIQUE);
    }
}
