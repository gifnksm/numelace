//! Async work module split by platform.
//!
//! This module defines shared request/response types and delegates the
//! implementation to platform-specific modules to keep `#[cfg]` usage
//! centralized. The `native` module uses threads/channels, while the
//! `wasm` module uses a Web Worker with message passing.

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::game_factory;
pub(crate) use platform::warm_up;
use platform::{WorkHandle, enqueue};

pub(crate) mod generated_puzzle_dto;
mod platform;
pub(crate) mod solvability_dto;

use generated_puzzle_dto::GeneratedPuzzleDto;
use solvability_dto::{SolvabilityRequestDto, SolvabilityStateDto};

pub(crate) mod worker_api {
    use serde::{Deserialize, Serialize};

    use super::{WorkRequest as InnerWorkRequest, WorkResponse as InnerWorkResponse};

    #[derive(Deserialize, Serialize)]
    #[serde(transparent)]
    pub struct WorkRequest(InnerWorkRequest);

    #[derive(Deserialize, Serialize)]
    #[serde(transparent)]
    pub struct WorkResponse(InnerWorkResponse);

    impl WorkRequest {
        #[must_use]
        pub fn handle(self) -> WorkResponse {
            WorkResponse(self.0.handle())
        }
    }

    impl WorkResponse {
        #[must_use]
        pub fn deserialization_error() -> Self {
            Self(InnerWorkResponse::Error(
                super::WorkError::DeserializationFailed,
            ))
        }
    }
}

/// A request that can be offloaded to a background worker.
///
/// Internal: prefer typed helpers like `request_generate_puzzle` and `request_solvability`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum WorkRequest {
    /// Generate a Sudoku puzzle.
    GeneratePuzzle,
    /// Check solvability for a given puzzle state.
    CheckSolvability(SolvabilityRequestDto),
}

/// A response produced by background work.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum WorkResponse {
    /// Generated puzzle data ready for a fresh game.
    GeneratedPuzzleReady(GeneratedPuzzleDto),
    /// Solvability result ready for display.
    SolvabilityReady(SolvabilityStateDto),
    /// An error occurred while performing background work.
    Error(WorkError),
}

/// Errors that can occur while scheduling or receiving background work.
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, derive_more::Display, derive_more::Error,
)]
pub(crate) enum WorkError {
    /// The worker URL was missing or invalid.
    #[display("worker URL is missing")]
    WorkerUrlMissing,
    /// Failed to initialize the worker instance.
    #[display("worker initialization failed")]
    WorkerInitFailed,
    /// Failed to serialize a request payload.
    #[display("failed to serialize worker payload")]
    SerializationFailed,
    /// Failed to deserialize a response payload.
    #[display("failed to deserialize worker payload")]
    DeserializationFailed,
    /// The background channel was disconnected unexpectedly.
    #[display("worker disconnected")]
    WorkerDisconnected,
    /// Received a response that does not match the request.
    #[display("unexpected worker response")]
    UnexpectedResponse,
}

impl WorkRequest {
    /// Handle a request and produce the corresponding response.
    ///
    /// This keeps the request-to-response mapping centralized across backends.
    #[must_use]
    fn handle(self) -> WorkResponse {
        match self {
            WorkRequest::GeneratePuzzle => {
                WorkResponse::GeneratedPuzzleReady(game_factory::generate_random_puzzle().into())
            }
            WorkRequest::CheckSolvability(request) => handle_solvability_request(request),
        }
    }
}

fn handle_solvability_request(request: SolvabilityRequestDto) -> WorkResponse {
    let Ok(with_user_notes) = request.with_user_notes.try_into() else {
        return WorkResponse::Error(WorkError::DeserializationFailed);
    };
    let Ok(without_user_notes) = request.without_user_notes.try_into() else {
        return WorkResponse::Error(WorkError::DeserializationFailed);
    };

    let first_result = check_grid_solvability(with_user_notes, true);
    let result = if matches!(
        first_result,
        SolvabilityStateDto::Inconsistent | SolvabilityStateDto::NoSolution
    ) {
        check_grid_solvability(without_user_notes, false)
    } else {
        first_result
    };

    WorkResponse::SolvabilityReady(result)
}

fn check_grid_solvability(
    grid: numelace_core::CandidateGrid,
    with_user_notes: bool,
) -> SolvabilityStateDto {
    if grid.check_consistency().is_err() {
        return SolvabilityStateDto::Inconsistent;
    }

    let solver = numelace_solver::BacktrackSolver::with_all_techniques();
    match solver.solve(grid).map(|mut sol| sol.next()) {
        Ok(Some((_grid, stats))) => SolvabilityStateDto::Solvable {
            with_user_notes,
            stats: stats.into(),
        },
        Ok(None) | Err(_) => SolvabilityStateDto::NoSolution,
    }
}

/// Future that resolves to a background work response.
struct WorkResponseFuture {
    handle: Option<WorkHandle>,
    response: Option<WorkResponse>,
}

impl WorkResponseFuture {
    #[must_use]
    fn new(result: Result<WorkHandle, WorkError>) -> Self {
        match result {
            Ok(handle) => Self {
                handle: Some(handle),
                response: None,
            },
            Err(err) => Self {
                handle: None,
                response: Some(WorkResponse::Error(err)),
            },
        }
    }
}

impl Future for WorkResponseFuture {
    type Output = WorkResponse;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(response) = self.response.take() {
            return Poll::Ready(response);
        }

        let Some(handle) = self.handle.as_mut() else {
            return Poll::Ready(WorkResponse::Error(WorkError::WorkerDisconnected));
        };

        match handle.poll() {
            Ok(Some(response)) => Poll::Ready(response),
            Ok(None) => Poll::Pending,
            Err(err) => Poll::Ready(WorkResponse::Error(err)),
        }
    }
}

/// Enqueue background work and return a future for the response.
#[must_use]
fn request(request: WorkRequest) -> WorkResponseFuture {
    WorkResponseFuture::new(enqueue(request))
}

/// Enqueue background work for a generated puzzle and return the DTO.
pub(crate) async fn request_generate_puzzle() -> Result<GeneratedPuzzleDto, WorkError> {
    match request(WorkRequest::GeneratePuzzle).await {
        WorkResponse::GeneratedPuzzleReady(dto) => Ok(dto),
        WorkResponse::Error(err) => Err(err),
        WorkResponse::SolvabilityReady(_) => Err(WorkError::UnexpectedResponse),
    }
}

/// Enqueue background work for solvability check and return the state.
pub(crate) async fn request_solvability(
    solvability_request: SolvabilityRequestDto,
) -> Result<SolvabilityStateDto, WorkError> {
    match request(WorkRequest::CheckSolvability(solvability_request)).await {
        WorkResponse::SolvabilityReady(state) => Ok(state),
        WorkResponse::Error(err) => Err(err),
        WorkResponse::GeneratedPuzzleReady(_) => Err(WorkError::UnexpectedResponse),
    }
}
