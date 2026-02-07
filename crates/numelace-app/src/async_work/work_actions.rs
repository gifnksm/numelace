//! Centralized async work logic used by the app loop and action handling.

use numelace_game::Game;
use numelace_generator::GeneratedPuzzle;

use crate::state::{AppStateAccess, UiState};

use super::{
    WorkError, WorkRequest, WorkResponse, new_game_dto::NewGameDto,
    solvability_dto::SolvabilityRequestDto,
};

/// Build a solvability request for background work.
pub(crate) fn build_solvability_request(game: &Game) -> WorkRequest {
    let request = SolvabilityRequestDto {
        with_user_notes: game.to_candidate_grid_with_notes().into(),
        without_user_notes: game.to_candidate_grid().into(),
    };

    WorkRequest::CheckSolvability(request)
}

/// Apply a completed background response, updating app state.
pub(crate) fn apply_work_response(
    app_state: &mut AppStateAccess<'_>,
    ui_state: &mut UiState,
    response: WorkResponse,
) {
    match response {
        WorkResponse::NewGameReady(dto) => {
            if let Err(err) = apply_new_game_dto(app_state, ui_state, dto) {
                panic!("failed to apply new game response: {err}");
            }
        }
        WorkResponse::SolvabilityReady(_result) => {}
        WorkResponse::Error(err) => {
            panic!("background work failed: {err}");
        }
    }
}

fn apply_new_game_dto(
    app_state: &mut AppStateAccess<'_>,
    ui_state: &mut UiState,
    dto: NewGameDto,
) -> Result<(), WorkError> {
    let puzzle = GeneratedPuzzle::try_from(dto).map_err(|_| WorkError::DeserializationFailed)?;
    let game = Game::new(puzzle);

    let app_state = app_state.as_mut();
    app_state.game = game;
    app_state.selected_cell = None;
    app_state.apply_new_game_settings();
    ui_state.reset_history(app_state);

    Ok(())
}
