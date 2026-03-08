use std::ops::ControlFlow;

use numelace_core::{
    Digit, DigitIndexedArray, DigitPositions, DigitSet, Position, PositionIndexedArray,
};
use tinyvec::{ArrayVec, array_vec};

use crate::{
    BoxedTechnique, BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData,
    TechniqueTier,
};

const ID: &str = "xy_chain";
const NAME: &str = "XY-Chain";

/// A technique that removes candidates using an XY-Chain pattern.
///
/// An "XY-Chain" is a chain of bivalue cells where adjacent cells share
/// exactly one candidate, forming alternating strong/weak links.
/// If the chain starts and ends with the same digit, that digit can be
/// removed from any cell that sees both endpoints.
///
/// This implementation also treats closed loops (endpoints are peers) as
/// valid XY-Chain patterns and applies additional eliminations along each
/// link of the loop.
#[derive(Debug, Default, Clone, Copy)]
pub struct XyChain {}

struct Condition<'a> {
    stack: &'a [TraversalStackItem],
}

impl Condition<'_> {
    fn build_step(
        &self,
        before_grid: &TechniqueGrid,
        after_grid: &TechniqueGrid,
    ) -> BoxedTechniqueStep {
        let mut condition_positions = DigitPositions::new();
        let mut condition_digit_positions = vec![];
        for item in self.stack {
            condition_positions.insert(item.position);
            condition_digit_positions.push((
                DigitPositions::from_elem(item.position),
                DigitSet::from_iter([item.incoming_digit, item.outgoing_digit]),
            ));
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

#[derive(Debug)]
struct TraversalGraph {
    link_map: DigitIndexedArray<PositionIndexedArray<(Digit, ArrayVec<[Position; 20]>)>>,
}

#[derive(Debug, Default)]
struct TraversalStackItem {
    position: Position,
    incoming_digit: Digit,
    outgoing_digit: Digit,
    visited_positions: DigitPositions,
    remaining_peer_positions: DigitPositions,
}

impl TraversalStackItem {
    fn new(
        position: Position,
        incoming_digit: Digit,
        mut visited_positions: DigitPositions,
        graph: &TraversalGraph,
    ) -> Option<Self> {
        if !visited_positions.insert(position) {
            return None;
        }
        let (outgoing_digit, peer_positions) = graph.link_map[incoming_digit][position];
        let remaining_peer_positions =
            DigitPositions::from_iter(peer_positions) & !visited_positions;
        let this = Self {
            position,
            incoming_digit,
            outgoing_digit,
            visited_positions,
            remaining_peer_positions,
        };
        Some(this)
    }

    fn next_item(&mut self, graph: &TraversalGraph) -> Option<Self> {
        while let Some(outgoing_pos) = self.remaining_peer_positions.pop_first() {
            if let Some(next_item) = Self::new(
                outgoing_pos,
                self.outgoing_digit,
                self.visited_positions,
                graph,
            ) {
                return Some(next_item);
            }
        }
        None
    }
}

impl XyChain {
    /// Creates a new `XyChain` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    fn apply_with_control_flow<T, F>(grid: &mut TechniqueGrid, mut on_condition: F) -> Option<T>
    where
        F: for<'a> FnMut(&'a TechniqueGrid, &'a Condition<'a>) -> ControlFlow<T>,
    {
        let bivalue_positions = grid.classify_positions::<3>()[2];
        if bivalue_positions.len() < 2 {
            return None;
        }
        #[expect(clippy::large_stack_arrays)]
        let mut graph = TraversalGraph {
            link_map: DigitIndexedArray::from_array(
                [PositionIndexedArray::from_array([(Digit::D1, array_vec!([Position; 20])); 81]);
                    9],
            ),
        };
        for position in bivalue_positions {
            let [d1, d2] = grid.candidates_at(position).as_double().unwrap();
            graph.link_map[d1][position].0 = d2;
            let bivalue_positions_in_house_peers = position.house_peers() & bivalue_positions;
            for peer_pos in bivalue_positions_in_house_peers
                & grid.digit_positions(d2)
                & !grid.digit_positions(d1)
            {
                graph.link_map[d1][position].1.push(peer_pos);
            }
            graph.link_map[d2][position].0 = d1;
            for peer_pos in bivalue_positions_in_house_peers
                & grid.digit_positions(d1)
                & !grid.digit_positions(d2)
            {
                graph.link_map[d2][position].1.push(peer_pos);
            }
        }
        for start_digit in Digit::ALL {
            for start_pos in bivalue_positions & grid.digit_positions(start_digit) {
                let mut stack = array_vec!([TraversalStackItem; 81]);
                let Some(item) = TraversalStackItem::new(
                    start_pos,
                    start_digit,
                    DigitPositions::default(),
                    &graph,
                ) else {
                    continue;
                };
                stack.push(item);
                while let Some(item) = stack.last_mut() {
                    if let Some(next_item) = item.next_item(&graph) {
                        let end_pos = next_item.position;
                        let end_digit = next_item.outgoing_digit;
                        stack.push(next_item);
                        if end_digit == start_digit {
                            let mut changed = false;
                            let elimination = start_pos.house_peers() & end_pos.house_peers();
                            changed |= grid.remove_candidate_with_mask(elimination, start_digit);
                            if start_pos.house_peers().contains(end_pos) {
                                for [item1, item2] in stack.array_windows() {
                                    let digit = item1.outgoing_digit;
                                    let elimination =
                                        item1.position.house_peers() & item2.position.house_peers();
                                    changed |= grid.remove_candidate_with_mask(elimination, digit);
                                }
                            }
                            if changed {
                                let condition = Condition { stack: &stack };
                                if let ControlFlow::Break(step) = on_condition(grid, &condition) {
                                    return Some(step);
                                }
                            }
                        }
                        continue;
                    }
                    stack.pop();
                }
            }
        }
        None
    }
}

impl Technique for XyChain {
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
    use crate::testing;

    const TECHNIQUE: XyChain = XyChain::new();

    fn set_bivalue(grid: &mut CandidateGrid, position: Position, d1: Digit, d2: Digit) {
        for digit in Digit::ALL {
            if digit != d1 && digit != d2 {
                grid.remove_candidate(position, digit);
            }
        }
    }

    #[test]
    fn test_eliminates_xy_chain_candidates() {
        let mut grid = CandidateGrid::new();
        let start = Position::new(1, 1);
        let mid = Position::new(5, 1);
        let end = Position::new(5, 5);
        let elimination = Position::new(1, 5);

        set_bivalue(&mut grid, start, Digit::D1, Digit::D2);
        set_bivalue(&mut grid, mid, Digit::D2, Digit::D3);
        set_bivalue(&mut grid, end, Digit::D1, Digit::D3);

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t.assert_removed_includes(elimination, [Digit::D1]);
        });
    }

    #[test]
    fn test_eliminates_xy_chain_loop_candidates() {
        let mut grid = CandidateGrid::new();
        let start = Position::new(1, 1);
        let mid = Position::new(5, 1);
        let end = Position::new(8, 1);

        let elimination_start_end = Position::new(3, 1);
        let elimination_start_mid = Position::new(2, 1);
        let elimination_mid_end = Position::new(7, 1);

        set_bivalue(&mut grid, start, Digit::D1, Digit::D2);
        set_bivalue(&mut grid, mid, Digit::D2, Digit::D3);
        set_bivalue(&mut grid, end, Digit::D1, Digit::D3);

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t.assert_removed_includes(elimination_start_end, [Digit::D1])
                .assert_removed_includes(elimination_start_mid, [Digit::D2])
                .assert_removed_includes(elimination_mid_end, [Digit::D3]);
        });
    }

    #[test]
    fn test_no_change_when_no_xy_chain() {
        let grid = CandidateGrid::new();
        testing::test_technique_apply_pass_no_changes(grid, &TECHNIQUE);
    }
}
