use numelace_core::{
    Digit, DigitPositions, DigitSet, Position,
    containers::{Array9, Array81},
    index::{DigitSemantics, PositionSemantics},
};
use numelace_game::CellState;
use numelace_solver::TechniqueApplication;

use crate::{
    state::{AppState, GhostType, HintStage, HintState, UiState},
    ui::{
        game_screen::GameScreenViewModel,
        grid::{GridCell, GridViewModel, GridVisualState, NoteVisualState},
        keypad::{DigitKeyState, KeypadViewModel},
        modal::SettingsViewModel,
        status_line::{GameStatus, StatusLineViewModel},
        toolbar::ToolbarViewModel,
    },
};

#[must_use]
pub(crate) fn build_toolbar_vm(app_state: &AppState, _ui_state: &UiState) -> ToolbarViewModel {
    ToolbarViewModel::new(app_state.can_undo(), app_state.can_redo())
}

fn fill_notes_for_empty_cell(
    grid: &mut Array81<GridCell, PositionSemantics>,
    pos: Position,
) -> DigitSet {
    assert!(grid[pos].content.is_empty());
    let mut notes = DigitSet::FULL;
    for peer_pos in pos.house_peers() {
        if let Some(digit) = grid[peer_pos].content.as_digit() {
            notes.remove(digit);
        }
    }
    grid[pos].content = CellState::Notes(notes);
    notes
}

fn apply_conflict_ghost(
    grid: &mut Array81<GridCell, PositionSemantics>,
    pos: Position,
    ghost: GhostType,
) {
    match ghost {
        GhostType::Digit(digit) => {
            grid[pos].content = CellState::Filled(digit);
            grid[pos].visual_state |= GridVisualState::GHOST;
        }
        GhostType::Note(digit) => {
            let mut notes = grid[pos].content.as_notes().unwrap_or_default();
            notes.insert(digit);
            grid[pos].content = CellState::Notes(notes);
            grid[pos].note_visual_state.ghost.insert(digit);
        }
    }
}

fn effective_hint_applications(
    grid: &Array81<GridCell, PositionSemantics>,
    hint_state: &HintState,
) -> Vec<TechniqueApplication> {
    let mut apps = Vec::new();

    for app in hint_state.step.application() {
        match app {
            TechniqueApplication::Placement { position, digit } => {
                if grid[position].content.as_digit() != Some(digit) {
                    apps.push(TechniqueApplication::Placement { position, digit });
                }
            }
            TechniqueApplication::CandidateElimination { positions, digits } => {
                let mut by_digits: Array9<DigitPositions, DigitSemantics> =
                    Array9::from_fn(|_| DigitPositions::EMPTY);
                for pos in positions {
                    let notes = if grid[pos].content.is_empty() {
                        let mut notes = DigitSet::FULL;
                        for peer_pos in pos.house_peers() {
                            if let Some(digit) = grid[peer_pos].content.as_digit() {
                                notes.remove(digit);
                            }
                        }
                        notes
                    } else {
                        grid[pos].content.as_notes().unwrap_or_default()
                    };
                    for digit in digits {
                        if notes.contains(digit) {
                            by_digits[digit].insert(pos);
                        }
                    }
                }

                for digit in Digit::ALL {
                    let positions = by_digits[digit];
                    if !positions.is_empty() {
                        apps.push(TechniqueApplication::CandidateElimination {
                            positions,
                            digits: DigitSet::from_elem(digit),
                        });
                    }
                }
            }
        }
    }

    apps
}

fn apply_hint_ghost(grid: &mut Array81<GridCell, PositionSemantics>, hint_state: &HintState) {
    if hint_state.stage >= HintStage::Stage3Apply {
        return;
    }

    for pos in hint_state.step.condition_cells() {
        if grid[pos].content.is_empty() {
            let notes = fill_notes_for_empty_cell(grid, pos);
            grid[pos].note_visual_state.ghost |= notes;
            grid[pos].note_visual_state.hint_condition_temporary |= notes;
        }
    }

    if hint_state.stage >= HintStage::Stage3Preview {
        for app in effective_hint_applications(grid, hint_state) {
            match app {
                TechniqueApplication::Placement { position, digit } => {
                    if grid[position].content.as_digit() != Some(digit) {
                        grid[position].content = CellState::Filled(digit);
                        grid[position].visual_state |= GridVisualState::GHOST;
                    }
                    grid[position].visual_state |= GridVisualState::HINT_APPLICATION_PLACEMENT;
                }
                TechniqueApplication::CandidateElimination { positions, digits } => {
                    for pos in positions {
                        if grid[pos].content.is_empty() {
                            let notes = fill_notes_for_empty_cell(grid, pos);
                            grid[pos].note_visual_state.ghost |= notes;
                            grid[pos].note_visual_state.hint_application_temporary |= notes;
                        }
                        for digit in digits {
                            if let Some(mut notes) = grid[pos].content.as_notes()
                                && !notes.contains(digit)
                            {
                                notes.insert(digit);
                                grid[pos].content = CellState::Notes(notes);
                                grid[pos].note_visual_state.ghost.insert(digit);
                            }
                            grid[pos]
                                .note_visual_state
                                .hint_application_elimination
                                .insert(digit);
                        }
                    }
                }
            }
        }
    }
}

fn apply_hint_visuals(grid: &mut Array81<GridCell, PositionSemantics>, hint_state: &HintState) {
    if hint_state.stage >= HintStage::Stage3Apply {
        return;
    }

    for pos in hint_state.step.condition_cells() {
        grid[pos].visual_state |= GridVisualState::HINT_CONDITION_CELL;
    }

    if hint_state.stage >= HintStage::Stage2 {
        for (positions, digits) in hint_state.step.condition_digit_cells() {
            for pos in positions {
                if let Some(cell_digit) = grid[pos].content.as_digit()
                    && digits.contains(cell_digit)
                {
                    grid[pos].visual_state |= GridVisualState::HINT_CONDITION_DIGIT;
                }
                if let Some(notes) = grid[pos].content.as_notes() {
                    for digit in digits {
                        if notes.contains(digit) {
                            grid[pos]
                                .note_visual_state
                                .hint_condition_digit
                                .insert(digit);
                        }
                    }
                }
            }
        }
    }

    if hint_state.stage >= HintStage::Stage3Preview {
        for app in effective_hint_applications(grid, hint_state) {
            match app {
                TechniqueApplication::Placement { position, digit: _ } => {
                    grid[position].visual_state |= GridVisualState::HINT_APPLICATION_PLACEMENT;
                }
                TechniqueApplication::CandidateElimination { positions, digits } => {
                    for pos in positions {
                        grid[pos].visual_state |= GridVisualState::HINT_APPLICATION_ELIMINATION;
                        for digit in digits {
                            grid[pos]
                                .note_visual_state
                                .hint_application_elimination
                                .insert(digit);
                        }
                    }
                }
            }
        }
    }
}

fn apply_selection_highlights(grid: &mut Array81<GridCell, PositionSemantics>, pos: Position) {
    grid[pos].visual_state |= GridVisualState::SELECTED;
    for house_pos in pos.house_positions() {
        grid[house_pos].visual_state |= GridVisualState::HOUSE_SELECTED;
    }
}

fn apply_conflict_highlights(grid: &mut Array81<GridCell, PositionSemantics>) {
    for pos in Position::ALL {
        let Some(digit) = grid[pos].content.as_digit() else {
            continue;
        };

        for peer_pos in pos.house_peers() {
            if grid[peer_pos].content.as_digit() == Some(digit) {
                grid[peer_pos].visual_state |= GridVisualState::CONFLICT;
            }
            if grid[peer_pos]
                .content
                .as_notes()
                .is_some_and(|notes| notes.contains(digit))
            {
                grid[peer_pos].note_visual_state.conflict.insert(digit);
            }
        }
    }
}

fn apply_selected_digit_highlights(grid: &mut Array81<GridCell, PositionSemantics>, digit: Digit) {
    for pos in Position::ALL {
        if grid[pos].content.as_digit() == Some(digit) {
            grid[pos].visual_state |= GridVisualState::SAME_DIGIT;
            for house_pos in pos.house_positions() {
                grid[house_pos].visual_state |= GridVisualState::HOUSE_SAME_DIGIT;
            }
        }

        if grid[pos]
            .content
            .as_notes()
            .is_some_and(|notes| notes.contains(digit))
        {
            grid[pos].note_visual_state.same_digit.insert(digit);
        }
    }
}

fn build_grid(app_state: &AppState, ui_state: &UiState) -> Array81<GridCell, PositionSemantics> {
    let mut grid = Array81::from_fn(|pos| GridCell {
        content: *app_state.game.cell(pos),
        visual_state: GridVisualState::empty(),
        note_visual_state: NoteVisualState::default(),
    });

    if let Some((pos, ghost)) = ui_state.conflict_ghost {
        apply_conflict_ghost(&mut grid, pos, ghost);
    }

    if let Some(hint_state) = &ui_state.hint_state {
        apply_hint_ghost(&mut grid, hint_state);
        apply_hint_visuals(&mut grid, hint_state);
    }

    apply_conflict_highlights(&mut grid);

    if let Some(pos) = app_state.selected_cell {
        apply_selection_highlights(&mut grid, pos);
        if let Some(digit) = grid[pos].content.as_digit() {
            apply_selected_digit_highlights(&mut grid, digit);
        }
    }

    grid
}

#[must_use]
pub(crate) fn build_game_screen_view_model<'a>(
    app_state: &AppState,
    ui_state: &'a UiState,
) -> GameScreenViewModel<'a> {
    let game = &app_state.game;
    let selected_cell = app_state.selected_cell;
    let settings = &app_state.settings;
    let notes_mode = app_state.input_mode.is_notes();

    let status = if app_state.game.is_solved() {
        GameStatus::Solved
    } else if let Some(hint_state) = &ui_state.hint_state {
        GameStatus::Hint(hint_state)
    } else {
        GameStatus::InProgress
    };
    let status_line_vm = StatusLineViewModel::new(status);
    let toolbar_vm = build_toolbar_vm(app_state, ui_state);

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

#[must_use]
pub(crate) fn build_settings_view_model(app_state: &AppState) -> SettingsViewModel<'_> {
    let settings = &app_state.settings;
    SettingsViewModel::new(settings)
}

#[cfg(test)]
mod tests {
    use numelace_core::{Digit, DigitGrid, DigitPositions, DigitSet, Position};
    use numelace_game::{CellState, Game};
    use numelace_solver::{BoxedTechniqueStep, TechniqueApplication, TechniqueStep};

    use super::build_grid;
    use crate::{
        state::{AppState, GhostType, HintStage, HintState, UiState},
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

    #[derive(Debug, Clone)]
    struct HintTestStep {
        positions: DigitPositions,
    }

    impl TechniqueStep for HintTestStep {
        fn technique_name(&self) -> &'static str {
            "HintTest"
        }

        fn clone_box(&self) -> BoxedTechniqueStep {
            Box::new(self.clone())
        }

        fn condition_cells(&self) -> DigitPositions {
            self.positions
        }

        fn condition_digit_cells(&self) -> Vec<(DigitPositions, DigitSet)> {
            Vec::new()
        }

        fn application(&self) -> Vec<TechniqueApplication> {
            Vec::new()
        }
    }

    #[test]
    fn build_grid_highlights_selected_conflict_and_same_digit() {
        let mut app_state = AppState::new(game_from_filled(&filled_with_conflict()));
        app_state.selected_cell = Some(Position::new(0, 0));
        let ui_state = UiState::new();

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
        let mut ui_state = UiState::new();
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

    #[test]
    fn build_grid_highlights_hint_cells() {
        let app_state = AppState::new(game_from_filled(&blank_grid()));
        let mut ui_state = UiState::new();
        let positions = DigitPositions::from_elem(Position::new(2, 2));
        let step: BoxedTechniqueStep = Box::new(HintTestStep { positions });
        ui_state.hint_state = Some(HintState {
            stage: HintStage::Stage1,
            step,
        });

        let grid = build_grid(&app_state, &ui_state);

        assert!(
            grid[Position::new(2, 2)]
                .visual_state
                .contains(GridVisualState::HINT_CONDITION_CELL)
        );
        assert!(
            !grid[Position::new(3, 3)]
                .visual_state
                .contains(GridVisualState::HINT_CONDITION_CELL)
        );
    }
}
