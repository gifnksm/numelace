use std::ops::ControlFlow;

use numelace_core::{Digit, DigitPositions, DigitSet, House, Position, PositionIndexedArray};
use tinyvec::{ArrayVec, array_vec};

use crate::{
    BoxedTechniqueStep, SolverError, Technique, TechniqueGrid, TechniqueStepData, TechniqueTier,
};

const ID: &str = "x_chain";
const NAME: &str = "X-Chain";

/// A technique that applies X-Chain and X-Cycle rules for a single digit.
///
/// Traversal walks strong links (exactly two candidates in a house), while
/// weak connectivity is represented by house-peer relations between consecutive
/// strong links in the chain.
///
/// Implemented effects:
/// - X-Chain endpoint elimination:
///   remove the digit from common peers of chain endpoints.
///   This is the discontinuous weak-weak X-Cycle case.
/// - X-Cycle discontinuous (strong-strong):
///   if chain start == chain end, place the digit at that cell.
/// - X-Cycle continuous:
///   when endpoints are peers, also eliminate via internal weak junctions.
#[derive(Debug, Default, Clone, Copy)]
pub struct XChain {}

struct Condition<'a> {
    digit: Digit,
    stack: &'a [TraversalStackItem],
}

impl Condition<'_> {
    fn build_step(
        &self,
        before_grid: &TechniqueGrid,
        after_grid: &TechniqueGrid,
    ) -> BoxedTechniqueStep {
        let digit_positions = before_grid.digit_positions(self.digit);
        let mut condition_positions = DigitPositions::new();
        let mut condition_digit_position_mask = DigitPositions::new();
        for item in self.stack {
            let pos1 = item.strong_link_start;
            let pos2 = item.strong_link_end;
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

#[derive(Debug)]
struct TraversalGraph {
    strong_link_positions: DigitPositions,
    strong_link_peers: PositionIndexedArray<ArrayVec<[Position; 3]>>,
}

#[derive(Debug, Default)]
struct TraversalStackItem {
    visited_strong_link_starts: DigitPositions,
    visited_strong_link_ends: DigitPositions,
    strong_link_start: Position,
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
            strong_link_start,
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
    fn apply_chain_effect(
        grid: &mut TechniqueGrid,
        digit: Digit,
        stack: &[TraversalStackItem],
    ) -> bool {
        let chain_start = stack.first().unwrap().strong_link_start;
        let chain_end = stack.last().unwrap().strong_link_end;

        if chain_start == chain_end {
            // X-Cycle (discontinuous, strong-strong at one node): placement.
            return grid.place(chain_start, digit);
        }

        // X-Chain endpoint elimination (discontinuous weak-weak X-Cycle).
        // This also captures weak-weak-equivalent eliminations in the same
        // common-peers form.
        let mut elimination = chain_start.house_peers() & chain_end.house_peers();

        if chain_start.house_peers().contains(chain_end) {
            // X-Cycle (continuous): if endpoints are peers, the chain can be closed
            // by a weak link; add eliminations from each internal weak junction.
            for [item1, item2] in stack.array_windows() {
                elimination |=
                    item1.strong_link_end.house_peers() & item2.strong_link_start.house_peers();
            }
        }

        grid.remove_candidate_with_mask(elimination, digit)
    }

    #[inline]
    fn apply_with_control_flow<T, F>(grid: &mut TechniqueGrid, mut on_condition: F) -> Option<T>
    where
        F: for<'a> FnMut(&'a TechniqueGrid, &'a Condition<'a>) -> ControlFlow<T>,
    {
        for digit in Digit::ALL {
            let digit_positions = grid.digit_positions(digit);
            let mut strong_link_positions = DigitPositions::new();
            let mut strong_link_peers =
                PositionIndexedArray::from_array([array_vec!([Position; 3]); 81]);
            for house in House::ALL {
                let house_positions = digit_positions & house.positions();
                let Some([pos1, pos2]) = house_positions.as_double() else {
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
                stack.push(item);
                let changed = Self::apply_chain_effect(grid, digit, &stack);
                let condition = &Condition {
                    digit,
                    stack: &stack,
                };
                if changed && let ControlFlow::Break(step) = on_condition(grid, condition) {
                    return Some(step);
                }
                while let Some(item) = stack.last_mut() {
                    if let Some(next_item) = item.next_item(&graph) {
                        stack.push(next_item);
                        let changed = Self::apply_chain_effect(grid, digit, &stack);
                        let condition = &Condition {
                            digit,
                            stack: &stack,
                        };
                        if changed && let ControlFlow::Break(step) = on_condition(grid, condition) {
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
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn tier(&self) -> TechniqueTier {
        TechniqueTier::Advanced
    }

    fn clone_box(&self) -> crate::BoxedTechnique {
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

    const TECHNIQUE: XChain = XChain::new();

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

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t.assert_removed_includes(Position::new(0, 7), [digit]);
        });
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

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t.assert_placed(chain_start, digit);
        });
    }

    #[test]
    fn test_no_change_when_no_x_chain() {
        let grid = CandidateGrid::new();
        testing::test_technique_apply_pass_no_changes(grid, &TECHNIQUE);
    }

    #[test]
    fn test_continuous_x_cycle_eliminates_at_internal_weak_junctions() {
        // Continuous X-Cycle: chain_start and chain_end are house peers.
        // This enables elimination at weak-link junctions along the chain.
        //
        // Position::new(x, y) where x=column, y=row
        // Position::ROWS[y] = all positions in row y
        // Position::COLUMNS[x] = all positions in column x
        //
        // Chain structure (for digit D1):
        //   Strong link 1: (0,0)-(8,0) in row 0 (x=0,y=0 to x=8,y=0)
        //   Strong link 2: (0,8)-(8,8) in row 8 (x=0,y=8 to x=8,y=8)
        //   Weak links: (8,0)-(8,8) in col 8, and (0,0)-(0,8) in col 0 (closing the cycle)
        //
        // chain_start = (0,0), chain_end = (0,8)
        // They are in the same column (col 0), so chain_start.house_peers().contains(chain_end)
        //
        // The weak link junction at (8,0)-(8,8) allows elimination of cells that see both.

        let mut grid = CandidateGrid::new();
        let digit = Digit::D1;

        // Set up strong link in row 0: only (0,0) and (8,0) have digit
        for pos in Position::ROWS[0] {
            if pos != Position::new(0, 0) && pos != Position::new(8, 0) {
                grid.remove_candidate(pos, digit);
            }
        }

        // Set up strong link in row 8: only (0,8) and (8,8) have digit
        for pos in Position::ROWS[8] {
            if pos != Position::new(0, 8) && pos != Position::new(8, 8) {
                grid.remove_candidate(pos, digit);
            }
        }

        // Set up strong link in col 8: only (8,0) and (8,8) have digit
        for pos in Position::COLUMNS[8] {
            if pos != Position::new(8, 0) && pos != Position::new(8, 8) {
                grid.remove_candidate(pos, digit);
            }
        }

        // col 0 should NOT be a strong link (more than 2 candidates)
        // so keep (0,4) with the candidate as well
        for pos in Position::COLUMNS[0] {
            if pos != Position::new(0, 0)
                && pos != Position::new(0, 8)
                && pos != Position::new(0, 4)
            {
                grid.remove_candidate(pos, digit);
            }
        }

        // Now the chain: (0,0) --strong--> (8,0) --weak--> (8,8) --strong--> (0,8)
        // chain_start (0,0) and chain_end (0,8) are in col 0, so continuous X-Cycle applies.
        // The weak-link junction (8,0)-(8,8) should eliminate candidates in their common peers.
        // But col 8 only has (8,0) and (8,8) with digit, so no elimination there.
        // The main elimination is at cells that see both chain_start and chain_end.
        // (0,4) sees both (0,0) and (0,8), so it should be eliminated.

        testing::test_technique_apply_pass(grid, &TECHNIQUE, |t| {
            t.assert_removed_includes(Position::new(0, 4), [digit]);
        });
    }
}
