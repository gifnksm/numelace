use futures_channel::oneshot;
use numelace_game::Game;
use numelace_generator::GeneratedPuzzle;

use crate::{
    action::{ConfirmKind, ModalRequest, PuzzleLifecycleAction, SpinnerKind, UiAction},
    flow::{FlowExecutor, FlowHandle, helpers},
    state::NewGameOptions,
    worker,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
enum GameState {
    Uninitialized,
    InProgress,
    Solved,
}

/// Spawn a new game flow if no other flows are active.
pub(crate) fn spawn_new_game_flow(executor: &mut FlowExecutor, game: &Game) {
    if !executor.is_idle() {
        return;
    }
    let game_state = if !game.is_initialized() {
        GameState::Uninitialized
    } else if game.is_solved() {
        GameState::Solved
    } else {
        GameState::InProgress
    };
    let handle = executor.handle();
    executor.spawn(new_game_flow(handle, game_state));
}

/// Async flow for new game confirmation + work dispatch.
///
/// On confirm, it runs the background request and awaits the response.
async fn new_game_flow(handle: FlowHandle, game_state: GameState) {
    if game_state.is_in_progress() {
        let result = helpers::show_confirm_dialog(&handle, ConfirmKind::NewGame).await;
        if !result.is_confirmed() {
            return;
        }
    }

    let can_cancel = !game_state.is_uninitialized();
    let Some(options) = show_new_game_options_modal(&handle, can_cancel).await else {
        return;
    };

    let work = worker::request_generate_puzzle(options.into());
    let response = helpers::with_spinner(&handle, SpinnerKind::NewGame, work).await;
    let dto = response.unwrap();
    let puzzle = GeneratedPuzzle::try_from(dto)
        .unwrap_or_else(|err| panic!("failed to deserialize generated puzzle dto: {err}"));
    handle.request_action(PuzzleLifecycleAction::StartNewGame(puzzle).into());
}

async fn show_new_game_options_modal(
    handle: &FlowHandle,
    can_cancel: bool,
) -> Option<NewGameOptions> {
    let (responder, receiver) = oneshot::channel();
    handle.request_action(
        UiAction::OpenModal(ModalRequest::NewGameOptions {
            can_cancel,
            responder: Some(responder),
        })
        .into(),
    );
    receiver.await.unwrap_or_default()
}
