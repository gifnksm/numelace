pub(crate) use self::{hint::*, new_game::*, solvability::*};
use crate::{
    action::{BoardMutationAction, ConfirmKind},
    flow::{FlowExecutor, FlowHandle, helpers},
};

mod hint;
mod new_game;
mod solvability;

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
