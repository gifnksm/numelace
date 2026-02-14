use numelace_core::{Digit, DigitPositions, DigitSet, House, Position};

use super::{
    BoxedTechnique, BoxedTechniqueStep, ConditionCells, ConditionDigitCells, TechniqueApplication,
    TechniqueGrid,
};
use crate::{
    SolverError,
    technique::{Technique, TechniqueStep},
};

const NAME: &str = "Locked Candidates";
const NAME_POINTING: &str = "Locked Candidates (Pointing)";
const NAME_CLAIMING: &str = "Locked Candidates (Claiming)";

/// A technique that removes candidates using locked candidates (pointing/claiming).
///
/// - **Pointing**: Within a box, all candidates of a digit lie in a single row/column,
///   so that digit can be removed from the rest of that row/column outside the box.
/// - **Claiming**: Within a row/column, all candidates of a digit lie in a single box,
///   so that digit can be removed from the rest of that box outside the row/column.
///
/// # Examples
///
/// ```
/// use numelace_solver::technique::{LockedCandidates, Technique, TechniqueGrid};
///
/// let mut grid = TechniqueGrid::new();
/// let technique = LockedCandidates::new();
///
/// // Apply the technique
/// let changed = technique.apply(&mut grid)?;
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct LockedCandidates {}

impl LockedCandidates {
    /// Creates a new `LockedCandidates` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LockedCandidatesKind {
    Pointing,
    Claiming,
}

#[derive(Debug, Clone)]
pub struct LockedCandidatesStep {
    kind: LockedCandidatesKind,
    digit: Digit,
    box_index: u8,
    row_or_col: House,
    intersection_cells: DigitPositions,
    eliminations: DigitPositions,
}

impl LockedCandidatesStep {
    fn pointing(
        digit: Digit,
        box_index: u8,
        row_or_col: House,
        intersection_cells: DigitPositions,
        eliminations: DigitPositions,
    ) -> Self {
        Self {
            kind: LockedCandidatesKind::Pointing,
            digit,
            box_index,
            row_or_col,
            intersection_cells,
            eliminations,
        }
    }

    fn claiming(
        digit: Digit,
        box_index: u8,
        row_or_col: House,
        intersection_cells: DigitPositions,
        eliminations: DigitPositions,
    ) -> Self {
        Self {
            kind: LockedCandidatesKind::Claiming,
            digit,
            box_index,
            row_or_col,
            intersection_cells,
            eliminations,
        }
    }
}

impl TechniqueStep for LockedCandidatesStep {
    fn technique_name(&self) -> &'static str {
        match self.kind {
            LockedCandidatesKind::Pointing => NAME_POINTING,
            LockedCandidatesKind::Claiming => NAME_CLAIMING,
        }
    }

    fn clone_box(&self) -> BoxedTechniqueStep {
        Box::new(self.clone())
    }

    fn condition_cells(&self) -> ConditionCells {
        House::Box {
            index: self.box_index,
        }
        .positions()
            | self.row_or_col.positions()
    }

    fn condition_digit_cells(&self) -> ConditionDigitCells {
        vec![(self.intersection_cells, DigitSet::from_elem(self.digit))]
    }

    fn application(&self) -> Vec<TechniqueApplication> {
        vec![TechniqueApplication::CandidateElimination {
            positions: self.eliminations,
            digits: DigitSet::from_elem(self.digit),
        }]
    }
}

impl Technique for LockedCandidates {
    fn name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        for box_index in 0..9 {
            let box_ = House::Box { index: box_index };
            let origin = Position::box_origin(box_index);
            let row_or_cols = [
                House::Row { y: origin.y() },
                House::Row { y: origin.y() + 1 },
                House::Row { y: origin.y() + 2 },
                House::Column { x: origin.x() },
                House::Column { x: origin.x() + 1 },
                House::Column { x: origin.x() + 2 },
            ];
            for row_or_col in row_or_cols {
                let intersection = box_.positions() & row_or_col.positions();
                if (intersection & !grid.candidates.decided_cells()).is_empty() {
                    continue;
                }
                let rest_in_box = box_.positions() & !intersection;
                let rest_in_row_or_col = row_or_col.positions() & !intersection;
                for digit in Digit::ALL {
                    let digit_positions = grid.candidates.digit_positions(digit);
                    if (digit_positions & intersection).is_empty() {
                        continue;
                    }

                    if (digit_positions & rest_in_box).is_empty() {
                        // Pointing
                        let eliminations = digit_positions & rest_in_row_or_col;
                        if grid
                            .candidates
                            .would_remove_candidate_with_mask_change(eliminations, digit)
                        {
                            return Ok(Some(Box::new(LockedCandidatesStep::pointing(
                                digit,
                                box_index,
                                row_or_col,
                                digit_positions & intersection,
                                eliminations,
                            ))));
                        }
                    } else if (digit_positions & rest_in_row_or_col).is_empty() {
                        // Claiming
                        let eliminations = digit_positions & rest_in_box;
                        if grid
                            .candidates
                            .would_remove_candidate_with_mask_change(eliminations, digit)
                        {
                            return Ok(Some(Box::new(LockedCandidatesStep::claiming(
                                digit,
                                box_index,
                                row_or_col,
                                digit_positions & intersection,
                                eliminations,
                            ))));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
        let mut changed = false;

        for box_index in 0..9 {
            let box_ = House::Box { index: box_index };
            let origin = Position::box_origin(box_index);
            let row_or_cols = [
                House::Row { y: origin.y() },
                House::Row { y: origin.y() + 1 },
                House::Row { y: origin.y() + 2 },
                House::Column { x: origin.x() },
                House::Column { x: origin.x() + 1 },
                House::Column { x: origin.x() + 2 },
            ];
            for row_or_col in row_or_cols {
                let intersection = box_.positions() & row_or_col.positions();
                if (intersection & !grid.candidates.decided_cells()).is_empty() {
                    continue;
                }

                let rest_in_box = box_.positions() & !intersection;
                let rest_in_row_or_col = row_or_col.positions() & !intersection;
                for digit in Digit::ALL {
                    let digit_positions = grid.candidates.digit_positions(digit);
                    if (digit_positions & intersection).is_empty() {
                        continue;
                    }

                    if (digit_positions & rest_in_box).is_empty() {
                        // Pointing
                        let eliminations = digit_positions & rest_in_row_or_col;
                        changed |= grid
                            .candidates
                            .remove_candidate_with_mask(eliminations, digit);
                    } else if (digit_positions & rest_in_row_or_col).is_empty() {
                        // Claiming
                        let eliminations = digit_positions & rest_in_box;
                        changed |= grid
                            .candidates
                            .remove_candidate_with_mask(eliminations, digit);
                    }
                }
            }
        }

        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{CandidateGrid, Digit, Position};

    use super::*;
    use crate::testing::TechniqueTester;

    #[test]
    fn test_pointing_eliminates_from_row() {
        // Box 0 (rows 0-2, cols 0-2): limit D5 candidates to row 0 inside the box.
        let mut grid = CandidateGrid::new();
        for pos in Position::BOXES[0] {
            if pos.y() != 0 {
                grid.remove_candidate(pos, Digit::D5);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&LockedCandidates::new())
            // D5 removed from the rest of row 0 outside the box.
            .assert_removed_includes(Position::new(3, 0), [Digit::D5])
            .assert_removed_includes(Position::new(8, 0), [Digit::D5]);
    }

    #[test]
    fn test_claiming_eliminates_from_box() {
        // Row 0: limit D7 candidates to box 0 cells in row 0.
        let mut grid = CandidateGrid::new();
        for pos in Position::ROWS[0] {
            if pos.x() > 2 {
                grid.remove_candidate(pos, Digit::D7);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&LockedCandidates::new())
            // D7 removed from the rest of box 0 outside row 0.
            .assert_removed_includes(Position::new(0, 1), [Digit::D7])
            .assert_removed_includes(Position::new(2, 2), [Digit::D7]);
    }

    #[test]
    fn test_no_change_when_no_locked_candidates() {
        // A fresh grid has no locked candidate eliminations.
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&LockedCandidates::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }
}
