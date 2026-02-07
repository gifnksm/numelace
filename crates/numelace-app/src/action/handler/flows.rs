use futures_channel::oneshot;
use numelace_game::Game;
use numelace_generator::GeneratedPuzzle;

use crate::{
    action::{
        BoardMutationAction, ConfirmKind, ConfirmResponder, ConfirmResult, HistoryAction,
        ModalRequest, NotesFillScope, PuzzleLifecycleAction, SolvabilityDialogResult,
        SolvabilityResponder, SolvabilityUndoGridsResponder, SolvabilityUndoNoticeResponder,
        SolvabilityUndoNoticeResult, SpinnerKind, StateQueryAction, UiAction, flows,
    },
    flow_executor::{FlowExecutor, FlowHandle},
    state::{SolvabilityState, SolvabilityStats},
    worker::{
        self,
        tasks::{
            SolvabilityRequestDto, SolvabilityStateDto, SolvabilityUndoGridsDto,
            SolvabilityUndoScanResultDto,
        },
    },
};

async fn show_confirm_dialog(handle: &FlowHandle, kind: ConfirmKind) -> ConfirmResult {
    let (responder, receiver): (ConfirmResponder, oneshot::Receiver<ConfirmResult>) =
        oneshot::channel();
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
    match result {
        ConfirmResult::Cancelled => return,
        ConfirmResult::Confirmed => {}
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
    match result {
        ConfirmResult::Cancelled => return,
        ConfirmResult::Confirmed => {}
    }
    handle.request_action(BoardMutationAction::ResetInputs.into());
}

/// Spawn a solvability check flow if no other flows are active.
pub(crate) fn spawn_check_solvability_flow(executor: &mut FlowExecutor, game: &Game) {
    if !executor.is_idle() {
        return;
    }
    let handle = executor.handle();
    let request = build_solvability_request(game);
    executor.spawn(check_solvability_flow(handle, request));
}

/// Async flow for solvability check work dispatch.
///
/// Runs the background request and awaits the response.
async fn check_solvability_flow(handle: FlowHandle, request: SolvabilityRequestDto) {
    let work = worker::request_solvability(request);
    let state = flows::with_spinner(&handle, SpinnerKind::CheckSolvability, work)
        .await
        .unwrap();
    let state = map_solvability_state(state);
    let dialog_result = await_solvability_dialog(&handle, state).await;

    match dialog_result {
        SolvabilityDialogResult::RebuildNotes => {
            handle.request_action(
                BoardMutationAction::AutoFillNotes {
                    scope: NotesFillScope::AllCells,
                }
                .into(),
            );
        }
        SolvabilityDialogResult::Undo => {
            handle_solvability_undo(&handle).await;
        }
        SolvabilityDialogResult::Close => {}
    }
}

/// Await the solvability result dialog.
async fn await_solvability_dialog(
    handle: &FlowHandle,
    state: SolvabilityState,
) -> SolvabilityDialogResult {
    let (responder, receiver): (
        SolvabilityResponder,
        oneshot::Receiver<SolvabilityDialogResult>,
    ) = oneshot::channel();
    handle.request_action(
        UiAction::OpenModal(ModalRequest::CheckSolvabilityResult {
            state,
            responder: Some(responder),
        })
        .into(),
    );

    match receiver.await {
        Ok(result) => result,
        Err(_) => SolvabilityDialogResult::Close,
    }
}

async fn await_solvability_undo_notice(
    handle: &FlowHandle,
    steps: usize,
) -> SolvabilityUndoNoticeResult {
    let (responder, receiver): (
        SolvabilityUndoNoticeResponder,
        oneshot::Receiver<SolvabilityUndoNoticeResult>,
    ) = oneshot::channel();
    handle.request_action(
        UiAction::OpenModal(ModalRequest::SolvabilityUndoNotice {
            steps,
            responder: Some(responder),
        })
        .into(),
    );

    match receiver.await {
        Ok(result) => result,
        Err(_) => SolvabilityUndoNoticeResult::Close,
    }
}

async fn request_solvability_undo_grids(handle: &FlowHandle) -> Option<SolvabilityUndoGridsDto> {
    let (responder, receiver): (
        SolvabilityUndoGridsResponder,
        oneshot::Receiver<SolvabilityUndoGridsDto>,
    ) = oneshot::channel();
    handle.request_action(StateQueryAction::BuildSolvabilityUndoGrids { responder }.into());

    receiver.await.ok()
}

async fn handle_solvability_undo(handle: &FlowHandle) {
    let Some(grids) = request_solvability_undo_grids(handle).await else {
        return;
    };
    if grids.grids.is_empty() {
        return;
    }

    let work = worker::request_solvability_undo_scan(grids);
    let result = flows::with_spinner(handle, SpinnerKind::CheckSolvability, work)
        .await
        .unwrap();
    apply_solvability_undo_result(handle, result).await;
}

async fn apply_solvability_undo_result(handle: &FlowHandle, result: SolvabilityUndoScanResultDto) {
    let Some(index) = result.index else {
        return;
    };

    handle.request_action(HistoryAction::UndoSteps(index).into());

    if index > 0 {
        let _ = await_solvability_undo_notice(handle, index).await;
    }

    let state = map_solvability_state(result.state);
    if matches!(
        state,
        SolvabilityState::Solvable {
            with_user_notes: false,
            stats: _,
        }
    ) {
        let dialog_result = await_solvability_dialog(handle, state).await;
        if matches!(dialog_result, SolvabilityDialogResult::RebuildNotes) {
            handle.request_action(
                BoardMutationAction::AutoFillNotes {
                    scope: NotesFillScope::AllCells,
                }
                .into(),
            );
        }
    }
}

fn build_solvability_request(game: &Game) -> SolvabilityRequestDto {
    game.to_candidate_grid_with_notes().into()
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
