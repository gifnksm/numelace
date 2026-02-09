use numelace_game::Game;
use numelace_solver::{SolverError, TechniqueSolver, technique::BoxedTechniqueStep};

use crate::{
    action::{
        AlertKind, BoardMutationAction, ConfirmKind, HistoryAction, NotesFillScope, UiAction,
    },
    flow::{FlowExecutor, FlowHandle, helpers},
    state::{HintStage, HintState},
};

struct HintRequest {
    game: Game,
    hint_state: Option<HintState>,
}

/// Spawn a hint flow if no other flows are active.
pub(crate) fn spawn_hint_flow(
    executor: &mut FlowExecutor,
    game: &Game,
    hint_state: Option<HintState>,
) {
    if !executor.is_idle() {
        return;
    }
    let handle = executor.handle();
    let request = HintRequest {
        game: game.clone(),
        hint_state,
    };
    executor.spawn(hint_flow(handle, request));
}

async fn hint_flow(handle: FlowHandle, request: HintRequest) {
    if request.hint_state.is_some() {
        return;
    }

    let result = find_hint_step(&request.game);

    match result {
        Ok(Some((true, step))) => {
            let hint_state = HintState {
                stage: HintStage::Stage1,
                step,
            };
            handle.request_action(UiAction::SetHintState(Some(hint_state)).into());
        }
        Ok(Some((false, _step))) => {
            let result =
                helpers::show_confirm_dialog(&handle, ConfirmKind::HintNotesMaybeIncorrect).await;
            if result.is_confirmed() {
                handle.request_action(
                    BoardMutationAction::AutoFillNotes {
                        scope: NotesFillScope::AllCells,
                    }
                    .into(),
                );
            }
            handle.request_action(UiAction::SetHintState(None).into());
        }
        Ok(None) => {
            handle.request_action(UiAction::ClearHintState.into());
            let _ = helpers::show_alert_dialog(&handle, AlertKind::HintStuckNoStep).await;
        }
        Err(SolverError::Inconsistent(_)) => {
            let result = helpers::show_confirm_dialog(&handle, ConfirmKind::HintInconsistent).await;
            if result.is_confirmed() {
                handle_hint_undo(&handle).await;
            }
        }
    }
}

fn find_hint_step(game: &Game) -> Result<Option<(bool, BoxedTechniqueStep)>, SolverError> {
    let solver = TechniqueSolver::with_all_techniques();
    let grid_with_notes = game.to_candidate_grid_with_notes();
    match solver.find_step(&grid_with_notes) {
        Ok(Some(step)) if game.verify_hint_step(step.as_ref()) => {
            return Ok(Some((true, step)));
        }
        Ok(_) | Err(SolverError::Inconsistent(_)) => {}
    }

    let grid = game.to_candidate_grid();
    let step = solver.find_step(&grid)?;
    if let Some(step) = step
        && game.verify_hint_step(step.as_ref())
    {
        return Ok(Some((false, step)));
    }

    Ok(None)
}

async fn handle_hint_undo(handle: &FlowHandle) {
    let Some(games) = helpers::request_undo_games(handle).await else {
        return;
    };
    if games.is_empty() {
        return;
    }

    let outcome = scan_hint_rollback(&games);
    apply_hint_rollback_result(handle, outcome).await;
}

enum HintRollbackOutcome {
    Step {
        index: usize,
        step: BoxedTechniqueStep,
    },
    Stuck {
        index: usize,
    },
    NotFound,
}

fn scan_hint_rollback(games: &[Game]) -> HintRollbackOutcome {
    let mut first_consistent_index = None;

    for (index, game) in games.iter().enumerate() {
        match find_hint_step(game) {
            Ok(Some((true, step))) => return HintRollbackOutcome::Step { index, step },
            Ok(Some((false, _)) | None) => {
                if first_consistent_index.is_none() {
                    first_consistent_index = Some(index);
                }
            }
            Err(SolverError::Inconsistent(_)) => {}
        }
    }

    match first_consistent_index {
        Some(index) => HintRollbackOutcome::Stuck { index },
        None => HintRollbackOutcome::NotFound,
    }
}

async fn apply_hint_rollback_result(handle: &FlowHandle, outcome: HintRollbackOutcome) {
    match outcome {
        HintRollbackOutcome::Step { index, step } => {
            handle.request_action(HistoryAction::UndoSteps(index).into());

            if index > 0 {
                let _ =
                    helpers::show_alert_dialog(handle, AlertKind::HintUndoNotice { steps: index })
                        .await;
            }

            let hint_state = HintState {
                stage: HintStage::Stage1,
                step,
            };
            handle.request_action(UiAction::SetHintState(Some(hint_state)).into());
        }
        HintRollbackOutcome::Stuck { index } => {
            handle.request_action(HistoryAction::UndoSteps(index).into());

            if index > 0 {
                let _ =
                    helpers::show_alert_dialog(handle, AlertKind::HintUndoNotice { steps: index })
                        .await;
            }

            let _ = helpers::show_alert_dialog(handle, AlertKind::HintStuckAfterRollback).await;
        }
        HintRollbackOutcome::NotFound => {
            let _ = helpers::show_alert_dialog(handle, AlertKind::SolvabilityUndoNotFound).await;
        }
    }
}
