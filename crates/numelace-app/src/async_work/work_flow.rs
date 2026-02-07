//! Centralized async work handling for the app.

use crate::{
    action::{Action, ActionRequestQueue},
    state::WorkState,
};

use super::{WorkError, WorkHandle, WorkRequest, WorkResponse};

/// Minimal workflow coordinator for background tasks.
#[derive(Debug, Default)]
pub(crate) struct WorkFlow;

impl WorkFlow {
    /// Poll the current work handle and enqueue any resulting action.
    pub(crate) fn poll_and_queue(work: &mut WorkState, action_queue: &mut ActionRequestQueue) {
        let Some(handle) = work.pending.as_mut() else {
            return;
        };

        match handle.poll() {
            Ok(Some(response)) => {
                work.pending = None;
                action_queue.request(Action::ApplyWorkResponse(response));
            }
            Ok(None) => {}
            Err(err) => {
                Self::record_error(work, err.clone());
                panic!("background work poll failed: {err}");
            }
        }
    }

    /// Record an error from the async pipeline.
    pub(crate) fn record_error(work: &mut WorkState, err: WorkError) {
        work.pending = None;
        work.in_flight = None;
        work.work_responder = None;
        work.last_error = Some(err);
    }

    /// Mark a request as started with the given handle.
    pub(crate) fn start_request(work: &mut WorkState, request: &WorkRequest, handle: WorkHandle) {
        work.pending = Some(handle);
        work.in_flight = Some(request.kind());
    }

    /// Returns true if any background work is currently in flight.
    #[must_use]
    pub(crate) fn is_work_in_flight(work: &WorkState) -> bool {
        work.in_flight.is_some()
    }

    /// Clear pending state after a response is handled.
    pub(crate) fn finish_response(work: &mut WorkState, _response: &WorkResponse) {
        work.pending = None;
        work.in_flight = None;
        work.work_responder = None;
        work.last_error = None;
    }
}
