use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, House, Position, PositionIndexedArray};
use tinyvec::{ArrayVec, array_vec};

use crate::{
    BoxedTechniqueStep, SolverError, Technique, TechniqueApplication, TechniqueGrid,
    TechniqueStepData, TechniqueTier,
};

const NAME: &str = "X-Chain";

/// A technique that removes candidates using an X-Chain pattern.
///
/// An "X-Chain" forms a chain of alternating strong and weak links for a digit.
/// If the endpoints of an odd-length chain share a common peer, that peer cannot
/// contain the digit and can be eliminated.
#[derive(Debug, Default, Clone, Copy)]
pub struct XChain {}

#[derive(Debug)]
struct TraversalGraph {
    strong_link_positions: DigitPositions,
    strong_link_peers: PositionIndexedArray<ArrayVec<[Position; 3]>>,
}

#[derive(Debug, Default)]
struct TraversalStackItem {
    visited_strong_link_starts: DigitPositions,
    visited_strong_link_ends: DigitPositions,
    strong_link_end: Position,
    remaining_strong_link_ends: DigitPositions,
    remaining_strong_link_starts: DigitPositions,
}

impl TraversalStackItem {
    fn new(
        strong_link_start: Position,
        mut visited_strong_link_starts: DigitPositions,
        visited_strong_link_ends: DigitPositions,
        graph: &TraversalGraph,
    ) -> Option<Self> {
        if !visited_strong_link_starts.insert(strong_link_start) {
            return None;
        }

        let remaining_strong_link_ends =
            DigitPositions::from_iter(graph.strong_link_peers[strong_link_start])
                & !visited_strong_link_ends;

        let mut this = Self {
            visited_strong_link_starts,
            visited_strong_link_ends,
            strong_link_end: Position::default(),
            remaining_strong_link_ends,
            remaining_strong_link_starts: DigitPositions::default(),
        };
        if !this.visit_next_strong_link_end(graph) {
            return None;
        }
        Some(this)
    }

    fn visit_next_strong_link_end(&mut self, graph: &TraversalGraph) -> bool {
        if let Some(strong_link_end) = self.remaining_strong_link_ends.pop_first() {
            self.strong_link_end = strong_link_end;
            self.remaining_strong_link_starts = strong_link_end.house_peers()
                & graph.strong_link_positions
                & !self.visited_strong_link_starts;
            return true;
        }
        false
    }

    fn next_item(&mut self, graph: &TraversalGraph) -> Option<Self> {
        while let Some(strong_link_start) = self.remaining_strong_link_starts.pop_first() {
            let mut visited_strong_link_ends = self.visited_strong_link_ends;
            visited_strong_link_ends.insert(self.strong_link_end);
            if let Some(next_item) = Self::new(
                strong_link_start,
                self.visited_strong_link_starts,
                visited_strong_link_ends,
                graph,
            ) {
                return Some(next_item);
            }
        }
        None
    }
}

impl XChain {
    /// Creates a new `XChain` technique.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    fn apply_with_control_flow<F>(
        grid: &mut TechniqueGrid,
        mut on_condition: F,
    ) -> Option<BoxedTechniqueStep>
    where
        F: for<'a> FnMut(
            &'a TechniqueGrid,
            Digit,
            Position,
            &[TraversalStackItem],
            bool,
        ) -> ControlFlow<BoxedTechniqueStep>,
    {
        for digit in Digit::ALL {
            let digit_positions = grid.digit_positions(digit);
            let mut strong_link_positions = DigitPositions::new();
            let mut strong_link_peers =
                PositionIndexedArray::from_array([array_vec!([Position; 3]); 81]);
            for house in House::ALL {
                let house_positions = digit_positions & house.positions();
                let Some((pos1, pos2)) = house_positions.as_double() else {
                    continue;
                };
                strong_link_positions.insert(pos1);
                strong_link_positions.insert(pos2);
                strong_link_peers[pos1].push(pos2);
                strong_link_peers[pos2].push(pos1);
            }

            let graph = TraversalGraph {
                strong_link_positions,
                strong_link_peers,
            };

            for chain_start in strong_link_positions {
                let mut stack = array_vec!([TraversalStackItem; 81]);
                let Some(item) = TraversalStackItem::new(
                    chain_start,
                    DigitPositions::new(),
                    DigitPositions::new(),
                    &graph,
                ) else {
                    continue;
                };

                let strong_link_end = item.strong_link_end;
                stack.push(item);
                let (applied, is_placement) = if chain_start == strong_link_end {
                    (grid.place(chain_start, digit), true)
                } else {
                    let elimination = chain_start.house_peers() & strong_link_end.house_peers();
                    (grid.remove_candidate_with_mask(elimination, digit), false)
                };
                if applied
                    && let ControlFlow::Break(step) =
                        on_condition(grid, digit, chain_start, &stack, is_placement)
                {
                    return Some(step);
                }
                while let Some(item) = stack.last_mut() {
                    if let Some(next_item) = item.next_item(&graph) {
                        let strong_link_end = next_item.strong_link_end;
                        stack.push(next_item);
                        let (applied, is_placement) = if chain_start == strong_link_end {
                            (grid.place(chain_start, digit), true)
                        } else {
                            let elimination =
                                chain_start.house_peers() & strong_link_end.house_peers();
                            (grid.remove_candidate_with_mask(elimination, digit), false)
                        };
                        if applied
                            && let ControlFlow::Break(step) =
                                on_condition(grid, digit, chain_start, &stack, is_placement)
                        {
                            return Some(step);
                        }
                        continue;
                    }
                    if item.visit_next_strong_link_end(&graph) {
                        continue;
                    }
                    stack.pop();
                }
            }
        }
        None
    }
}

impl Technique for XChain {
    fn name(&self) -> &'static str {
        NAME
    }

    fn tier(&self) -> TechniqueTier {
        TechniqueTier::UpperIntermediate
    }

    fn clone_box(&self) -> crate::BoxedTechnique {
        Box::new(*self)
    }

    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        let mut after_grid = grid.clone();
        let step = Self::apply_with_control_flow(
            &mut after_grid,
            |after_grid, digit, chain_start, stack, is_placement| {
                let mut positions = DigitPositions::new();
                positions.insert(chain_start);
                for item in stack {
                    positions.insert(item.strong_link_end);
                }
                let extra = if is_placement {
                    vec![TechniqueApplication::Placement {
                        position: chain_start,
                        digit,
                    }]
                } else {
                    vec![]
                };
                ControlFlow::Break(Box::new(TechniqueStepData::from_diff_with_extra(
                    NAME,
                    positions,
                    vec![(positions, DigitSet::from_elem(digit))],
                    grid,
                    after_grid,
                    extra,
                )))
            },
        );
        Ok(step)
    }

    fn apply(&self, grid: &mut TechniqueGrid) -> Result<usize, SolverError> {
        let mut changed = 0;
        Self::apply_with_control_flow(grid, |_, _, _, _, _| {
            changed += 1;
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
    fn test_eliminates_x_chain_candidates() {
        let mut grid = CandidateGrid::new();
        let digit = Digit::D1;
        let chain_start = Position::new(0, 0);
        let strong_link_partner = Position::new(4, 0);

        for pos in Position::ROWS[0] {
            if pos != chain_start && pos != strong_link_partner {
                grid.remove_candidate(pos, digit);
            }
        }

        let weak_link_node = Position::new(3, 1);
        let chain_end = Position::new(3, 7);
        for pos in Position::COLUMNS[3] {
            if pos != weak_link_node && pos != chain_end {
                grid.remove_candidate(pos, digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&XChain::new())
            .assert_removed_includes(Position::new(0, 7), [digit]);
    }

    #[test]
    fn test_places_x_chain_when_endpoints_match() {
        let mut grid = CandidateGrid::new();
        let digit = Digit::D1;
        let chain_start = Position::new(4, 4);
        let strong_row_partner = Position::new(3, 4);
        let strong_col_partner = Position::new(4, 3);

        for pos in Position::ROWS[4] {
            if pos != chain_start && pos != strong_row_partner {
                grid.remove_candidate(pos, digit);
            }
        }
        for pos in Position::COLUMNS[4] {
            if pos != chain_start && pos != strong_col_partner {
                grid.remove_candidate(pos, digit);
            }
        }

        TechniqueTester::new(grid)
            .apply_once(&XChain::new())
            .assert_placed(chain_start, digit);
    }

    #[test]
    fn test_no_change_when_no_x_chain() {
        let grid = CandidateGrid::new();

        TechniqueTester::new(grid)
            .apply_once(&XChain::new())
            .assert_no_change(Position::new(0, 0))
            .assert_no_change(Position::new(4, 4));
    }
}
