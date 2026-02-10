use numelace_core::{CandidateGrid, Digit, DigitPositions, DigitSet, Position};

use super::{BoxedTechnique, TechniqueApplication};
use crate::{
    SolverError,
    technique::{BoxedTechniqueStep, Technique, TechniqueStep},
};

const NAME: &str = "naked single";

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
/// use numelace_core::CandidateGrid;
/// use numelace_solver::technique::{NakedSingle, Technique};
///
/// let mut grid = CandidateGrid::new();
/// let technique = NakedSingle::new();
///
/// // Apply the technique
/// let changed = technique.apply(&mut grid)?;
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct NakedSingle;

impl NakedSingle {
    /// Creates a new `NakedSingle` technique.
    #[must_use]
    pub const fn new() -> Self {
        NakedSingle
    }
}

#[derive(Debug, Clone)]
pub struct NakedSingleStep {
    position: Position,
    digit: Digit,
    affected_positions: DigitPositions,
}

impl NakedSingleStep {
    fn new(position: Position, digit: Digit, affected_positions: DigitPositions) -> Self {
        Self {
            position,
            digit,
            affected_positions,
        }
    }
}

impl TechniqueStep for NakedSingleStep {
    fn technique_name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> BoxedTechniqueStep {
        Box::new(self.clone())
    }

    fn condition_cells(&self) -> DigitPositions {
        DigitPositions::from_elem(self.position)
    }

    fn condition_digit_cells(&self) -> Vec<(DigitPositions, DigitSet)> {
        vec![(
            DigitPositions::from_elem(self.position),
            DigitSet::from_elem(self.digit),
        )]
    }

    fn application(&self) -> Vec<TechniqueApplication> {
        vec![
            TechniqueApplication::Placement {
                position: self.position,
                digit: self.digit,
            },
            TechniqueApplication::CandidateElimination {
                positions: self.affected_positions,
                digits: DigitSet::from_elem(self.digit),
            },
        ]
    }
}

impl Technique for NakedSingle {
    fn name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &CandidateGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        let decided_cells = grid.decided_cells();
        for digit in Digit::ALL {
            for pos in grid.digit_positions(digit) & decided_cells {
                let mut affected_pos = (DigitPositions::ROW_POSITIONS[pos.y()]
                    | DigitPositions::COLUMN_POSITIONS[pos.x()]
                    | DigitPositions::BOX_POSITIONS[pos.box_index()])
                    & grid.digit_positions(digit);
                affected_pos.remove(pos);
                if grid.would_remove_candidate_with_mask_change(affected_pos, digit) {
                    return Ok(Some(Box::new(NakedSingleStep::new(
                        pos,
                        digit,
                        affected_pos,
                    ))));
                }
            }
        }
        Ok(None)
    }

    fn apply(&self, grid: &mut CandidateGrid) -> Result<bool, SolverError> {
        let mut changed = false;

        let decided_cells = grid.decided_cells();
        for digit in Digit::ALL {
            for pos in grid.digit_positions(digit) & decided_cells {
                let mut affected_pos = DigitPositions::ROW_POSITIONS[pos.y()]
                    | DigitPositions::COLUMN_POSITIONS[pos.x()]
                    | DigitPositions::BOX_POSITIONS[pos.box_index()];
                affected_pos.remove(pos);
                changed |= grid.remove_candidate_with_mask(affected_pos, digit);
            }
        }

        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{CandidateGrid, Digit, DigitPositions, DigitSet, Position};

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
    fn test_find_step_matches_apply() {
        let mut grid = CandidateGrid::new();

        // Make (0, 0) have only D5 as candidate
        grid.place(Position::new(0, 0), Digit::D5);

        let technique = NakedSingle::new();
        let step = technique.find_step(&grid).unwrap().expect("expected step");

        let condition_cells = step.condition_cells();
        assert_eq!(
            condition_cells,
            DigitPositions::from_elem(Position::new(0, 0))
        );

        let condition_digit_cells = step.condition_digit_cells();
        assert_eq!(condition_digit_cells.len(), 1);
        assert_eq!(
            condition_digit_cells[0],
            (
                DigitPositions::from_elem(Position::new(0, 0)),
                DigitSet::from_elem(Digit::D5)
            )
        );

        let applications = step.application();
        assert_eq!(applications.len(), 1);
        match applications[0] {
            TechniqueApplication::CandidateElimination { positions, digits } => {
                assert!(!positions.contains(Position::new(0, 0)));
                assert!(positions.contains(Position::new(1, 0)));
                assert!(positions.contains(Position::new(0, 1)));
                assert!(positions.contains(Position::new(1, 1)));
                assert_eq!(digits, DigitSet::from_elem(Digit::D5));
            }
            TechniqueApplication::Placement { .. } => panic!("expected candidate elimination step"),
        }

        TechniqueTester::new(grid)
            .apply_once(&NakedSingle::new())
            .assert_removed_exact(Position::new(1, 0), [Digit::D5])
            .assert_removed_exact(Position::new(0, 1), [Digit::D5])
            .assert_removed_exact(Position::new(1, 1), [Digit::D5]);
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
