use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

use crate::action::{Action, ActionRequestQueue};

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
    pub(crate) fn request_action(&self, action: Action) {
        self.state.borrow_mut().pending_actions.push(action);
    }
}

struct FlowTask {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

#[derive(Default)]
struct FlowState {
    pending_actions: Vec<Action>,
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
