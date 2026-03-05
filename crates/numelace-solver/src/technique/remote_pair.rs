use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, House, Position, PositionIndexedArray};
use tinyvec::{ArrayVec, array_vec};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
};

const ID: &str = "remote_pair";
const NAME: &str = "Remote Pair";

/// A technique that removes candidates using a Remote Pair pattern.
///
/// A "Remote Pair" forms a chain of bivalue cells containing the same two digits.
/// If the chain length is even, the endpoints have opposite parity, and any cell
/// that sees both endpoints cannot contain either digit.
#[derive(Debug, Default, Clone, Copy)]
pub struct RemotePair {}

struct Condition<'a> {
    digit1: Digit,
    digit2: Digit,
    stack: &'a [TraversalStackItem],
}

impl Condition<'_> {
    fn build_step(
        &self,
        before_grid: &TechniqueGrid,
        after_grid: &TechniqueGrid,
    ) -> BoxedTechniqueStep {
        let bivalue_positions = before_grid.classify_positions::<3>()[2];
        let digit_positions = bivalue_positions
            & before_grid.digit_positions(self.digit1)
            & before_grid.digit_positions(self.digit2);
        let mut condition_positions = DigitPositions::new();
        let mut condition_digit_position_mask = DigitPositions::new();
        for items in self.stack.windows(2) {
            let pos1 = items[0].position;
            let pos2 = items[1].position;
            condition_digit_position_mask.insert(pos1);
            condition_digit_position_mask.insert(pos2);
            if pos1.y() == pos2.y() && digit_positions.positions_in_row(pos1.y()).len() == 2 {
                condition_positions |= DigitPositions::ROW_POSITIONS[pos1.y()];
            } else if pos1.x() == pos2.x() && digit_positions.positions_in_col(pos1.x()).len() == 2
            {
                condition_positions |= DigitPositions::COLUMN_POSITIONS[pos1.x()];
            } else {
                debug_assert_eq!(pos1.box_index(), pos2.box_index());
                debug_assert_eq!(digit_positions.positions_in_box(pos1.box_index()).len(), 2);
                condition_positions |= DigitPositions::BOX_POSITIONS[pos1.box_index()];
            }
        }
        let condition_digit_positions = vec![(
            condition_digit_position_mask,
            DigitSet::from_iter([self.digit1, self.digit2]),
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

#[derive(Debug)]
struct TraversalGraph {
    link_peers: PositionIndexedArray<ArrayVec<[Position; 3]>>,
}

#[derive(Debug, Default)]
struct TraversalStackItem {
    position: Position,
    visited_positions: DigitPositions,
    remaining_positions: DigitPositions,
}

impl TraversalStackItem {
    fn new(
        position: Position,
        mut visited_positions: DigitPositions,
        graph: &TraversalGraph,
    ) -> Option<Self> {
        if !visited_positions.insert(position) {
            return None;
        }
        let remaining_positions =
            DigitPositions::from_iter(graph.link_peers[position]) & !visited_positions;
        let this = Self {
            position,
            visited_positions,
            remaining_positions,
        };
        Some(this)
    }

    fn next_item(&mut self, graph: &TraversalGraph) -> Option<Self> {
        while let Some(next) = self.remaining_positions.pop_first() {
            if let Some(next_item) = Self::new(next, self.visited_positions, graph) {
                return Some(next_item);
            }
        }
        None
    }
}

impl RemotePair {
    /// Creates a new `RemotePair` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    fn apply_with_control_flow<T, F>(grid: &mut TechniqueGrid, mut on_condition: F) -> Option<T>
    where
        F: for<'a> FnMut(&'a TechniqueGrid, &'a Condition<'a>) -> ControlFlow<T>,
    {
        let bivalue_positions = grid.classify_positions::<3>()[2];
        if bivalue_positions.len() < 4 {
            return None;
        }

        let mut digits = DigitSet::FULL.iter();
        while let Some(digit1) = digits.next() {
            let digit_positions1 = bivalue_positions & grid.digit_positions(digit1);
            if digit_positions1.len() < 4 {
                continue;
            }
            for digit2 in digits.as_set() {
                let digits12 = DigitSet::from_iter([digit1, digit2]);
                let digit_positions12 = digit_positions1 & grid.digit_positions(digit2);
                if digit_positions12.len() < 4 {
                    continue;
                }
                let mut link_positions = DigitPositions::new();
                let mut link_peers =
                    PositionIndexedArray::from_array([array_vec!([Position; 3]); 81]);
                for house in House::ALL {
                    let house_positions = digit_positions12 & house.positions();
                    let Some([pos1, pos2]) = house_positions.as_double() else {
                        continue;
                    };
                    link_positions.insert(pos1);
                    link_positions.insert(pos2);
                    link_peers[pos1].push(pos2);
                    link_peers[pos2].push(pos1);
                }
                let graph = TraversalGraph { link_peers };
                for chain_start in link_positions {
                    let mut stack = array_vec!([TraversalStackItem; 81]);
                    let Some(item) =
                        TraversalStackItem::new(chain_start, DigitPositions::new(), &graph)
                    else {
                        continue;
                    };
                    stack.push(item);
                    while let Some(item) = stack.last_mut() {
                        if let Some(next_item) = item.next_item(&graph) {
                            let chain_end = next_item.position;
                            stack.push(next_item);
                            if stack.len() % 2 == 0 {
                                let elimination =
                                    chain_start.house_peers() & chain_end.house_peers();
                                if grid.remove_candidate_set_with_mask(elimination, digits12) {
                                    let condition = &Condition {
                                        digit1,
                                        digit2,
                                        stack: &stack,
                                    };
                                    if let ControlFlow::Break(result) =
                                        on_condition(grid, condition)
                                    {
                                        return Some(result);
                                    }
                                }
                            }
                            continue;
                        }
                        stack.pop();
                    }
                }
            }
        }
        None
    }
}

impl Technique for RemotePair {
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
    fn test_eliminates_remote_pair_candidates() {
        let mut grid = CandidateGrid::new();
        let digit1 = Digit::D1;
        let digit2 = Digit::D2;

        let chain_start = Position::new(0, 0);
        let chain_mid1 = Position::new(4, 0);
        let chain_mid2 = Position::new(4, 5);
        let chain_end = Position::new(1, 5);

        for pos in [chain_start, chain_mid1, chain_mid2, chain_end] {
            for digit in Digit::ALL {
                if digit != digit1 && digit != digit2 {
                    grid.remove_candidate(pos, digit);
                }
            }
        }

        TechniqueTester::new(grid)
            .apply_pass(&RemotePair::new())
            .assert_removed_includes(Position::new(0, 5), [digit1, digit2])
            .assert_removed_includes(Position::new(1, 0), [digit1, digit2]);
    }

    #[test]
    fn test_no_change_when_no_remote_pair() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_pass(&RemotePair::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }
}
