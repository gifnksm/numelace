//! Centralized async work logic used by the app loop and action handling.

use numelace_core::DigitGrid;
use numelace_game::Game;

use crate::state::{AppStateAccess, UiState};

use super::{
    WorkError, WorkRequest, WorkResponse,
    new_game_dto::NewGameDto,
    solvability_dto::{SolvabilityGridDto, SolvabilityRequestDto},
};

/// Build a solvability request for background work.
pub(crate) fn build_solvability_request(game: &Game) -> WorkRequest {
    let request = SolvabilityRequestDto {
        with_user_notes: SolvabilityGridDto::from_candidate_grid(
            &game.to_candidate_grid_with_notes(),
        ),
        without_user_notes: SolvabilityGridDto::from_candidate_grid(&game.to_candidate_grid()),
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
            if let Err(err) = apply_new_game_dto(app_state, ui_state, &dto) {
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
    dto: &NewGameDto,
) -> Result<(), WorkError> {
    let problem: DigitGrid = dto
        .problem
        .parse()
        .map_err(|_| WorkError::DeserializationFailed)?;
    let solution: DigitGrid = dto
        .solution
        .parse()
        .map_err(|_| WorkError::DeserializationFailed)?;

    let filled = DigitGrid::new();
    let notes = [[0u16; 9]; 9];
    let game = Game::from_problem_filled_notes(&problem, &solution, &filled, &notes)
        .map_err(|_| WorkError::DeserializationFailed)?;

    let app_state = app_state.as_mut();
    app_state.game = game;
    app_state.selected_cell = None;
    app_state.apply_new_game_settings();
    ui_state.reset_history(app_state);

    Ok(())
}
