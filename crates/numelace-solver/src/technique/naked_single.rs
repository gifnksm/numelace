use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, Position};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueApplication,
    TechniqueGrid, TechniqueStepData, TechniqueTier,
};

const ID: &str = "naked_single";
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
#[derive(Debug, Default, Clone, Copy)]
pub struct NakedSingle {}

struct Condition {
    digit: Digit,
    position: Position,
}

impl Condition {
    fn build_step(
        &self,
        before_grid: &TechniqueGrid,
        after_grid: &TechniqueGrid,
    ) -> BoxedTechniqueStep {
        let condition_positions = DigitPositions::from_elem(self.position);
        let condition_digit_positions = vec![(
            DigitPositions::from_elem(self.position),
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

impl NakedSingle {
    /// Creates a new `NakedSingle` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Builds a naked single step for a univalue position, without gating on eliminations.
    ///
    /// This is useful for hint systems that need to recognize valid placements even
    /// when no candidate elimination would occur in peers.
    #[must_use]
    pub fn build_step(grid: &TechniqueGrid, pos: Position) -> Option<BoxedTechniqueStep> {
        let digit = grid.candidates_at(pos).as_single()?;
        let mut affected_pos = (DigitPositions::ROW_POSITIONS[pos.row()]
            | DigitPositions::COL_POSITIONS[pos.col()]
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
        Some(TechniqueStepData::new_boxed(
            NAME,
            DigitPositions::from_elem(pos),
            vec![(DigitPositions::from_elem(pos), DigitSet::from_elem(digit))],
            application,
        ))
    }

    #[inline]
    fn apply_with_control_flow<T, F>(grid: &mut TechniqueGrid, mut on_condition: F) -> Option<T>
    where
        F: for<'a> FnMut(&'a mut TechniqueGrid, &'a Condition) -> ControlFlow<T>,
    {
        let univalue_positions = grid.univalue_positions() & !grid.univalue_propagated();
        for digit in Digit::ALL {
            let univalue_positions = grid.digit_positions(digit) & univalue_positions;
            for pos in univalue_positions {
                let mut affected_pos = DigitPositions::ROW_POSITIONS[pos.row()]
                    | DigitPositions::COL_POSITIONS[pos.col()]
                    | DigitPositions::BOX_POSITIONS[pos.box_index()];
                affected_pos.remove(pos);
                grid.insert_univalue_propagated(pos);
                if grid.remove_candidate_with_mask(affected_pos, digit)
                    && let ControlFlow::Break(value) = on_condition(
                        grid,
                        &Condition {
                            position: pos,
                            digit,
                        },
                    )
                {
                    return Some(value);
                }
            }
        }
        None
    }
}

impl Technique for NakedSingle {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn tier(&self) -> TechniqueTier {
        TechniqueTier::Fundamental
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
    use std::str::FromStr as _;

    use numelace_core::{CandidateGrid, Digit, DigitGrid, Position};

    use super::*;
    use crate::testing;

    const TECHNIQUE: NakedSingle = NakedSingle::new();

    #[test]
    fn test_places_naked_single() {
        // When a cell has only one candidate, placing it removes that digit
        // from all cells in the same row, column, and box
        let mut grid = CandidateGrid::new();

        // Make (0, 0) have only D5 as candidate
        grid.place(Position::new(0, 0), Digit::D5);

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t
                // D5 removed from same row
                .assert_removed_exact(Position::new(0, 1), [Digit::D5])
                // D5 removed from same column
                .assert_removed_exact(Position::new(1, 0), [Digit::D5])
                // D5 removed from same box
                .assert_removed_exact(Position::new(1, 1), [Digit::D5]);
        });
    }

    #[test]
    fn test_places_multiple_naked_singles() {
        // Multiple naked singles in different regions are all placed
        let mut grid = CandidateGrid::new();

        // Create naked single at (0, 0) with D3
        grid.place(Position::new(0, 0), Digit::D3);

        // Create naked single at (5, 5) with D7
        grid.place(Position::new(5, 5), Digit::D7);

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t
                // D3 removed from a cell in same row as (0, 0)
                .assert_removed_exact(Position::new(0, 1), [Digit::D3])
                // D7 removed from a cell in same column as (5, 5)
                .assert_removed_exact(Position::new(4, 5), [Digit::D7]);
        });
    }

    #[test]
    fn test_no_change_when_no_naked_singles() {
        // When no cells have a single candidate, nothing changes
        let grid = CandidateGrid::new();
        testing::test_technique_apply_pass_no_changes(grid, &TECHNIQUE);
    }

    #[test]
    fn test_real_puzzle() {
        // Test with an actual puzzle
        let grid = DigitGrid::from_str(
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
        .unwrap();
        testing::test_technique_apply_until_stuck(grid, &TECHNIQUE, |t| {
            t
                // Naked singles should be found and placed.
                // Verify at least one placement occurred by checking candidate removal.
                .assert_removed_includes(Position::new(1, 1), [Digit::D4]);
        });
    }
}
