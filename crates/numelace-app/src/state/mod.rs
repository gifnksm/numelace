pub(crate) use self::{app_state::*, history::*, settings::*, ui_state::*};

mod app_state;
mod history;
mod settings;
mod ui_state;

#[cfg(test)]
mod tests {
    use numelace_core::{Digit, DigitGrid, Position};
    use numelace_game::{CellState, Game, InputDigitOptions};

    use super::AppState;

    fn fixed_game() -> Game {
        let problem: DigitGrid = "\
.1.......\
.........\
.........\
.........\
.........\
.........\
.........\
.........\
.........\
"
        .parse()
        .unwrap();
        let solution: DigitGrid =
            "185362947793148526246795183564239871931874265827516394318427659672951438459683712"
                .parse()
                .unwrap();
        let filled: DigitGrid = "\
.........\
.........\
.........\
.........\
.........\
.........\
.........\
.........\
.........\
"
        .parse()
        .unwrap();
        let notes = [[0u16; 9]; 9];
        Game::from_problem_filled_notes(&problem, &solution, &filled, &notes).unwrap()
    }

    #[test]
    fn undo_redo_restores_game_and_selection() {
        let mut app_state = AppState::new(fixed_game());

        app_state.selected_cell = Some(Position::new(0, 0));
        app_state
            .game
            .set_digit(
                Position::new(0, 0),
                Digit::D2,
                &InputDigitOptions::default(),
            )
            .unwrap();
        app_state.push_history();

        app_state.selected_cell = Some(Position::new(2, 0));
        app_state
            .game
            .set_digit(
                Position::new(2, 0),
                Digit::D3,
                &InputDigitOptions::default(),
            )
            .unwrap();
        app_state.push_history();

        assert!(app_state.undo());

        assert!(matches!(
            app_state.game.cell(Position::new(0, 0)),
            CellState::Filled(Digit::D2)
        ));
        assert!(matches!(
            app_state.game.cell(Position::new(2, 0)),
            CellState::Empty
        ));
        assert_eq!(app_state.selected_cell, Some(Position::new(2, 0)));

        assert!(app_state.redo());

        assert!(matches!(
            app_state.game.cell(Position::new(2, 0)),
            CellState::Filled(Digit::D3)
        ));
        assert_eq!(app_state.selected_cell, Some(Position::new(2, 0)));
    }
}
