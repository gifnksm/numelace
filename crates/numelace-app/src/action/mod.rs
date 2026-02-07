use std::mem;

use numelace_core::{Digit, Position};
use numelace_generator::GeneratedPuzzle;

use crate::state::{Settings, SolvabilityState};
use crate::worker::tasks::SolvabilityUndoGridsDto;

pub(crate) mod flows;
pub(crate) mod handler;

#[derive(Debug, derive_more::From)]
pub(crate) enum Action {
    App(AppAction),
    Ui(UiAction),
    Flow(FlowAction),
}

#[derive(Debug, derive_more::From)]
pub(crate) enum AppAction {
    BoardMutation(BoardMutationAction),
    PuzzleLifecycle(PuzzleLifecycleAction),
    History(HistoryAction),
    StateQuery(StateQueryAction),
    Selection(SelectionAction),
    InputMode(InputModeAction),
    Settings(SettingsAction),
}

#[derive(Debug)]
pub(crate) enum BoardMutationAction {
    RequestDigit { digit: Digit, swap: bool },
    ClearCell,
    AutoFillNotes { scope: NotesFillScope },
    ResetInputs,
}

#[derive(Debug)]
pub(crate) enum PuzzleLifecycleAction {
    StartNewGame(GeneratedPuzzle),
}

#[derive(Debug)]
pub(crate) enum HistoryAction {
    Undo,
    UndoSteps(usize),
    Redo,
}

#[derive(Debug)]
pub(crate) enum StateQueryAction {
    BuildSolvabilityUndoGrids {
        responder: SolvabilityUndoGridsResponder,
    },
}

#[derive(Debug)]
pub(crate) enum SelectionAction {
    SelectCell(Position),
    ClearSelection,
    MoveSelection(MoveDirection),
}

#[derive(Debug)]
pub(crate) enum InputModeAction {
    ToggleInputMode,
}

#[derive(Debug)]
pub(crate) enum SettingsAction {
    UpdateSettings(Settings),
}

#[derive(Debug)]
pub(crate) enum UiAction {
    OpenModal(ModalRequest),
    CloseModal,
    StartSpinner { id: SpinnerId, kind: SpinnerKind },
    StopSpinner { id: SpinnerId },
}

#[derive(Debug)]
pub(crate) enum FlowAction {
    StartNewGame,
    ResetInputs,
    CheckSolvability,
}

impl From<BoardMutationAction> for Action {
    fn from(action: BoardMutationAction) -> Self {
        Action::App(action.into())
    }
}

impl From<PuzzleLifecycleAction> for Action {
    fn from(action: PuzzleLifecycleAction) -> Self {
        Action::App(action.into())
    }
}

impl From<HistoryAction> for Action {
    fn from(action: HistoryAction) -> Self {
        Action::App(action.into())
    }
}

impl From<StateQueryAction> for Action {
    fn from(action: StateQueryAction) -> Self {
        Action::App(action.into())
    }
}

impl From<SelectionAction> for Action {
    fn from(action: SelectionAction) -> Self {
        Action::App(action.into())
    }
}

impl From<InputModeAction> for Action {
    fn from(action: InputModeAction) -> Self {
        Action::App(action.into())
    }
}

impl From<SettingsAction> for Action {
    fn from(action: SettingsAction) -> Self {
        Action::App(action.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SpinnerId(u64);

impl SpinnerId {
    #[must_use]
    pub(crate) fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinnerKind {
    NewGame,
    CheckSolvability,
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
    Undo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SolvabilityUndoNoticeResult {
    Close,
}

pub(crate) type Responder<T> = futures_channel::oneshot::Sender<T>;
pub(crate) type ConfirmResponder = Responder<ConfirmResult>;
pub(crate) type SolvabilityResponder = Responder<SolvabilityDialogResult>;
pub(crate) type SolvabilityUndoGridsResponder = Responder<SolvabilityUndoGridsDto>;
pub(crate) type SolvabilityUndoNoticeResponder = Responder<SolvabilityUndoNoticeResult>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfirmKind {
    NewGame,
    ResetInputs,
}

#[derive(Debug)]
pub(crate) enum ModalRequest {
    Confirm {
        kind: ConfirmKind,
        responder: Option<ConfirmResponder>,
    },
    CheckSolvabilityResult {
        state: SolvabilityState,
        responder: Option<SolvabilityResponder>,
    },
    SolvabilityUndoNotice {
        steps: usize,
        responder: Option<SolvabilityUndoNoticeResponder>,
    },
    SolvabilityUndoNotFound,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub(crate) enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

impl MoveDirection {
    pub(crate) fn apply_to(self, pos: Position) -> Option<Position> {
        match self {
            Self::Up => pos.up(),
            Self::Down => pos.down(),
            Self::Left => pos.left(),
            Self::Right => pos.right(),
        }
    }
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
    use crate::action::{AppAction, BoardMutationAction, InputModeAction};

    use super::{Action, ActionRequestQueue};

    #[test]
    fn take_all_returns_actions_and_clears_queue() {
        let mut queue = ActionRequestQueue::default();
        queue.request(InputModeAction::ToggleInputMode.into());
        queue.request(BoardMutationAction::ClearCell.into());

        let drained = queue.take_all();
        assert_eq!(drained.len(), 2);
        assert!(matches!(
            drained[0],
            Action::App(AppAction::InputMode(InputModeAction::ToggleInputMode))
        ));
        assert!(matches!(
            drained[1],
            Action::App(AppAction::BoardMutation(BoardMutationAction::ClearCell))
        ));

        let drained_again = queue.take_all();
        assert!(drained_again.is_empty());
    }
}
