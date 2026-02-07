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

pub(crate) mod new_game_dto;
mod platform;
pub(crate) mod solvability_dto;

use new_game_dto::NewGameDto;
use solvability_dto::{SolvabilityRequestDto, SolvabilityStateDto};

/// A request that can be offloaded to a background worker.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum WorkRequest {
    /// Generate a new Sudoku puzzle.
    GenerateNewGame,
    /// Check solvability for a given puzzle state.
    CheckSolvability(SolvabilityRequestDto),
}

/// A response produced by background work.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum WorkResponse {
    /// New puzzle data ready for a fresh game.
    NewGameReady(NewGameDto),
    /// Solvability result ready for display.
    SolvabilityReady(SolvabilityStateDto),
    /// An error occurred while performing background work.
    Error(WorkError),
}

/// Errors that can occur while scheduling or receiving background work.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum WorkError {
    /// The worker URL was missing or invalid.
    WorkerUrlMissing,
    /// Failed to initialize the worker instance.
    WorkerInitFailed,
    /// Failed to serialize a request payload.
    SerializationFailed,
    /// Failed to deserialize a response payload.
    DeserializationFailed,
    /// The background channel was disconnected unexpectedly.
    WorkerDisconnected,
}

impl std::fmt::Display for WorkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkError::WorkerUrlMissing => write!(f, "worker URL is missing"),
            WorkError::WorkerInitFailed => write!(f, "worker initialization failed"),
            WorkError::SerializationFailed => write!(f, "failed to serialize worker payload"),
            WorkError::DeserializationFailed => write!(f, "failed to deserialize worker payload"),
            WorkError::WorkerDisconnected => write!(f, "worker disconnected"),
        }
    }
}

impl std::error::Error for WorkError {}

impl WorkRequest {
    /// Handle a request and produce the corresponding response.
    ///
    /// This keeps the request-to-response mapping centralized across backends.
    #[must_use]
    pub(crate) fn handle(self) -> WorkResponse {
        match self {
            WorkRequest::GenerateNewGame => {
                WorkResponse::NewGameReady(game_factory::generate_random_puzzle().into())
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
pub(crate) struct WorkResponseFuture {
    handle: Option<WorkHandle>,
    response: Option<WorkResponse>,
}

impl WorkResponseFuture {
    #[must_use]
    pub(crate) fn new(result: Result<WorkHandle, WorkError>) -> Self {
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
pub(crate) fn request(request: WorkRequest) -> WorkResponseFuture {
    WorkResponseFuture::new(enqueue(request))
}
