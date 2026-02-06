use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crate::{
    action::{
        Action, ActionRequestQueue, ConfirmResult, ModalResponse, NotesFillScope,
        SolvabilityDialogResult,
    },
    async_work::{WorkRequest, WorkResponse, solvability_dto::SolvabilityStateDto},
    state::{ModalKind, SolvabilityState, SolvabilityStats},
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
    pub(crate) fn is_idle(&self) -> bool {
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

    pub(crate) fn resolve_modal_response(&mut self, response: ModalResponse) {
        let mut state = self.state.borrow_mut();
        state.modal_response = Some(response);
        if let Some(waker) = state.modal_waker.take() {
            waker.wake();
        }
    }

    /// Notify the flow executor that background work completed.
    ///
    /// This updates flow state so awaiting tasks can resume.
    pub(crate) fn record_work_response(&mut self, response: &WorkResponse) {
        let mut state = self.state.borrow_mut();

        if state.work_pending && state.work_response.is_none() {
            state.work_response = Some(response.clone());
            if let Some(waker) = state.work_waker.take() {
                waker.wake();
            }
        }
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
        match self
            .await_modal(ModalKind::NewGameConfirm, ModalResponseKind::Confirm)
            .await
        {
            ModalResponse::Confirm(result) => result,
            ModalResponse::Solvability(_) => ConfirmResult::Cancelled,
        }
    }

    /// Await the solvability result dialog.
    #[must_use]
    pub(crate) async fn await_solvability_dialog(
        &self,
        state: SolvabilityState,
    ) -> SolvabilityDialogResult {
        let modal = ModalKind::CheckSolvabilityResult(state);
        match self
            .await_modal(modal, ModalResponseKind::Solvability)
            .await
        {
            ModalResponse::Solvability(result) => result,
            ModalResponse::Confirm(_) => SolvabilityDialogResult::Close,
        }
    }

    /// Await a modal response from the UI.
    #[must_use]
    fn await_modal(&self, modal: ModalKind, expected: ModalResponseKind) -> ModalResponseFuture {
        ModalResponseFuture {
            state: Rc::clone(&self.state),
            modal,
            expected,
            started: false,
        }
    }

    /// Dispatch background work and await the response.
    #[must_use]
    pub(crate) fn await_work(&self, request: WorkRequest) -> WorkResponseFuture {
        WorkResponseFuture {
            state: Rc::clone(&self.state),
            request,
            started: false,
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

/// Async flow for new game confirmation + work dispatch.
///
/// On confirm, it runs the background request and awaits the response.
pub(crate) async fn new_game_flow(handle: FlowHandle) {
    let result = handle.confirm_new_game().await;
    if matches!(result, ConfirmResult::Confirmed) {
        let work = handle.await_work(WorkRequest::GenerateNewGame);
        let _ = handle.with_spinner(SpinnerKind::NewGame, work).await;
    }
}

/// Async flow for solvability check work dispatch.
///
/// Runs the background request and awaits the response.
pub(crate) async fn check_solvability_flow(handle: FlowHandle, request: WorkRequest) {
    let work = handle.await_work(request);
    let response = handle
        .with_spinner(SpinnerKind::CheckSolvability, work)
        .await;

    let WorkResponse::SolvabilityReady(state) = response else {
        return;
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
    modal_response: Option<ModalResponse>,
    modal_waker: Option<Waker>,

    work_pending: bool,
    work_response: Option<WorkResponse>,
    work_waker: Option<Waker>,
    active_spinner: Option<SpinnerKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinnerKind {
    NewGame,
    CheckSolvability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalResponseKind {
    Confirm,
    Solvability,
}

/// Awaitable for modal responses.
struct ModalResponseFuture {
    state: Rc<RefCell<FlowState>>,
    modal: ModalKind,
    expected: ModalResponseKind,
    started: bool,
}

impl Future for ModalResponseFuture {
    type Output = ModalResponse;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            self.started = true;
            self.state
                .borrow_mut()
                .pending_actions
                .push(Action::OpenModal(self.modal.clone()));
        }

        let mut state = self.state.borrow_mut();
        if let Some(response) = state.modal_response.take() {
            let matches_expected = matches!(
                (self.expected, &response),
                (ModalResponseKind::Confirm, ModalResponse::Confirm(_))
                    | (
                        ModalResponseKind::Solvability,
                        ModalResponse::Solvability(_)
                    )
            );
            if matches_expected {
                Poll::Ready(response)
            } else {
                state.modal_response = Some(response);
                state.modal_waker = Some(cx.waker().clone());
                Poll::Pending
            }
        } else {
            state.modal_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
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

/// Awaitable for background work responses.
pub(crate) struct WorkResponseFuture {
    state: Rc<RefCell<FlowState>>,
    request: WorkRequest,
    started: bool,
}

impl Future for WorkResponseFuture {
    type Output = WorkResponse;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            self.started = true;
            let mut state = self.state.borrow_mut();
            state.work_pending = true;
            state.work_response = None;
            state.work_waker = None;
            state
                .pending_actions
                .push(Action::StartWork(self.request.clone()));
        }

        let mut state = self.state.borrow_mut();
        if let Some(response) = state.work_response.take() {
            state.work_pending = false;
            Poll::Ready(response)
        } else {
            state.work_waker = Some(cx.waker().clone());
            Poll::Pending
        }
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
