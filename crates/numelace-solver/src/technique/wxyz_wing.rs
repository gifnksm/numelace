use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, Position};

use crate::{
    BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData, TechniqueTier,
};

const ID: &str = "wxyz_wing";
const NAME: &str = "WXYZ-Wing";

/// A technique that removes candidates using a WXYZ-Wing pattern.
///
/// A WXYZ-Wing occurs when four cells contain exactly four candidates (W/X/Y/Z),
/// the digit Z appears in at least two of those cells, and each of W/X/Y is
/// confined to a single house within the four cells. The digit Z can be
/// eliminated from any cell that sees all Z-bearing cells.
#[derive(Debug, Default, Clone, Copy)]
pub struct WxyzWing {}

struct Condition {
    z_positions: DigitPositions,
    other_positions: DigitPositions,
    wxy: DigitSet,
    z: Digit,
}

impl Condition {
    fn build_step(
        &self,
        before_grid: &TechniqueGrid,
        after_grid: &TechniqueGrid,
    ) -> BoxedTechniqueStep {
        let condition_positions = self.z_positions | self.other_positions;
        let mut condition_digit_positions = vec![];
        let digits = self.wxy | DigitSet::from_elem(self.z);
        for pos in condition_positions {
            let pos_digits = before_grid.candidates_at(pos) & digits;
            condition_digit_positions.push((DigitPositions::from_elem(pos), pos_digits));
        }
        TechniqueStepData::from_diff(
            NAME,
            condition_positions,
            condition_digit_positions,
            before_grid,
            after_grid,
        )
    }
}

fn is_in_single_house(digit_positions: DigitPositions) -> bool {
    digit_positions.rows_set().len() <= 1
        || digit_positions.cols_set().len() <= 1
        || digit_positions.boxes_set().len() <= 1
}

impl WxyzWing {
    /// Creates a new `WxyzWing` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    fn apply_with_control_flow<T, F>(grid: &mut TechniqueGrid, mut on_condition: F) -> Option<T>
    where
        F: for<'a> FnMut(&'a mut TechniqueGrid, &'a Condition) -> ControlFlow<T>,
    {
        let [
            _,
            _,
            bivalue_positions,
            trivalue_positions,
            quadvalue_positions,
        ] = grid.classify_positions();
        let candidate_positions = bivalue_positions | trivalue_positions | quadvalue_positions;
        for z in Digit::ALL {
            let z_positions = grid.digit_positions(z) & candidate_positions;
            if z_positions.len() < 2 {
                continue;
            }
            let mut z_positions = z_positions.iter();
            while let Some(pos1) = z_positions.next() {
                let positions1 = DigitPositions::from_elem(pos1);
                let digits1 = grid.candidates_at(pos1);
                for pos2 in z_positions.as_set() {
                    let positions12 = positions1 | DigitPositions::from_elem(pos2);
                    let digits12 = digits1 | grid.candidates_at(pos2);
                    if digits12.len() > 4 {
                        continue;
                    }
                    let wxy = digits12 & !DigitSet::from_elem(z);
                    if wxy
                        .iter()
                        .any(|d| !is_in_single_house(positions12 & grid.digit_positions(d)))
                    {
                        continue;
                    }
                    let mut other_positions = (pos1.house_peers() | pos2.house_peers())
                        & candidate_positions
                        & !DigitPositions::from_iter([pos1, pos2]);
                    if other_positions.len() < 2 {
                        continue;
                    }
                    if digits12.len() == 4 {
                        other_positions &= digits12.iter().map(|d| grid.digit_positions(d)).sum();
                    }
                    let mut other_positions = other_positions.iter();
                    while let Some(pos3) = other_positions.next() {
                        let positions123 = positions12 | DigitPositions::from_elem(pos3);
                        let digits123 = digits12 | grid.candidates_at(pos3);
                        if digits123.len() > 4 {
                            continue;
                        }
                        let wxy = digits123 & !DigitSet::from_elem(z);
                        if wxy
                            .iter()
                            .any(|d| !is_in_single_house(positions123 & grid.digit_positions(d)))
                        {
                            continue;
                        }
                        for pos4 in other_positions.as_set() {
                            let positions1234 = positions123 | DigitPositions::from_elem(pos4);
                            let digits1234 = digits123 | grid.candidates_at(pos4);
                            if digits1234.len() != 4 {
                                continue;
                            }
                            let wxy = digits1234 & !DigitSet::from_elem(z);
                            if wxy.iter().any(|d| {
                                !is_in_single_house(positions1234 & grid.digit_positions(d))
                            }) {
                                continue;
                            }
                            let z_positions = grid.digit_positions(z) & positions1234;
                            let other_positions = positions1234 & !z_positions;
                            if other_positions.is_empty() {
                                continue;
                            }
                            let elimination = z_positions
                                .iter()
                                .map(Position::house_peers)
                                .product::<DigitPositions>();
                            if grid.remove_candidate_with_mask(elimination, z)
                                && let ControlFlow::Break(step) = on_condition(
                                    grid,
                                    &Condition {
                                        z_positions,
                                        other_positions,
                                        wxy,
                                        z,
                                    },
                                )
                            {
                                return Some(step);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

impl Technique for WxyzWing {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn tier(&self) -> TechniqueTier {
        TechniqueTier::Expert
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

    const TECHNIQUE: WxyzWing = WxyzWing::new();

    #[test]
    fn test_eliminates_wxyz_wing_candidates() {
        let mut grid = CandidateGrid::new();
        let pos1 = Position::from_xy(0, 0);
        let pos2 = Position::from_xy(1, 0);
        let pos3 = Position::from_xy(0, 1);
        let pos4 = Position::from_xy(1, 1);
        let elimination = Position::from_xy(2, 0);
        let non_elimination = Position::from_xy(0, 3);

        for digit in Digit::ALL {
            if digit != Digit::D1 && digit != Digit::D4 {
                grid.remove_candidate(pos1, digit);
            }
            if digit != Digit::D2 && digit != Digit::D4 {
                grid.remove_candidate(pos2, digit);
            }
            if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 {
                grid.remove_candidate(pos3, digit);
                grid.remove_candidate(pos4, digit);
            }
        }

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t.assert_removed_includes(elimination, [Digit::D4])
                .assert_no_change(non_elimination);
        });
    }

    #[test]
    fn test_no_change_when_no_wxyz_wing() {
        let grid = CandidateGrid::new();
        testing::test_technique_apply_pass_no_changes(grid, &TECHNIQUE);
    }
}
