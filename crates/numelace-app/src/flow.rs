use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use futures_channel::oneshot;
use numelace_game::Game;
use numelace_generator::GeneratedPuzzle;

use crate::{
    action::{
        Action, ActionRequestQueue, ConfirmResponder, ConfirmResult, ModalRequest, NotesFillScope,
        SolvabilityDialogResult, SolvabilityResponder,
    },
    async_work::{
        self,
        solvability_dto::{SolvabilityRequestDto, SolvabilityStateDto},
    },
    state::{SolvabilityState, SolvabilityStats},
};

/// Lightweight async flow executor for UI orchestration.
///
/// This executor is polled from the app update loop and drives flow futures
/// that request UI actions and await UI events.
pub(crate) struct FlowExecutor {
    state: Rc<RefCell<FlowState>>,
    tasks: Vec<FlowTask>,
}

impl std::fmt::Debug for FlowExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlowExecutor")
            .field("tasks", &self.tasks.len())
            .finish_non_exhaustive()
    }
}

impl Default for FlowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowExecutor {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(FlowState::default())),
            tasks: Vec::new(),
        }
    }

    /// Returns a handle for flows to request actions and await events.
    #[must_use]
    pub(crate) fn handle(&self) -> FlowHandle {
        FlowHandle {
            state: Rc::clone(&self.state),
        }
    }

    /// Returns the active spinner (if any) for flow-driven UI feedback.
    #[must_use]
    pub(crate) fn active_spinner(&self) -> Option<SpinnerKind> {
        self.state.borrow().active_spinner
    }

    /// Returns true if no flows are currently running.
    #[must_use]
    fn is_idle(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Spawn a new flow future.
    pub(crate) fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        self.tasks.push(FlowTask {
            future: Box::pin(future),
        });
    }

    /// Poll all active flows and drain any queued actions into the UI action queue.
    pub(crate) fn poll(&mut self, action_queue: &mut ActionRequestQueue) {
        self.drain_actions(action_queue);

        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        let mut i = 0;
        while i < self.tasks.len() {
            let task = &mut self.tasks[i];
            if task.future.as_mut().poll(&mut cx).is_ready() {
                self.tasks.swap_remove(i);
            } else {
                i += 1;
            }
        }

        self.drain_actions(action_queue);
    }

    fn drain_actions(&mut self, action_queue: &mut ActionRequestQueue) {
        let mut state = self.state.borrow_mut();
        for action in state.pending_actions.drain(..) {
            action_queue.request(action);
        }
    }
}

/// Flow handle used by async flows to request actions and await events.
#[derive(Clone)]
pub(crate) struct FlowHandle {
    state: Rc<RefCell<FlowState>>,
}

impl FlowHandle {
    fn request_action(&self, action: Action) {
        self.state.borrow_mut().pending_actions.push(action);
    }

    /// Await a new game confirmation dialog.
    #[must_use]
    pub(crate) async fn confirm_new_game(&self) -> ConfirmResult {
        let (responder, receiver): (ConfirmResponder, oneshot::Receiver<ConfirmResult>) =
            oneshot::channel();
        self.request_action(Action::OpenModal(ModalRequest::NewGameConfirm(Some(
            responder,
        ))));

        match receiver.await {
            Ok(result) => result,
            Err(_) => ConfirmResult::Cancelled,
        }
    }

    /// Await the solvability result dialog.
    #[must_use]
    pub(crate) async fn await_solvability_dialog(
        &self,
        state: SolvabilityState,
    ) -> SolvabilityDialogResult {
        let (responder, receiver): (
            SolvabilityResponder,
            oneshot::Receiver<SolvabilityDialogResult>,
        ) = oneshot::channel();
        self.request_action(Action::OpenModal(ModalRequest::CheckSolvabilityResult {
            state,
            responder: Some(responder),
        }));

        match receiver.await {
            Ok(result) => result,
            Err(_) => SolvabilityDialogResult::Close,
        }
    }

    /// Wrap a future with a flow-driven spinner.
    #[must_use]
    fn with_spinner<F>(&self, kind: SpinnerKind, future: F) -> WithSpinnerFuture<F>
    where
        F: Future,
    {
        WithSpinnerFuture {
            state: Rc::clone(&self.state),
            kind,
            started: false,
            inner: Box::pin(future),
        }
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
    let result = handle.confirm_new_game().await;
    if matches!(result, ConfirmResult::Confirmed) {
        let work = async_work::request_generate_puzzle();
        let response = handle.with_spinner(SpinnerKind::NewGame, work).await;
        let dto = match response {
            Ok(dto) => dto,
            Err(err) => {
                panic!("background work failed: {err}");
            }
        };
        let puzzle = GeneratedPuzzle::try_from(dto)
            .unwrap_or_else(|err| panic!("failed to deserialize generated puzzle dto: {err}"));
        handle.request_action(Action::NewGameReady(puzzle));
    }
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

fn build_solvability_request(game: &Game) -> SolvabilityRequestDto {
    SolvabilityRequestDto {
        with_user_notes: game.to_candidate_grid_with_notes().into(),
        without_user_notes: game.to_candidate_grid().into(),
    }
}

/// Async flow for solvability check work dispatch.
///
/// Runs the background request and awaits the response.
async fn check_solvability_flow(handle: FlowHandle, request: SolvabilityRequestDto) {
    let work = async_work::request_solvability(request);
    let response = handle
        .with_spinner(SpinnerKind::CheckSolvability, work)
        .await;

    let state = match response {
        Ok(state) => state,
        Err(err) => {
            panic!("background work failed: {err}");
        }
    };

    let state = map_solvability_state(state);
    let dialog_result = handle.await_solvability_dialog(state).await;

    if matches!(dialog_result, SolvabilityDialogResult::RebuildNotes) {
        handle.request_action(Action::AutoFillNotes {
            scope: NotesFillScope::AllCells,
        });
    }
}

struct FlowTask {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

#[derive(Default)]
struct FlowState {
    pending_actions: Vec<Action>,
    active_spinner: Option<SpinnerKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinnerKind {
    NewGame,
    CheckSolvability,
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

/// Awaitable wrapper that toggles a flow spinner while the inner future runs.
struct WithSpinnerFuture<F>
where
    F: Future,
{
    state: Rc<RefCell<FlowState>>,
    kind: SpinnerKind,
    started: bool,
    inner: Pin<Box<F>>,
}

impl<F> Future for WithSpinnerFuture<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            self.started = true;
            self.state.borrow_mut().active_spinner = Some(self.kind);
        }

        let result = self.inner.as_mut().poll(cx);

        if result.is_ready() {
            let mut state = self.state.borrow_mut();
            if state.active_spinner == Some(self.kind) {
                state.active_spinner = None;
            }
        }

        result
    }
}

fn noop_waker() -> Waker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }

    unsafe fn wake(_: *const ()) {}

    unsafe fn wake_by_ref(_: *const ()) {}

    unsafe fn drop(_: *const ()) {}

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}
