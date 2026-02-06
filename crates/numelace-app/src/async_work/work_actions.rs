//! Centralized async work logic used by the action handler.

use numelace_core::DigitGrid;
use numelace_game::Game;

use crate::state::{AppState, UiState};

use super::{
    WorkError, WorkRequest, WorkResponse, enqueue,
    new_game_dto::NewGameDto,
    solvability_dto::{SolvabilityGridDto, SolvabilityRequestDto},
    work_flow::WorkFlow,
};

/// Request background work using the async pipeline and panic on failure.
#[expect(clippy::unnecessary_wraps, clippy::needless_pass_by_value)]
pub(crate) fn request_work(request: WorkRequest, ui_state: &mut UiState) -> Result<(), WorkError> {
    if WorkFlow::is_work_in_flight(&ui_state.work) {
        return Ok(());
    }

    ui_state.work.last_error = None;

    match enqueue(request.clone()) {
        Ok(handle) => {
            WorkFlow::start_request(&mut ui_state.work, &request, handle);
            Ok(())
        }
        Err(err) => {
            WorkFlow::record_error(&mut ui_state.work, err.clone());
            panic!("background work failed: {err}");
        }
    }
}

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
    app_state: &mut AppState,
    ui_state: &mut UiState,
    response: WorkResponse,
) {
    WorkFlow::finish_response(&mut ui_state.work, &response);

    match response {
        WorkResponse::NewGameReady(dto) => {
            if let Err(err) = apply_new_game_dto(app_state, ui_state, &dto) {
                ui_state.work.last_error = Some(err.clone());
                panic!("failed to apply new game response: {err}");
            }
        }
        WorkResponse::SolvabilityReady(_result) => {}
        WorkResponse::Error(err) => {
            ui_state.work.last_error = Some(err.clone());
            panic!("background work failed: {err}");
        }
    }
}

fn apply_new_game_dto(
    app_state: &mut AppState,
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

    app_state.game = game;
    app_state.selected_cell = None;
    app_state.apply_new_game_settings();
    ui_state.reset_history(app_state);

    Ok(())
}
