use numelace_generator::GeneratedPuzzle;

use crate::{
    action::{BoardMutationAction, ConfirmKind, PuzzleLifecycleAction, SpinnerKind},
    flow::{FlowExecutor, FlowHandle, helpers},
    worker,
};

pub(crate) use self::{hint::*, solvability::*};

mod hint;
mod solvability;

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
    let result = helpers::show_confirm_dialog(&handle, ConfirmKind::NewGame).await;
    if !result.is_confirmed() {
        return;
    }
    let work = worker::request_generate_puzzle();
    let response = helpers::with_spinner(&handle, SpinnerKind::NewGame, work).await;
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
    let result = helpers::show_confirm_dialog(&handle, ConfirmKind::ResetInputs).await;
    if !result.is_confirmed() {
        return;
    }
    handle.request_action(BoardMutationAction::ResetInputs.into());
}
