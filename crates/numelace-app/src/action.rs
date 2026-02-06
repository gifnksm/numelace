use std::mem;

use numelace_core::{Digit, Position};

use crate::async_work::{WorkRequest, WorkResponse};
use crate::state::{ModalKind, Settings};

#[derive(Debug, Clone)]
pub(crate) enum Action {
    SelectCell(Position),
    ClearSelection,
    MoveSelection(MoveDirection),
    ToggleInputMode,
    RequestDigit { digit: Digit, swap: bool },
    ClearCell,
    AutoFillNotes { scope: NotesFillScope },
    CheckSolvability,
    Undo,
    Redo,
    OpenModal(ModalKind),
    CloseModal,
    StartWork(WorkRequest),
    ResetCurrentPuzzle,
    ApplyWorkResponse(WorkResponse),
    UpdateSettings(Settings),
    StartNewGameFlow,
    ModalResponse(ModalResponse),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfirmResult {
    Confirmed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SolvabilityDialogResult {
    Close,
    RebuildNotes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModalResponse {
    Confirm(ConfirmResult),
    Solvability(SolvabilityDialogResult),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub(crate) enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub(crate) enum NotesFillScope {
    Cell,
    AllCells,
}

#[derive(Debug, Default)]
pub(crate) struct ActionRequestQueue {
    actions: Vec<Action>,
}

impl ActionRequestQueue {
    pub(crate) fn request(&mut self, action: Action) {
        self.actions.push(action);
    }

    pub(crate) fn take_all(&mut self) -> Vec<Action> {
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
