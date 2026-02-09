use std::{collections::VecDeque, num::NonZero};

use numelace_core::{DigitGrid, Position};
use numelace_game::{CellState, Game};

use crate::undo_redo_stack::UndoRedoStack;

#[derive(Debug)]
pub(crate) struct HistorySource<'a> {
    pub(crate) game: &'a Game,
    pub(crate) selected_cell: Option<Position>,
}

impl<'a> HistorySource<'a> {
    pub(crate) fn new(game: &'a Game, selected_cell: Option<Position>) -> Self {
        Self {
            game,
            selected_cell,
        }
    }
}

#[derive(Debug)]
pub(crate) struct HistoryTarget<'a> {
    pub(crate) game: &'a mut Game,
    pub(crate) selected_cell: &'a mut Option<Position>,
}

impl<'a> HistoryTarget<'a> {
    pub(crate) fn new(game: &'a mut Game, selected_cell: &'a mut Option<Position>) -> Self {
        Self {
            game,
            selected_cell,
        }
    }
}

#[derive(Debug)]
pub(crate) struct History {
    stack: UndoRedoStack<HistorySnapshot>,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    pub const fn default_capacity() -> NonZero<usize> {
        NonZero::new(5000).unwrap()
    }

    pub(crate) fn new() -> Self {
        Self::with_capacity(Self::default_capacity())
    }

    pub(crate) fn with_capacity(capacity: NonZero<usize>) -> Self {
        Self {
            stack: UndoRedoStack::new(capacity),
        }
    }

    pub(crate) fn from_parts(
        capacity: NonZero<usize>,
        entries: Vec<HistorySnapshot>,
        cursor: usize,
    ) -> Self {
        let mut stack = UndoRedoStack::new(capacity);
        let entries = VecDeque::from(entries);
        stack.restore_from_parts(entries, cursor);
        Self { stack }
    }

    pub(crate) fn capacity(&self) -> NonZero<usize> {
        self.stack.capacity()
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = &HistorySnapshot> {
        self.stack.entries()
    }

    pub(crate) fn cursor(&self) -> usize {
        self.stack.cursor()
    }

    pub(crate) fn build_undo_games(&self, game: &Game) -> Vec<Game> {
        let (problem, solution) = base_problem_and_solution(game);
        self.stack
            .iter_from_current()
            .map(|snapshot| {
                Game::from_problem_filled_notes(
                    &problem,
                    &solution,
                    &snapshot.filled,
                    &snapshot.notes,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default()
    }

    pub(crate) fn reset(&mut self, source: &HistorySource<'_>) {
        self.stack.clear();
        self.stack.push(HistorySnapshot::new(source));
    }

    pub(crate) fn can_undo(&self) -> bool {
        self.stack.can_undo()
    }

    pub(crate) fn undo(&mut self, target: &mut HistoryTarget<'_>) -> bool {
        let Some(current) = self.stack.current() else {
            return false;
        };
        let change_location = current.selected_at_change;
        if self.stack.undo()
            && let Some(snapshot) = self.stack.current().cloned()
            && snapshot.apply(target)
        {
            *target.selected_cell = change_location;
            true
        } else {
            false
        }
    }

    pub(crate) fn undo_steps(&mut self, steps: usize, target: &mut HistoryTarget<'_>) -> bool {
        if steps == 0 {
            return true;
        }
        let mut undone = 0;
        for _ in 0..steps {
            if self.undo(target) {
                undone += 1;
            } else {
                break;
            }
        }
        undone > 0
    }

    pub(crate) fn can_redo(&self) -> bool {
        self.stack.can_redo()
    }

    pub(crate) fn redo(&mut self, target: &mut HistoryTarget<'_>) -> bool {
        if self.stack.redo()
            && let Some(snapshot) = self.stack.current().cloned()
            && snapshot.apply(target)
        {
            true
        } else {
            false
        }
    }

    pub(crate) fn push(&mut self, source: &HistorySource<'_>) {
        let snapshot = HistorySnapshot::new(source);
        self.stack.push(snapshot);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HistorySnapshot {
    pub(crate) filled: DigitGrid,
    pub(crate) notes: [[u16; 9]; 9],
    pub(crate) selected_at_change: Option<Position>,
}

fn base_problem_and_solution(game: &Game) -> (DigitGrid, DigitGrid) {
    let mut problem = DigitGrid::new();
    for pos in Position::ALL {
        if let CellState::Given(digit) = game.cell(pos) {
            problem.set(pos, Some(*digit));
        }
    }
    (problem, game.solution().clone())
}

impl HistorySnapshot {
    fn new(source: &HistorySource<'_>) -> Self {
        let mut filled = DigitGrid::new();
        let mut notes = [[0u16; 9]; 9];
        for pos in Position::ALL {
            match source.game.cell(pos) {
                CellState::Filled(digit) => {
                    filled.set(pos, Some(*digit));
                }
                CellState::Notes(digits) => {
                    notes[usize::from(pos.y())][usize::from(pos.x())] = digits.bits();
                }
                CellState::Given(_) | CellState::Empty => {}
            }
        }
        Self {
            filled,
            notes,
            selected_at_change: source.selected_cell,
        }
    }

    fn apply(&self, target: &mut HistoryTarget<'_>) -> bool {
        let (problem, solution) = base_problem_and_solution(target.game);
        match Game::from_problem_filled_notes(&problem, &solution, &self.filled, &self.notes) {
            Ok(new_game) => {
                *target.game = new_game;
                *target.selected_cell = self.selected_at_change;
                true
            }
            Err(_) => false,
        }
    }
}
