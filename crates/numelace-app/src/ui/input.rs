use eframe::egui::{InputState, Key};
use numelace_core::Digit;

use crate::{
    action::{Action, ActionRequestQueue, MoveDirection},
    state::ModalKind,
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

    const fn plain(key: Key, action: Action) -> Self {
        Self::new(Trigger::new(key, false, false), action)
    }

    const fn digit(key: Key, digit: Digit, command: bool) -> Self {
        Self::new(
            Trigger::new(key, command, false),
            Action::RequestDigit {
                digit,
                swap: command,
            },
        )
    }
}

const SHORTCUTS: [Shortcut; 31] = [
    Shortcut::command(Key::N, Action::OpenModal(ModalKind::NewGameConfirm)),
    Shortcut::command(Key::Comma, Action::OpenModal(ModalKind::Settings)),
    Shortcut::command_shift(
        Key::Backspace,
        Action::OpenModal(ModalKind::ResetCurrentPuzzleConfirm),
    ),
    Shortcut::command(Key::Z, Action::Undo),
    Shortcut::command(Key::Y, Action::Redo),
    Shortcut::plain(Key::ArrowUp, Action::MoveSelection(MoveDirection::Up)),
    Shortcut::plain(Key::ArrowDown, Action::MoveSelection(MoveDirection::Down)),
    Shortcut::plain(Key::ArrowLeft, Action::MoveSelection(MoveDirection::Left)),
    Shortcut::plain(Key::ArrowRight, Action::MoveSelection(MoveDirection::Right)),
    Shortcut::plain(Key::Escape, Action::ClearSelection),
    Shortcut::plain(Key::S, Action::ToggleInputMode),
    Shortcut::plain(Key::Delete, Action::ClearCell),
    Shortcut::plain(Key::Backspace, Action::ClearCell),
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

pub fn handle_input(i: &InputState, action_queue: &mut ActionRequestQueue) {
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
