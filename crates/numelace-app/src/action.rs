use std::mem;

use numelace_core::{Digit, Position};

use crate::state::{ModalKind, Settings};

#[derive(Debug, Clone)]
pub enum Action {
    SelectCell(Position),
    ClearSelection,
    MoveSelection(MoveDirection),
    ToggleInputMode,
    RequestDigit { digit: Digit, swap: bool },
    ClearCell,
    Undo,
    Redo,
    OpenModal(ModalKind),
    CloseModal,
    StartNewGame,
    ResetCurrentPuzzle,
    UpdateSettings(Settings),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Default)]
pub struct ActionRequestQueue {
    actions: Vec<Action>,
}

impl ActionRequestQueue {
    pub fn request(&mut self, action: Action) {
        self.actions.push(action);
    }

    pub fn take_all(&mut self) -> Vec<Action> {
        mem::take(&mut self.actions)
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, ActionRequestQueue};

    #[test]
    fn take_all_returns_actions_and_clears_queue() {
        let mut queue = ActionRequestQueue::default();
        queue.request(Action::ToggleInputMode);
        queue.request(Action::ClearCell);

        let drained = queue.take_all();
        assert_eq!(drained.len(), 2);
        assert!(matches!(drained[0], Action::ToggleInputMode));
        assert!(matches!(drained[1], Action::ClearCell));

        let drained_again = queue.take_all();
        assert!(drained_again.is_empty());
    }
}
