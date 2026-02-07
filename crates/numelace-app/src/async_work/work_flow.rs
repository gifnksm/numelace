//! Centralized async work handling for the app.

use crate::{
    action::{Action, ActionRequestQueue},
    state::{WorkEntry, WorkState},
};

use super::{WorkError, WorkHandle, WorkResponse};

/// Minimal workflow coordinator for background tasks.
#[derive(Debug, Default)]
pub(crate) struct WorkFlow;

impl WorkFlow {
    /// Poll all in-flight work handles and enqueue any resulting actions.
    pub(crate) fn poll_and_queue(work: &mut WorkState, action_queue: &mut ActionRequestQueue) {
        let mut i = 0;
        while i < work.in_flight.len() {
            let entry = &mut work.in_flight[i];
            match entry.handle.poll() {
                Ok(Some(response)) => {
                    let entry = work.in_flight.swap_remove(i);
                    let _ = entry.responder.send(response.clone());
                    action_queue.request(Action::ApplyWorkResponse(response));
                }
                Ok(None) => {
                    i += 1;
                }
                Err(err) => {
                    work.in_flight.swap_remove(i);
                    Self::record_error(work, err.clone());
                    panic!("background work poll failed: {err}");
                }
            }
        }
    }

    /// Record an error from the async pipeline.
    pub(crate) fn record_error(work: &mut WorkState, err: WorkError) {
        work.in_flight.clear();
        work.last_error = Some(err);
    }

    /// Mark a request as started with the given handle.
    pub(crate) fn start_request(
        work: &mut WorkState,
        handle: WorkHandle,
        responder: crate::action::WorkResponder,
    ) {
        work.in_flight.push(WorkEntry { handle, responder });
    }

    /// Clear pending state after a response is handled.
    pub(crate) fn finish_response(work: &mut WorkState, _response: &WorkResponse) {
        work.last_error = None;
    }
}
