use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crate::{
    action::{Action, ActionRequestQueue, ConfirmResult},
    async_work::{WorkRequest, WorkResponse},
    state::ModalKind,
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

    /// Provide the result of the new game confirmation dialog.
    pub(crate) fn confirm_new_game(&mut self, result: ConfirmResult) {
        let mut state = self.state.borrow_mut();
        state.new_game_confirm = Some(result);
        if let Some(waker) = state.new_game_confirm_waker.take() {
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
    /// Await a new game confirmation dialog.
    #[must_use]
    fn confirm_new_game(&self) -> ConfirmNewGameFuture {
        ConfirmNewGameFuture {
            state: Rc::clone(&self.state),
            started: false,
        }
    }

    /// Dispatch background work and await the response.
    #[must_use]
    fn await_work(&self, request: WorkRequest) -> WorkResponseFuture {
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
    let _ = handle
        .with_spinner(SpinnerKind::CheckSolvability, work)
        .await;
}

struct FlowTask {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

#[derive(Default)]
struct FlowState {
    pending_actions: Vec<Action>,
    new_game_confirm: Option<ConfirmResult>,
    new_game_confirm_waker: Option<Waker>,

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

/// Awaitable for the new game confirmation dialog.
///
/// On first poll, it opens the dialog via an action request.
struct ConfirmNewGameFuture {
    state: Rc<RefCell<FlowState>>,
    started: bool,
}

impl Future for ConfirmNewGameFuture {
    type Output = ConfirmResult;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            self.started = true;
            self.state
                .borrow_mut()
                .pending_actions
                .push(Action::OpenModal(ModalKind::NewGameConfirm));
        }

        let mut state = self.state.borrow_mut();
        if let Some(result) = state.new_game_confirm.take() {
            Poll::Ready(result)
        } else {
            state.new_game_confirm_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Awaitable for background work responses.
struct WorkResponseFuture {
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
