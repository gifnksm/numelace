use eframe::egui::{InputState, Key};
use numelace_core::Digit;

use crate::ui::Action;

pub fn handle_input(i: &InputState) -> Vec<Action> {
    let mut actions = vec![];
    if i.modifiers.command && i.key_pressed(Key::N) {
        actions.push(Action::RequestNewGameConfirm);
    }
    if i.key_pressed(Key::ArrowUp) {
        actions.push(Action::MoveSelection(crate::ui::MoveDirection::Up));
    }
    if i.key_pressed(Key::ArrowDown) {
        actions.push(Action::MoveSelection(crate::ui::MoveDirection::Down));
    }
    if i.key_pressed(Key::ArrowLeft) {
        actions.push(Action::MoveSelection(crate::ui::MoveDirection::Left));
    }
    if i.key_pressed(Key::ArrowRight) {
        actions.push(Action::MoveSelection(crate::ui::MoveDirection::Right));
    }
    if i.key_pressed(Key::Escape) {
        actions.push(Action::ClearSelection);
    }
    let pairs = [
        (Key::Delete, None),
        (Key::Backspace, None),
        (Key::Num1, Some(Digit::D1)),
        (Key::Num2, Some(Digit::D2)),
        (Key::Num3, Some(Digit::D3)),
        (Key::Num4, Some(Digit::D4)),
        (Key::Num5, Some(Digit::D5)),
        (Key::Num6, Some(Digit::D6)),
        (Key::Num7, Some(Digit::D7)),
        (Key::Num8, Some(Digit::D8)),
        (Key::Num9, Some(Digit::D9)),
    ];
    for (key, digit) in pairs {
        if i.key_pressed(key) {
            if let Some(digit) = digit {
                actions.push(Action::SetDigit(digit));
            } else {
                actions.push(Action::RemoveDigit);
            }
        }
    }
    actions
}
