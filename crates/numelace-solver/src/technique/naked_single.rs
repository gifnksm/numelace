use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, Position};

use super::{BoxedTechnique, TechniqueApplication};
use crate::{
    SolverError,
    technique::{BoxedTechniqueStep, Technique, TechniqueGrid, TechniqueStepData},
};

const NAME: &str = "Naked Single";

/// A technique that finds cells with only one remaining candidate and propagates constraints.
///
/// When a cell has only one possible digit (a "naked single"), that digit
/// is placed in that cell, and then constraint propagation is performed by removing
/// that digit from all cells in the same row, column, and box. This combines the
/// simplest Sudoku solving technique with the fundamental constraint propagation mechanism.
///
/// This technique is fundamental to the solver's architecture: it handles all constraint
/// propagation for the system. Other techniques only identify and place digits; the
/// subsequent constraint propagation is performed when control returns to this technique.
///
/// # Examples
///
/// ```
/// use numelace_solver::{
///     TechniqueGrid,
///     technique::{NakedSingle, Technique},
/// };
///
/// let mut grid = TechniqueGrid::new();
/// let technique = NakedSingle::new();
///
/// // Apply the technique
/// let changed = technique.apply(&mut grid)?;
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct NakedSingle {}

impl NakedSingle {
    /// Creates a new `NakedSingle` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Builds a naked single step for a decided position, without gating on eliminations.
    ///
    /// This is useful for hint systems that need to recognize valid placements even
    /// when no candidate elimination would occur in peers.
    #[must_use]
    pub fn build_step(grid: &TechniqueGrid, pos: Position) -> Option<BoxedTechniqueStep> {
        let digit = grid.candidates_at(pos).as_single()?;
        let mut affected_pos = (DigitPositions::ROW_POSITIONS[pos.y()]
            | DigitPositions::COLUMN_POSITIONS[pos.x()]
            | DigitPositions::BOX_POSITIONS[pos.box_index()])
            & grid.digit_positions(digit);
        affected_pos.remove(pos);
        let mut application = vec![TechniqueApplication::CandidateElimination {
            positions: affected_pos,
            digits: DigitSet::from_elem(digit),
        }];
        application.push(TechniqueApplication::Placement {
            position: pos,
            digit,
        });
        Some(Box::new(TechniqueStepData::new(
            NAME,
            DigitPositions::from_elem(pos),
            vec![(DigitPositions::from_elem(pos), DigitSet::from_elem(digit))],
            application,
        )))
    }
}

impl NakedSingle {
    #[inline]
    fn apply_with_control_flow<F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Option<BoxedTechniqueStep>
    where
        F: for<'a> FnMut(&'a mut TechniqueGrid, Position, Digit) -> ControlFlow<BoxedTechniqueStep>,
    {
        let decided_cells = grid.decided_cells();
        for digit in Digit::ALL {
            let decided_digit_positions =
                grid.digit_positions(digit) & decided_cells & !grid.decided_propagated();
            for pos in decided_digit_positions {
                let mut affected_pos = DigitPositions::ROW_POSITIONS[pos.y()]
                    | DigitPositions::COLUMN_POSITIONS[pos.x()]
                    | DigitPositions::BOX_POSITIONS[pos.box_index()];
                affected_pos.remove(pos);
                grid.insert_decided_propagated(pos);
                if grid.remove_candidate_with_mask(affected_pos, digit)
                    && let ControlFlow::Break(value) = on_condition(grid, pos, digit)
                {
                    return Some(value);
                }
            }
        }
        None
    }
}

impl Technique for NakedSingle {
    fn name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        let mut after_grid = grid.clone();
        let step = Self::apply_with_control_flow(&mut after_grid, |after_grid, pos, digit| {
            ControlFlow::Break(Box::new(TechniqueStepData::from_diff_with_extra(
                NAME,
                DigitPositions::from_elem(pos),
                vec![(DigitPositions::from_elem(pos), DigitSet::from_elem(digit))],
                grid,
                after_grid,
                vec![TechniqueApplication::Placement {
                    position: pos,
                    digit,
                }],
            )))
        });
        Ok(step)
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let mut changed = false;
        Self::apply_with_control_flow(grid, |_, _, _| {
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
    fn test_places_naked_single() {
        // When a cell has only one candidate, placing it removes that digit
        // from all cells in the same row, column, and box
        let mut grid = CandidateGrid::new();

        // Make (0, 0) have only D5 as candidate
        grid.place(Position::new(0, 0), Digit::D5);

        TechniqueTester::new(grid)
            .apply_once(&NakedSingle::new())
            // D5 removed from same row
            .assert_removed_exact(Position::new(1, 0), [Digit::D5])
            // D5 removed from same column
            .assert_removed_exact(Position::new(0, 1), [Digit::D5])
            // D5 removed from same box
            .assert_removed_exact(Position::new(1, 1), [Digit::D5]);
    }

    #[test]
    fn test_places_multiple_naked_singles() {
        // Multiple naked singles in different regions are all placed
        let mut grid = CandidateGrid::new();

        // Create naked single at (0, 0) with D3
        grid.place(Position::new(0, 0), Digit::D3);

        // Create naked single at (5, 5) with D7
        grid.place(Position::new(5, 5), Digit::D7);

        TechniqueTester::new(grid)
            .apply_once(&NakedSingle::new())
            // D3 removed from a cell in same row as (0, 0)
            .assert_removed_exact(Position::new(1, 0), [Digit::D3])
            // D7 removed from a cell in same column as (5, 5)
            .assert_removed_exact(Position::new(5, 4), [Digit::D7]);
    }

    #[test]
    fn test_no_change_when_no_naked_singles() {
        // When no cells have a single candidate, nothing changes
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&NakedSingle::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }

    #[test]
    fn test_real_puzzle() {
        // Test with an actual puzzle
        TechniqueTester::from_str(
            "
            53_ _7_ ___
            6__ 195 ___
            _98 ___ _6_
            8__ _6_ __3
            4__ 8_3 __1
            7__ _2_ __6
            _6_ ___ 28_
            ___ 419 __5
            ___ _8_ _79
        ",
        )
        .apply_until_stuck(&NakedSingle::new())
        // Naked singles should be found and placed.
        // Verify at least one placement occurred by checking candidate removal.
        .assert_removed_includes(Position::new(1, 1), [Digit::D4]);
    }
}
