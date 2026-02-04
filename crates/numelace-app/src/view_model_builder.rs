use numelace_core::{
    Position,
    containers::{Array9, Array81},
    index::PositionSemantics,
};
use numelace_game::CellState;

use crate::{
    state::{AppState, GhostType, UiState},
    ui::{
        game_screen::GameScreenViewModel,
        grid::{GridCell, GridViewModel, GridVisualState, NoteVisualState},
        keypad::{DigitKeyState, KeypadViewModel},
        settings::SettingsViewModel,
        status_line::{GameStatus, StatusLineViewModel},
        toolbar::ToolbarViewModel,
    },
};

pub fn build_toolbar_vm(ui_state: &UiState) -> ToolbarViewModel {
    ToolbarViewModel::new(ui_state.can_undo(), ui_state.can_redo())
}

fn build_grid(app_state: &AppState, ui_state: &UiState) -> Array81<GridCell, PositionSemantics> {
    let mut grid = Array81::from_fn(|pos| GridCell {
        content: *app_state.game.cell(pos),
        visual_state: GridVisualState::empty(),
        note_visual_state: NoteVisualState::default(),
    });

    if let Some((pos, ghost)) = ui_state.conflict_ghost {
        match ghost {
            GhostType::Digit(digit) => {
                grid[pos].content = CellState::Filled(digit);
                grid[pos].visual_state.insert(GridVisualState::GHOST);
            }
            GhostType::Note(digit) => {
                let mut notes = grid[pos].content.as_notes().unwrap_or_default();
                notes.insert(digit);
                grid[pos].content = CellState::Notes(notes);
                grid[pos].note_visual_state.ghost.insert(digit);
            }
        }
    }

    let selected_cell = app_state.selected_cell;
    let selected_digit = selected_cell.and_then(|pos| grid[pos].content.as_digit());

    // Highlight the selected cell and its house.
    if let Some(pos) = app_state.selected_cell {
        grid[pos].visual_state.insert(GridVisualState::SELECTED);
        for house_pos in pos.house_positions() {
            grid[house_pos]
                .visual_state
                .insert(GridVisualState::HOUSE_SELECTED);
        }
    }

    for pos in Position::ALL {
        let cell_digit = grid[pos].content.as_digit();
        let cell_notes = grid[pos].content.as_notes();

        // Highlight conflicts in the same house.
        if let Some(digit) = cell_digit {
            for peer_pos in pos.house_peers() {
                let peer_digit = grid[peer_pos].content.as_digit();
                let peer_notes = grid[peer_pos].content.as_notes();
                if peer_digit == Some(digit) {
                    grid[peer_pos]
                        .visual_state
                        .insert(GridVisualState::CONFLICT);
                }
                if peer_notes.is_some_and(|notes| notes.contains(digit)) {
                    grid[peer_pos].note_visual_state.conflict.insert(digit);
                }
            }
        }

        // Highlight same digits and notes as the selected cell.
        if let Some(digit) = selected_digit {
            if cell_digit == Some(digit) {
                grid[pos].visual_state.insert(GridVisualState::SAME_DIGIT);
                for house_pos in pos.house_positions() {
                    grid[house_pos]
                        .visual_state
                        .insert(GridVisualState::HOUSE_SAME_DIGIT);
                }
            }

            if cell_notes.is_some_and(|notes| notes.contains(digit)) {
                grid[pos].note_visual_state.same_digit.insert(digit);
            }
        }
    }

    grid
}

pub fn build_game_screen_view_model(
    app_state: &AppState,
    ui_state: &UiState,
) -> GameScreenViewModel {
    let game = &app_state.game;
    let selected_cell = app_state.selected_cell;
    let settings = &app_state.settings;
    let notes_mode = app_state.input_mode.is_notes();

    let status = if app_state.game.is_solved() {
        GameStatus::Solved
    } else {
        GameStatus::InProgress
    };
    let status_line_vm = StatusLineViewModel::new(status);
    let toolbar_vm = build_toolbar_vm(ui_state);

    let grid = build_grid(app_state, ui_state);
    let grid_vm = GridViewModel::new(grid, &settings.assist.highlight);

    let policy = app_state.rule_check_policy();
    let decided_digit_count = game.decided_digit_count();
    let digit_capabilities = Array9::from_fn(|digit| {
        let set_digit = selected_cell.map(|pos| game.set_digit_capability(pos, digit, policy));
        let toggle_note = selected_cell.map(|pos| game.toggle_note_capability(pos, digit, policy));
        DigitKeyState::new(set_digit, toggle_note, decided_digit_count[digit])
    });
    let has_removable_input = selected_cell.is_some_and(|pos| game.has_removable_input(pos));
    let auto_fill_capability = selected_cell.map(|pos| game.auto_fill_cell_notes_capability(pos));
    let keypad_vm = KeypadViewModel::new(
        digit_capabilities,
        has_removable_input,
        notes_mode,
        auto_fill_capability,
    );

    GameScreenViewModel::new(toolbar_vm, status_line_vm, grid_vm, keypad_vm)
}

pub fn build_settings_view_model(app_state: &AppState) -> SettingsViewModel<'_> {
    let settings = &app_state.settings;
    SettingsViewModel::new(settings)
}

#[cfg(test)]
mod tests {
    use numelace_core::{Digit, DigitGrid, Position};
    use numelace_game::{CellState, Game};

    use super::build_grid;
    use crate::{
        DEFAULT_MAX_HISTORY_LENGTH,
        state::{AppState, GhostType, UiState},
        ui::grid::GridVisualState,
    };

    fn blank_grid() -> DigitGrid {
        "\
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
        .unwrap()
    }

    fn filled_with_conflict() -> DigitGrid {
        "\
11.......\
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
        .unwrap()
    }

    fn game_from_filled(filled: &DigitGrid) -> Game {
        let problem = blank_grid();
        let solution: DigitGrid =
            "185362947793148526246795183564239871931874265827516394318427659672951438459683712"
                .parse()
                .unwrap();
        let notes = [[0u16; 9]; 9];
        Game::from_problem_filled_notes(&problem, &solution, filled, &notes).unwrap()
    }

    #[test]
    fn build_grid_highlights_selected_conflict_and_same_digit() {
        let mut app_state = AppState::new(game_from_filled(&filled_with_conflict()));
        app_state.selected_cell = Some(Position::new(0, 0));
        let ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);

        let grid = build_grid(&app_state, &ui_state);

        assert!(
            grid[Position::new(0, 0)]
                .visual_state
                .contains(GridVisualState::SELECTED)
        );
        assert!(
            grid[Position::new(1, 0)]
                .visual_state
                .contains(GridVisualState::CONFLICT)
        );
        assert!(
            grid[Position::new(1, 0)]
                .visual_state
                .contains(GridVisualState::SAME_DIGIT)
        );
        assert!(
            grid[Position::new(1, 1)]
                .visual_state
                .contains(GridVisualState::HOUSE_SELECTED)
        );
        assert!(
            grid[Position::new(2, 2)]
                .visual_state
                .contains(GridVisualState::HOUSE_SAME_DIGIT)
        );
    }

    #[test]
    fn build_grid_applies_digit_ghost() {
        let app_state = AppState::new(game_from_filled(&blank_grid()));
        let mut ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);
        ui_state.conflict_ghost = Some((Position::new(3, 3), GhostType::Digit(Digit::D2)));

        let grid = build_grid(&app_state, &ui_state);

        assert!(matches!(
            grid[Position::new(3, 3)].content,
            CellState::Filled(Digit::D2)
        ));
        assert!(
            grid[Position::new(3, 3)]
                .visual_state
                .contains(GridVisualState::GHOST)
        );
    }
}
