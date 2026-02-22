use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, Position};

use crate::{BoxedTechniqueStep, Technique, TechniqueGrid, TechniqueStepData};

const NAME: &str = "Y-Wing";

/// A technique that removes candidates using a Y-Wing pattern.
///
/// A "Y-Wing" occurs when a pivot cell has two candidates (A/B),
/// and two wing cells each see the pivot with candidates (A/C) and (B/C),
/// respectively. The shared candidate C can be eliminated from any cell
/// that sees both wings.
#[derive(Debug, Default, Clone, Copy)]
pub struct YWing {}

impl YWing {
    /// Creates a new `YWing` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl YWing {
    #[inline]
    fn apply_with_control_flow<F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Option<BoxedTechniqueStep>
    where
        F: for<'a> FnMut(
            &'a mut TechniqueGrid,
            (Position, Position, Position),
            (Digit, Digit, Digit),
        ) -> ControlFlow<BoxedTechniqueStep>,
    {
        let pair_candidate_cells = grid.classify_cells::<3>()[2];
        for pivot in pair_candidate_cells {
            let pivot_peers = pivot.house_peers() & pair_candidate_cells;
            let pivot_digits = grid.candidates_at(pivot);
            let Some((d1, d2)) = pivot_digits.as_double() else {
                // `grid.remove_candidate_with_mask` may have changed the candidates at pivot, so we need to check again
                continue;
            };
            for wing1 in pivot_peers & grid.digit_positions(d1) {
                let wing1_digits = grid.candidates_at(wing1);
                let Some(d3) = (wing1_digits & !pivot_digits).as_single() else {
                    continue;
                };
                for wing2 in pivot_peers & grid.digit_positions(d2) & grid.digit_positions(d3) {
                    let elimination_cells =
                        (wing1.house_peers() & wing2.house_peers()) & grid.digit_positions(d3);
                    if grid.remove_candidate_with_mask(elimination_cells, d3)
                        && let ControlFlow::Break(value) =
                            on_condition(grid, (pivot, wing1, wing2), (d1, d2, d3))
                    {
                        return Some(value);
                    }
                }
            }
        }
        None
    }
}

impl Technique for YWing {
    fn name(&self) -> &'static str {
        NAME
    }

    fn clone_box(&self) -> crate::BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(
        &self,
        grid: &TechniqueGrid,
    ) -> Result<Option<BoxedTechniqueStep>, crate::SolverError> {
        let mut after_grid = grid.clone();
        let step = Self::apply_with_control_flow(
            &mut after_grid,
            |after_grid, (pivot, wing1, wing2), (d1, d2, d3)| {
                ControlFlow::Break(Box::new(TechniqueStepData::from_diff(
                    NAME,
                    DigitPositions::from_iter([pivot, wing1, wing2]),
                    vec![
                        (
                            DigitPositions::from_elem(pivot),
                            DigitSet::from_iter([d1, d2]),
                        ),
                        (
                            DigitPositions::from_elem(wing1),
                            DigitSet::from_iter([d1, d3]),
                        ),
                        (
                            DigitPositions::from_elem(wing2),
                            DigitSet::from_iter([d2, d3]),
                        ),
                    ],
                    grid,
                    after_grid,
                )))
            },
        );
        Ok(step)
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, crate::SolverError> {
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
    fn test_eliminates_y_wing_candidates() {
        let mut grid = CandidateGrid::new();
        let pivot = Position::new(1, 1);
        let wing1 = Position::new(1, 5);
        let wing2 = Position::new(5, 1);
        let elimination = Position::new(5, 5);

        // Pivot: {1,2}
        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 {
                grid.remove_candidate(pivot, digit);
            }
        }

        // Wing1: {1,3}
        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D3 {
                grid.remove_candidate(wing1, digit);
            }
        }

        // Wing2: {2,3}
        for digit in Digit::ALL {
            if digit != Digit::D2 && digit != Digit::D3 {
                grid.remove_candidate(wing2, digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&YWing::new())
            .assert_removed_includes(elimination, [Digit::D3]);
    }

    #[test]
    fn test_no_change_when_no_y_wing() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&YWing::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }

    #[test]
    fn test_only_common_peers_are_eliminated() {
        let mut grid = CandidateGrid::new();
        let pivot = Position::new(1, 1);
        let wing1 = Position::new(1, 5);
        let wing2 = Position::new(5, 1);
        let elimination = Position::new(5, 5);
        let non_elimination = Position::new(7, 1);

        // Pivot: {1,2}
        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 {
                grid.remove_candidate(pivot, digit);
            }
        }

        // Wing1: {1,3}
        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D3 {
                grid.remove_candidate(wing1, digit);
            }
        }

        // Wing2: {2,3}
        for digit in Digit::ALL {
            if digit != Digit::D2 && digit != Digit::D3 {
                grid.remove_candidate(wing2, digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&YWing::new())
            .assert_removed_includes(elimination, [Digit::D3])
            .assert_no_change(non_elimination);
    }
}
