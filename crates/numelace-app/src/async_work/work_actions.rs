//! Centralized async work logic used by the action handler.

use numelace_core::DigitGrid;
use numelace_game::Game;

use crate::state::{AppState, ModalKind, SolvabilityState, SolvabilityStats, UiState};

use super::{
    WorkError, WorkRequest, WorkResponse, enqueue,
    new_game_dto::NewGameDto,
    solvability_dto::{SolvabilityGridDto, SolvabilityRequestDto, SolvabilityStateDto},
    work_flow::WorkFlow,
};

/// Request a new game using the async pipeline and panic on failure.
pub fn request_new_game(ui_state: &mut UiState) -> Result<(), WorkError> {
    if WorkFlow::is_work_in_flight(&ui_state.work) {
        return Ok(());
    }

    ui_state.work.last_error = None;

    match enqueue(WorkRequest::GenerateNewGame) {
        Ok(handle) => {
            WorkFlow::start_new_game(&mut ui_state.work, handle);
            Ok(())
        }
        Err(err) => {
            WorkFlow::record_error(&mut ui_state.work, err.clone());
            panic!("background work failed: {err}");
        }
    }
}

/// Request a solvability check using the async pipeline and panic on failure.
pub fn request_check_solvability(game: &Game, ui_state: &mut UiState) -> Result<(), WorkError> {
    if WorkFlow::is_work_in_flight(&ui_state.work) {
        return Ok(());
    }

    ui_state.work.last_error = None;

    let request = SolvabilityRequestDto {
        with_user_notes: SolvabilityGridDto::from_candidate_grid(
            &game.to_candidate_grid_with_notes(),
        ),
        without_user_notes: SolvabilityGridDto::from_candidate_grid(&game.to_candidate_grid()),
    };

    let work_request = WorkRequest::CheckSolvability(request);

    match enqueue(work_request.clone()) {
        Ok(handle) => {
            WorkFlow::start_request(&mut ui_state.work, &work_request, handle);
            Ok(())
        }
        Err(err) => {
            WorkFlow::record_error(&mut ui_state.work, err.clone());
            panic!("background work failed: {err}");
        }
    }
}

/// Apply a completed background response, updating app state.
pub fn apply_work_response(
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
        WorkResponse::SolvabilityReady(result) => {
            let state = map_solvability_state(result);
            ui_state.active_modal = Some(ModalKind::CheckSolvabilityResult(state));
        }
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

fn map_solvability_state(result: SolvabilityStateDto) -> SolvabilityState {
    match result {
        SolvabilityStateDto::Inconsistent => SolvabilityState::Inconsistent,
        SolvabilityStateDto::NoSolution => SolvabilityState::NoSolution,
        SolvabilityStateDto::Solvable {
            with_user_notes,
            stats,
        } => SolvabilityState::Solvable {
            with_user_notes,
            stats: SolvabilityStats {
                assumptions_len: stats.assumptions_len,
                backtrack_count: stats.backtrack_count,
                solved_without_assumptions: stats.solved_without_assumptions,
            },
        },
    }
}
