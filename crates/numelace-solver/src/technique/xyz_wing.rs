use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, Position};

use crate::{
    BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData, TechniqueTier,
};

const ID: &str = "xyz_wing";
const NAME: &str = "XYZ-Wing";

/// A technique that removes candidates using an XYZ-Wing pattern.
///
/// An "XYZ-Wing" occurs when a pivot cell has three candidates (A/B/C),
/// and two wing cells each see the pivot with candidates (A/B) and (A/C),
/// respectively. The shared candidate A can be eliminated from any cell
/// that sees the pivot and both wings.
#[derive(Debug, Default, Clone, Copy)]
pub struct XyzWing {}

struct Condition {
    pivot: Position,
    wing1: Position,
    wing2: Position,
    common_digit: Digit,
    wing1_digit: Digit,
    wing2_digit: Digit,
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
                DigitSet::from_iter([self.common_digit, self.wing1_digit, self.wing2_digit]),
            ),
            (
                DigitPositions::from_elem(self.wing1),
                DigitSet::from_iter([self.common_digit, self.wing1_digit]),
            ),
            (
                DigitPositions::from_elem(self.wing2),
                DigitSet::from_iter([self.common_digit, self.wing2_digit]),
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

impl XyzWing {
    /// Creates a new `XyzWing` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    fn apply_with_control_flow<T, F>(grid: &mut TechniqueGrid, mut on_condition: F) -> Option<T>
    where
        F: for<'a> FnMut(&'a mut TechniqueGrid, &'a Condition) -> ControlFlow<T>,
    {
        let [_, _, bivalue_positions, trivalue_positions] = grid.classify_positions();
        for pivot in trivalue_positions {
            let pivot_digits = grid.candidates_at(pivot);
            let Some([pd1, pd2, pd3]) = pivot_digits.as_triple() else {
                // `grid.remove_candidate_with_mask` may have changed the candidates at pivot, so we need to check again
                continue;
            };
            let pd1_mask = grid.digit_positions(pd1);
            let pd2_mask = grid.digit_positions(pd2);
            let pd3_mask = grid.digit_positions(pd3);
            let wing_candidates = pivot.house_peers()
                & bivalue_positions
                & ((pd1_mask & pd2_mask) | (pd2_mask & pd3_mask) | (pd3_mask & pd1_mask));
            for wing1 in wing_candidates {
                let wing1_digits = grid.candidates_at(wing1);
                if wing1_digits.len() != 2 {
                    // `grid.remove_candidate_with_mask` may have changed the candidates at wing1, so we need to check again
                    continue;
                }
                let Some(wing2_digit) = pivot_digits.difference(wing1_digits).as_single() else {
                    // `grid.remove_candidate_with_mask` may have changed the candidates at wing1, so we need to check again
                    continue;
                };
                for wing2 in wing_candidates & grid.digit_positions(wing2_digit) {
                    let wing2_digits = grid.candidates_at(wing2);
                    if wing2_digits.len() != 2 {
                        // `grid.remove_candidate_with_mask` may have changed the candidates at wing2, so we need to check again
                        continue;
                    }
                    let common_digit = (pivot_digits & wing1_digits & wing2_digits)
                        .as_single()
                        .unwrap();
                    let wing1_digit =
                        (pivot_digits & wing1_digits & !DigitSet::from_elem(common_digit))
                            .as_single()
                            .unwrap();
                    let elimination_mask =
                        pivot.house_peers() & wing1.house_peers() & wing2.house_peers();
                    if grid.remove_candidate_with_mask(elimination_mask, common_digit)
                        && let ControlFlow::Break(value) = on_condition(
                            grid,
                            &Condition {
                                pivot,
                                wing1,
                                wing2,
                                common_digit,
                                wing1_digit,
                                wing2_digit,
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

impl Technique for XyzWing {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn tier(&self) -> TechniqueTier {
        TechniqueTier::Advanced
    }

    fn clone_box(&self) -> Box<dyn Technique> {
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
    use crate::testing;

    const TECHNIQUE: XyzWing = XyzWing::new();

    #[test]
    fn test_eliminates_xyz_wing_candidates() {
        let mut grid = CandidateGrid::new();
        let pivot = Position::from_xy(1, 1);
        let wing1 = Position::from_xy(1, 2);
        let wing2 = Position::from_xy(2, 1);
        let elimination = Position::from_xy(2, 2);

        // Pivot: {1,2,3}
        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 {
                grid.remove_candidate(pivot, digit);
            }
        }

        // Wing1: {1,2}
        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 {
                grid.remove_candidate(wing1, digit);
            }
        }

        // Wing2: {1,3}
        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D3 {
                grid.remove_candidate(wing2, digit);
            }
        }

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t.assert_removed_includes(elimination, [Digit::D1]);
        });
    }

    #[test]
    fn test_no_change_when_no_xyz_wing() {
        let grid = CandidateGrid::new();
        testing::test_technique_apply_pass_no_changes(grid, &TECHNIQUE);
    }

    #[test]
    fn test_only_common_peers_are_eliminated() {
        let mut grid = CandidateGrid::new();
        let pivot = Position::from_xy(1, 1);
        let wing1 = Position::from_xy(1, 2);
        let wing2 = Position::from_xy(2, 1);
        let elimination = Position::from_xy(2, 2);
        let non_elimination = Position::from_xy(1, 5);

        // Pivot: {1,2,3}
        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 {
                grid.remove_candidate(pivot, digit);
            }
        }

        // Wing1: {1,2}
        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D2 {
                grid.remove_candidate(wing1, digit);
            }
        }

        // Wing2: {1,3}
        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D3 {
                grid.remove_candidate(wing2, digit);
            }
        }

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t.assert_removed_includes(elimination, [Digit::D1])
                .assert_no_change(non_elimination);
        });
    }
}
