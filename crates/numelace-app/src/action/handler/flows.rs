use futures_channel::oneshot;
use numelace_game::Game;
use numelace_generator::GeneratedPuzzle;

use crate::{
    action::{
        AlertKind, AlertResult, BoardMutationAction, ConfirmKind, ConfirmResult, HistoryAction,
        ModalRequest, NotesFillScope, PuzzleLifecycleAction, SpinnerKind, StateQueryAction,
        UiAction, flows,
    },
    flow_executor::{FlowExecutor, FlowHandle},
    state::SolvabilityState,
    worker::{
        self,
        tasks::{CandidateGridPairDto, SolvabilityUndoScanResultDto},
    },
};

async fn show_confirm_dialog(handle: &FlowHandle, kind: ConfirmKind) -> ConfirmResult {
    let (responder, receiver) = oneshot::channel();
    handle.request_action(
        UiAction::OpenModal(ModalRequest::Confirm {
            kind,
            responder: Some(responder),
        })
        .into(),
    );
    match receiver.await {
        Ok(result) => result,
        Err(_) => ConfirmResult::Cancelled,
    }
}

async fn show_alert_dialog(handle: &FlowHandle, kind: AlertKind) -> AlertResult {
    let (responder, receiver) = oneshot::channel();
    handle.request_action(
        UiAction::OpenModal(ModalRequest::Alert {
            kind,
            responder: Some(responder),
        })
        .into(),
    );
    match receiver.await {
        Ok(result) => result,
        Err(_) => AlertResult::Ok,
    }
}

/// Spawn a new game flow if no other flows are active.
pub(crate) fn spawn_new_game_flow(executor: &mut FlowExecutor) {
    if !executor.is_idle() {
        return;
    }
    let handle = executor.handle();
    executor.spawn(new_game_flow(handle));
}

/// Async flow for new game confirmation + work dispatch.
///
/// On confirm, it runs the background request and awaits the response.
async fn new_game_flow(handle: FlowHandle) {
    let result = show_confirm_dialog(&handle, ConfirmKind::NewGame).await;
    if !result.is_confirmed() {
        return;
    }
    let work = worker::request_generate_puzzle();
    let response = flows::with_spinner(&handle, SpinnerKind::NewGame, work).await;
    let dto = response.unwrap();
    let puzzle = GeneratedPuzzle::try_from(dto)
        .unwrap_or_else(|err| panic!("failed to deserialize generated puzzle dto: {err}"));
    handle.request_action(PuzzleLifecycleAction::StartNewGame(puzzle).into());
}

pub(crate) fn spawn_reset_inputs_flow(executor: &mut FlowExecutor) {
    if !executor.is_idle() {
        return;
    }
    let handle = executor.handle();
    executor.spawn(reset_inputs_flow(handle));
}

async fn reset_inputs_flow(handle: FlowHandle) {
    let result = show_confirm_dialog(&handle, ConfirmKind::ResetInputs).await;
    if !result.is_confirmed() {
        return;
    }
    handle.request_action(BoardMutationAction::ResetInputs.into());
}

/// Spawn a solvability check flow if no other flows are active.
pub(crate) fn spawn_check_solvability_flow(executor: &mut FlowExecutor, game: &Game) {
    if !executor.is_idle() {
        return;
    }
    let handle = executor.handle();
    let request = game.into();
    executor.spawn(check_solvability_flow(handle, request));
}

/// Async flow for solvability check work dispatch.
///
/// Runs the background request and awaits the response.
async fn check_solvability_flow(handle: FlowHandle, request: CandidateGridPairDto) {
    let work = worker::request_solvability(request);
    let state = flows::with_spinner(&handle, SpinnerKind::CheckSolvability, work)
        .await
        .unwrap();
    let state = state.into();

    match state {
        SolvabilityState::Inconsistent => {
            let result = show_confirm_dialog(&handle, ConfirmKind::SolvabilityInconsistent).await;
            if result.is_confirmed() {
                handle_solvability_undo(&handle).await;
            }
        }
        SolvabilityState::NoSolution => {
            let result = show_confirm_dialog(&handle, ConfirmKind::SolvabilityNoSolution).await;
            if result.is_confirmed() {
                handle_solvability_undo(&handle).await;
            }
        }
        SolvabilityState::Solvable {
            with_user_notes: true,
            stats: _stats,
        } => {
            let _ = show_alert_dialog(&handle, AlertKind::SolvabilitySolvable).await;
        }
        SolvabilityState::Solvable {
            with_user_notes: false,
            stats: _stats,
        } => {
            let result =
                show_confirm_dialog(&handle, ConfirmKind::SolvabilityNotesMaybeIncorrect).await;
            if result.is_confirmed() {
                handle.request_action(
                    BoardMutationAction::AutoFillNotes {
                        scope: NotesFillScope::AllCells,
                    }
                    .into(),
                );
            }
        }
    }
}

fn open_solvability_undo_not_found(handle: &FlowHandle) {
    handle.request_action(
        UiAction::OpenModal(ModalRequest::Alert {
            kind: AlertKind::SolvabilityUndoNotFound,
            responder: None,
        })
        .into(),
    );
}

async fn request_undo_games(handle: &FlowHandle) -> Option<Vec<Game>> {
    let (responder, receiver) = oneshot::channel();
    handle.request_action(StateQueryAction::BuildUndoGames { responder }.into());
    receiver.await.ok()
}

async fn handle_solvability_undo(handle: &FlowHandle) {
    let Some(games) = request_undo_games(handle).await else {
        return;
    };
    if games.is_empty() {
        return;
    }

    let work = worker::request_solvability_undo_scan(games.into());
    let result = flows::with_spinner(handle, SpinnerKind::CheckSolvability, work)
        .await
        .unwrap();
    apply_solvability_undo_result(handle, result).await;
}

async fn apply_solvability_undo_result(handle: &FlowHandle, result: SolvabilityUndoScanResultDto) {
    let Some(index) = result.index else {
        open_solvability_undo_not_found(handle);
        return;
    };

    handle.request_action(HistoryAction::UndoSteps(index).into());

    if index > 0 {
        let _ = show_alert_dialog(handle, AlertKind::SolvabilityUndoNotice { steps: index }).await;
    }

    let state = result.state.into();
    if matches!(
        state,
        SolvabilityState::Solvable {
            with_user_notes: false,
            stats: _,
        }
    ) {
        let result = show_confirm_dialog(handle, ConfirmKind::SolvabilityNotesMaybeIncorrect).await;
        if result.is_confirmed() {
            handle.request_action(
                BoardMutationAction::AutoFillNotes {
                    scope: NotesFillScope::AllCells,
                }
                .into(),
            );
        }
    }
}
