use eframe::egui::{InputState, Key};
use numelace_core::Digit;

use crate::action::{
    Action, ActionRequestQueue, AppAction, BoardMutationAction, FlowAction, HistoryAction,
    InputModeAction, ModalRequest, MoveDirection, NotesFillScope, SelectionAction, UiAction,
};

struct Trigger {
    key: Key,
    command: bool,
    shift: bool,
}

impl Trigger {
    const fn new(key: Key, command: bool, shift: bool) -> Self {
        Self {
            key,
            command,
            shift,
        }
    }
}

struct Shortcut {
    trigger: Trigger,
    action: Action,
}

impl Shortcut {
    const fn new(trigger: Trigger, action: Action) -> Self {
        Self { trigger, action }
    }

    const fn command(key: Key, action: Action) -> Self {
        Self::new(Trigger::new(key, true, false), action)
    }

    const fn command_shift(key: Key, action: Action) -> Self {
        Self::new(Trigger::new(key, true, true), action)
    }

    const fn shift(key: Key, action: Action) -> Self {
        Self::new(Trigger::new(key, false, true), action)
    }

    const fn plain(key: Key, action: Action) -> Self {
        Self::new(Trigger::new(key, false, false), action)
    }

    const fn digit(key: Key, digit: Digit, command: bool) -> Self {
        Self::new(
            Trigger::new(key, command, false),
            Action::App(AppAction::BoardMutation(
                BoardMutationAction::RequestDigit {
                    digit,
                    swap: command,
                },
            )),
        )
    }
}

const fn board_mutation_action(action: BoardMutationAction) -> Action {
    Action::App(AppAction::BoardMutation(action))
}

const fn history_action(action: HistoryAction) -> Action {
    Action::App(AppAction::History(action))
}

const fn selection_action(action: SelectionAction) -> Action {
    Action::App(AppAction::Selection(action))
}

const fn move_selection_action(direction: MoveDirection) -> Action {
    selection_action(SelectionAction::MoveSelection(direction))
}

const fn input_mode_action(action: InputModeAction) -> Action {
    Action::App(AppAction::InputMode(action))
}

const SHORTCUTS: [Shortcut; 34] = [
    Shortcut::command(Key::N, Action::Flow(FlowAction::StartNewGame)),
    Shortcut::command(
        Key::Comma,
        Action::Ui(UiAction::OpenModal(ModalRequest::Settings)),
    ),
    Shortcut::command_shift(Key::Backspace, Action::Flow(FlowAction::ResetInputs)),
    Shortcut::command(Key::K, Action::Flow(FlowAction::CheckSolvability)),
    Shortcut::command(Key::Z, history_action(HistoryAction::Undo)),
    Shortcut::command(Key::Y, history_action(HistoryAction::Redo)),
    Shortcut::plain(Key::ArrowUp, move_selection_action(MoveDirection::Up)),
    Shortcut::plain(Key::ArrowDown, move_selection_action(MoveDirection::Down)),
    Shortcut::plain(Key::ArrowLeft, move_selection_action(MoveDirection::Left)),
    Shortcut::plain(Key::ArrowRight, move_selection_action(MoveDirection::Right)),
    Shortcut::plain(
        Key::Escape,
        selection_action(SelectionAction::ClearSelection),
    ),
    Shortcut::plain(Key::S, input_mode_action(InputModeAction::ToggleInputMode)),
    Shortcut::plain(
        Key::A,
        board_mutation_action(BoardMutationAction::AutoFillNotes {
            scope: NotesFillScope::Cell,
        }),
    ),
    Shortcut::shift(
        Key::A,
        board_mutation_action(BoardMutationAction::AutoFillNotes {
            scope: NotesFillScope::AllCells,
        }),
    ),
    Shortcut::plain(
        Key::Delete,
        board_mutation_action(BoardMutationAction::ClearCell),
    ),
    Shortcut::plain(
        Key::Backspace,
        board_mutation_action(BoardMutationAction::ClearCell),
    ),
    Shortcut::digit(Key::Num1, Digit::D1, true),
    Shortcut::digit(Key::Num1, Digit::D1, false),
    Shortcut::digit(Key::Num2, Digit::D2, true),
    Shortcut::digit(Key::Num2, Digit::D2, false),
    Shortcut::digit(Key::Num3, Digit::D3, true),
    Shortcut::digit(Key::Num3, Digit::D3, false),
    Shortcut::digit(Key::Num4, Digit::D4, true),
    Shortcut::digit(Key::Num4, Digit::D4, false),
    Shortcut::digit(Key::Num5, Digit::D5, true),
    Shortcut::digit(Key::Num5, Digit::D5, false),
    Shortcut::digit(Key::Num6, Digit::D6, true),
    Shortcut::digit(Key::Num6, Digit::D6, false),
    Shortcut::digit(Key::Num7, Digit::D7, true),
    Shortcut::digit(Key::Num7, Digit::D7, false),
    Shortcut::digit(Key::Num8, Digit::D8, true),
    Shortcut::digit(Key::Num8, Digit::D8, false),
    Shortcut::digit(Key::Num9, Digit::D9, true),
    Shortcut::digit(Key::Num9, Digit::D9, false),
];

pub(crate) fn handle_input(i: &InputState, action_queue: &mut ActionRequestQueue) {
    // `i.modifiers.command` is true when Ctrl (Windows/Linux) or Cmd (Mac) is pressed
    for shortcut in SHORTCUTS {
        let triggered = i.key_pressed(shortcut.trigger.key)
            && i.modifiers.command == shortcut.trigger.command
            && i.modifiers.shift == shortcut.trigger.shift;

        if triggered {
            action_queue.request(shortcut.action);
            return;
        }
    }
}
