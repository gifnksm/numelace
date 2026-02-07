use numelace_game::Game;

use crate::{
    action::{
        AlertKind, BoardMutationAction, ConfirmKind, HistoryAction, NotesFillScope, SpinnerKind,
    },
    flow::{FlowExecutor, FlowHandle, helpers},
    state::SolvabilityState,
    worker::{
        self,
        tasks::{CandidateGridPairDto, SolvabilityUndoScanResultDto},
    },
};

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
    let state = helpers::with_spinner(&handle, SpinnerKind::CheckSolvability, work)
        .await
        .unwrap();
    let state = state.into();

    match state {
        SolvabilityState::Inconsistent => {
            let result =
                helpers::show_confirm_dialog(&handle, ConfirmKind::SolvabilityInconsistent).await;
            if result.is_confirmed() {
                handle_solvability_undo(&handle).await;
            }
        }
        SolvabilityState::NoSolution => {
            let result =
                helpers::show_confirm_dialog(&handle, ConfirmKind::SolvabilityNoSolution).await;
            if result.is_confirmed() {
                handle_solvability_undo(&handle).await;
            }
        }
        SolvabilityState::Solvable {
            with_user_notes: true,
            stats: _stats,
        } => {
            let _ = helpers::show_alert_dialog(&handle, AlertKind::SolvabilitySolvable).await;
        }
        SolvabilityState::Solvable {
            with_user_notes: false,
            stats: _stats,
        } => {
            let result =
                helpers::show_confirm_dialog(&handle, ConfirmKind::SolvabilityNotesMaybeIncorrect)
                    .await;
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

async fn handle_solvability_undo(handle: &FlowHandle) {
    let Some(games) = helpers::request_undo_games(handle).await else {
        return;
    };
    if games.is_empty() {
        return;
    }

    let work = worker::request_solvability_undo_scan(games.into());
    let result = helpers::with_spinner(handle, SpinnerKind::CheckSolvability, work)
        .await
        .unwrap();
    apply_solvability_undo_result(handle, result).await;
}

async fn apply_solvability_undo_result(handle: &FlowHandle, result: SolvabilityUndoScanResultDto) {
    let Some(index) = result.index else {
        let _ = helpers::show_alert_dialog(handle, AlertKind::SolvabilityUndoNotFound).await;
        return;
    };

    handle.request_action(HistoryAction::UndoSteps(index).into());

    if index > 0 {
        let _ =
            helpers::show_alert_dialog(handle, AlertKind::SolvabilityUndoNotice { steps: index })
                .await;
    }

    let state = result.state.into();
    if matches!(
        state,
        SolvabilityState::Solvable {
            with_user_notes: false,
            stats: _,
        }
    ) {
        let result =
            helpers::show_confirm_dialog(handle, ConfirmKind::SolvabilityNotesMaybeIncorrect).await;
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
