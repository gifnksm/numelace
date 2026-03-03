use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, Position};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
};

const ID: &str = "y_wing";
const NAME: &str = "Y-Wing";

/// A technique that removes candidates using a Y-Wing pattern.
///
/// A "Y-Wing" occurs when a pivot cell has two candidates (A/B),
/// and two wing cells each see the pivot with candidates (A/C) and (B/C),
/// respectively. The shared candidate C can be eliminated from any cell
/// that sees both wings.
#[derive(Debug, Default, Clone, Copy)]
pub struct YWing {}

struct Condition {
    pivot: Position,
    wing1: Position,
    wing2: Position,
    d1: Digit,
    d2: Digit,
    d3: Digit,
}

impl Condition {
    fn build_step(
        &self,
        before_grid: &TechniqueGrid,
        after_grid: &TechniqueGrid,
    ) -> BoxedTechniqueStep {
        let condition_positions = DigitPositions::from_iter([self.pivot, self.wing1, self.wing2]);
        let condition_digit_positions = vec![
            (
                DigitPositions::from_elem(self.pivot),
                DigitSet::from_iter([self.d1, self.d2]),
            ),
            (
                DigitPositions::from_elem(self.wing1),
                DigitSet::from_iter([self.d1, self.d3]),
            ),
            (
                DigitPositions::from_elem(self.wing2),
                DigitSet::from_iter([self.d2, self.d3]),
            ),
        ];
        TechniqueStepData::from_diff(
            NAME,
            condition_positions,
            condition_digit_positions,
            before_grid,
            after_grid,
        )
    }
}

impl YWing {
    /// Creates a new `YWing` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    fn apply_with_control_flow<T, F>(grid: &mut TechniqueGrid, mut on_condition: F) -> Option<T>
    where
        F: for<'a> FnMut(&'a mut TechniqueGrid, &'a Condition) -> ControlFlow<T>,
    {
        let bivalue_positions = grid.classify_positions::<3>()[2];
        for pivot in bivalue_positions {
            let pivot_peers = pivot.house_peers() & bivalue_positions;
            let pivot_digits = grid.candidates_at(pivot);
            let Some([d1, d2]) = pivot_digits.as_double() else {
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
                        && let ControlFlow::Break(value) = on_condition(
                            grid,
                            &Condition {
                                pivot,
                                wing1,
                                wing2,
                                d1,
                                d2,
                                d3,
                            },
                        )
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
    fn id(&self) -> &'static str {
        ID
    }

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
            .apply_pass(&YWing::new())
            .assert_removed_includes(elimination, [Digit::D3]);
    }

    #[test]
    fn test_no_change_when_no_y_wing() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_pass(&YWing::new())
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
            .apply_pass(&YWing::new())
            .assert_removed_includes(elimination, [Digit::D3])
            .assert_no_change(non_elimination);
    }
}
