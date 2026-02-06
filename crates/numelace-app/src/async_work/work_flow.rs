//! Centralized async work handling for the app.

use crate::{
    action::{Action, ActionRequestQueue},
    state::WorkState,
};

use super::{WorkError, WorkHandle, WorkRequest, WorkResponse};

/// Status from polling the async workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkFlowStatus {
    Idle,
    Pending,
    Completed,
    Failed,
}

/// Minimal workflow coordinator for background tasks.
#[derive(Debug, Default)]
pub struct WorkFlow;

impl WorkFlow {
    /// Poll the current work handle and enqueue any resulting action.
    pub fn poll_and_queue(
        &self,
        work: &mut WorkState,
        action_queue: &mut ActionRequestQueue,
    ) -> WorkFlowStatus {
        let Some(handle) = work.pending.as_mut() else {
            return WorkFlowStatus::Idle;
        };

        match handle.poll() {
            Ok(Some(response)) => {
                work.pending = None;
                action_queue.request(Action::ApplyWorkResponse(response));
                WorkFlowStatus::Completed
            }
            Ok(None) => WorkFlowStatus::Pending,
            Err(err) => {
                Self::record_error(work, err.clone());
                panic!("background work poll failed: {err}");
            }
        }
    }

    /// Record an error from the async pipeline.
    pub fn record_error(work: &mut WorkState, err: WorkError) {
        work.pending = None;
        work.is_generating_new_game = false;
        work.is_checking_solvability = false;
        work.last_error = Some(err);
    }

    /// Mark a request as started with the given handle.
    pub fn start_request(work: &mut WorkState, request: &WorkRequest, handle: WorkHandle) {
        work.pending = Some(handle);
        work.is_generating_new_game = matches!(request, WorkRequest::GenerateNewGame);
        work.is_checking_solvability = matches!(request, WorkRequest::CheckSolvability(_));
    }

    /// Returns true if any background work is currently in flight.
    #[must_use]
    pub fn is_work_in_flight(work: &WorkState) -> bool {
        work.is_generating_new_game || work.is_checking_solvability
    }

    /// Clear pending state after a response is handled.
    pub fn finish_response(work: &mut WorkState, _response: &WorkResponse) {
        work.pending = None;
        work.is_generating_new_game = false;
        work.is_checking_solvability = false;
        work.last_error = None;
    }
}
