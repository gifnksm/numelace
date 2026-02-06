use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crate::{
    action::{Action, ActionRequestQueue, ConfirmResult},
    async_work::{WorkError, WorkResponse},
    state::ModalKind,
};

/// Lightweight async flow executor for UI orchestration.
///
/// This executor is polled from the app update loop and drives flow futures
/// that request UI actions and await UI events.
pub struct FlowExecutor {
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
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(FlowState::default())),
            tasks: Vec::new(),
        }
    }

    /// Returns a handle for flows to request actions and await events.
    #[must_use]
    pub fn handle(&self) -> FlowHandle {
        FlowHandle {
            state: Rc::clone(&self.state),
        }
    }

    /// Returns true if no flows are currently running.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Spawn a new flow future.
    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        self.tasks.push(FlowTask {
            future: Box::pin(future),
        });
    }

    /// Poll all active flows and drain any queued actions into the UI action queue.
    pub fn poll(&mut self, action_queue: &mut ActionRequestQueue) {
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
    pub fn confirm_new_game(&mut self, result: ConfirmResult) {
        let mut state = self.state.borrow_mut();
        state.new_game_confirm = Some(result);
        if let Some(waker) = state.new_game_confirm_waker.take() {
            waker.wake();
        }
    }

    /// Notify the flow executor that background work completed.
    ///
    /// This updates flow state so awaiting tasks can resume.
    pub fn record_work_response(&mut self, response: &WorkResponse) {
        let mut state = self.state.borrow_mut();
        if !state.new_game_completion_pending || state.new_game_completion.is_some() {
            return;
        }

        match response {
            WorkResponse::NewGameReady(_) => {
                state.new_game_completion = Some(NewGameCompletion::Completed);
            }
            WorkResponse::Error(err) => {
                state.new_game_completion = Some(NewGameCompletion::Failed(err.clone()));
            }
            WorkResponse::SolvabilityReady(_) => {}
        }

        if let Some(waker) = state.new_game_completion_waker.take() {
            waker.wake();
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
pub struct FlowHandle {
    state: Rc<RefCell<FlowState>>,
}

impl FlowHandle {
    /// Request an action to be queued for the next update loop.
    pub fn request_action(&self, action: Action) {
        self.state.borrow_mut().pending_actions.push(action);
    }

    /// Mark the new game completion path as pending.
    pub fn mark_new_game_completion_pending(&self) {
        let mut state = self.state.borrow_mut();
        state.new_game_completion_pending = true;
        state.new_game_completion = None;
        state.new_game_completion_waker = None;
    }

    /// Await a new game confirmation dialog.
    #[must_use]
    pub fn confirm_new_game(&self) -> ConfirmNewGameFuture {
        ConfirmNewGameFuture {
            state: Rc::clone(&self.state),
            started: false,
        }
    }

    /// Await background completion of a new game request.
    #[must_use]
    pub fn await_new_game_completion(&self) -> NewGameCompletionFuture {
        NewGameCompletionFuture {
            state: Rc::clone(&self.state),
        }
    }
}

/// Async flow for new game confirmation + work dispatch.
///
/// On confirm, it queues `StartNewGame` which triggers the async work pipeline,
/// then awaits the completion.
pub async fn new_game_flow(handle: FlowHandle) {
    let result = handle.confirm_new_game().await;
    if matches!(result, ConfirmResult::Confirmed) {
        handle.mark_new_game_completion_pending();
        handle.request_action(Action::StartNewGame);
        let _ = handle.await_new_game_completion().await;
    }
}

struct FlowTask {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

#[derive(Default)]
struct FlowState {
    pending_actions: Vec<Action>,
    new_game_confirm: Option<ConfirmResult>,
    new_game_confirm_waker: Option<Waker>,
    new_game_completion_pending: bool,
    new_game_completion: Option<NewGameCompletion>,
    new_game_completion_waker: Option<Waker>,
}

#[derive(Debug, Clone)]
pub enum NewGameCompletion {
    Completed,
    Failed(WorkError),
}

/// Awaitable for the new game confirmation dialog.
///
/// On first poll, it opens the dialog via an action request.
pub struct ConfirmNewGameFuture {
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

/// Awaitable for new game completion.
pub struct NewGameCompletionFuture {
    state: Rc<RefCell<FlowState>>,
}

impl Future for NewGameCompletionFuture {
    type Output = NewGameCompletion;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();
        state.new_game_completion_pending = true;
        if let Some(result) = state.new_game_completion.take() {
            state.new_game_completion_pending = false;
            Poll::Ready(result)
        } else {
            state.new_game_completion_waker = Some(cx.waker().clone());
            Poll::Pending
        }
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
