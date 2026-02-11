use numelace_core::{ConsistencyError, Position};
use numelace_game::{CellState, Game};
use numelace_solver::{
    SolverError, TechniqueSolver,
    technique::{BoxedTechniqueStep, NakedSingle, TechniqueGrid},
};

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

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
enum HintStepError {
    #[display("inconsistency detected: {_0}")]
    Inconsistent(#[from] ConsistencyError),
    #[display("hint step conflicts with solution")]
    SolutionMismatch,
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
    match request.hint_state {
        None
        | Some(HintState {
            stage: HintStage::Stage3Apply,
            ..
        }) => {
            let result = find_hint_step(&request.game);

            match result {
                Ok(Some((true, step))) => {
                    let hint_state = HintState {
                        stage: HintStage::Stage1,
                        step,
                    };
                    handle.request_action(UiAction::SetHintState(Some(hint_state)).into());
                }
                Ok(Some((false, _step))) => handle_hint_notes_maybe_incorrect(&handle).await,
                Ok(None) => {
                    handle.request_action(UiAction::ClearHintState.into());
                    let _ = helpers::show_alert_dialog(&handle, AlertKind::HintStuckNoStep).await;
                }
                Err(HintStepError::Inconsistent(_) | HintStepError::SolutionMismatch) => {
                    let result =
                        helpers::show_confirm_dialog(&handle, ConfirmKind::HintInconsistent).await;
                    if result.is_confirmed() {
                        handle_hint_undo(&handle).await;
                    }
                }
            }
        }
        Some(mut hint_state) => match hint_state.stage {
            HintStage::Stage1 => {
                hint_state.stage = HintStage::Stage2;
                handle.request_action(UiAction::SetHintState(Some(hint_state)).into());
            }
            HintStage::Stage2 => {
                hint_state.stage = HintStage::Stage3Preview;
                handle.request_action(UiAction::SetHintState(Some(hint_state)).into());
            }
            HintStage::Stage3Preview => {
                handle.request_action(
                    BoardMutationAction::ApplyTechniqueStep(hint_state.step.clone()).into(),
                );
                hint_state.stage = HintStage::Stage3Apply;
                handle.request_action(UiAction::SetHintState(Some(hint_state)).into());
            }
            HintStage::Stage3Apply => unreachable!(),
        },
    }
}

async fn handle_hint_notes_maybe_incorrect(handle: &FlowHandle) {
    let result = helpers::show_confirm_dialog(handle, ConfirmKind::HintNotesMaybeIncorrect).await;
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

fn find_naked_single_hint(game: &Game, grid: &TechniqueGrid) -> Option<BoxedTechniqueStep> {
    // Naked single hints must consider placement validity even when no eliminations occur.
    // The solver's NakedSingle::find_step intentionally gates on eliminations, which can
    // skip valid placements once peers already lack that candidate.
    for pos in Position::ALL {
        match game.cell(pos) {
            CellState::Empty | CellState::Notes(_) => {
                // Empty/notes cells are valid hint targets.
            }
            CellState::Given(_) | CellState::Filled(_) => continue,
        }

        let Some(step) = NakedSingle::build_step(grid, pos) else {
            continue;
        };

        return Some(step);
    }

    None
}

fn find_hint_step_from_grid(
    game: &Game,
    grid: &TechniqueGrid,
    solver: &TechniqueSolver,
) -> Result<Option<BoxedTechniqueStep>, HintStepError> {
    grid.check_consistency()?;

    if let Some(step) = find_naked_single_hint(game, grid) {
        if game.verify_hint_step(step.as_ref()) {
            return Ok(Some(step));
        }
        return Err(HintStepError::SolutionMismatch);
    }

    let step = solver.find_step(grid).map_err(|err| match err {
        SolverError::Inconsistent(consistency) => HintStepError::Inconsistent(consistency),
    })?;

    match step {
        Some(step) => {
            if game.verify_hint_step(step.as_ref()) {
                return Ok(Some(step));
            }
            Err(HintStepError::SolutionMismatch)
        }
        None => Ok(None),
    }
}

fn find_hint_step(game: &Game) -> Result<Option<(bool, BoxedTechniqueStep)>, HintStepError> {
    let solver = TechniqueSolver::with_all_techniques();
    let grid_with_notes = TechniqueGrid::from(game.to_candidate_grid_with_notes());

    // Notes-derived grids can be stale; treat inconsistency or solution mismatch as a signal
    // to fall back to the no-notes grid before surfacing an error.
    match find_hint_step_from_grid(game, &grid_with_notes, &solver) {
        Ok(Some(step_with_notes)) => return Ok(Some((true, step_with_notes))),
        Ok(None) | Err(HintStepError::Inconsistent(_) | HintStepError::SolutionMismatch) => {}
    }

    let grid = TechniqueGrid::from(game.to_candidate_grid());

    if let Some(step) = find_hint_step_from_grid(game, &grid, &solver)? {
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
    FoundWithNotes {
        index: usize,
        step: BoxedTechniqueStep,
    },
    FoundWithoutNotes {
        index: usize,
    },
    StuckButConsistent {
        index: usize,
    },
    Inconsistent,
}

fn scan_hint_rollback(games: &[Game]) -> HintRollbackOutcome {
    let mut first_consistent_index = None;

    for (index, game) in games.iter().enumerate() {
        match find_hint_step(game) {
            Ok(Some((true, step))) => return HintRollbackOutcome::FoundWithNotes { index, step },
            Ok(Some((false, _))) => return HintRollbackOutcome::FoundWithoutNotes { index },
            Ok(None) => {
                if first_consistent_index.is_none() {
                    first_consistent_index = Some(index);
                }
            }
            Err(HintStepError::Inconsistent(_) | HintStepError::SolutionMismatch) => {}
        }
    }

    if let Some(index) = first_consistent_index {
        return HintRollbackOutcome::StuckButConsistent { index };
    }

    HintRollbackOutcome::Inconsistent
}

async fn apply_hint_rollback_result(handle: &FlowHandle, outcome: HintRollbackOutcome) {
    match outcome {
        HintRollbackOutcome::FoundWithNotes { index, step } => {
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
        HintRollbackOutcome::FoundWithoutNotes { index } => {
            handle.request_action(HistoryAction::UndoSteps(index).into());

            if index > 0 {
                let _ =
                    helpers::show_alert_dialog(handle, AlertKind::HintUndoNotice { steps: index })
                        .await;
            }

            handle_hint_notes_maybe_incorrect(handle).await;
        }
        HintRollbackOutcome::StuckButConsistent { index } => {
            handle.request_action(HistoryAction::UndoSteps(index).into());

            if index > 0 {
                let _ =
                    helpers::show_alert_dialog(handle, AlertKind::HintUndoNotice { steps: index })
                        .await;
            }

            let _ = helpers::show_alert_dialog(handle, AlertKind::HintStuckAfterRollback).await;
        }
        HintRollbackOutcome::Inconsistent => {
            let _ =
                helpers::show_alert_dialog(handle, AlertKind::HintInconsistentAfterRollback).await;
        }
    }
}
