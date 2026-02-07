//! Worker module split by platform.
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

use self::tasks::{
    GeneratedPuzzleDto, SolvabilityRequestDto, SolvabilityStateDto, SolvabilityUndoGridsDto,
    SolvabilityUndoScanResultDto,
};
pub(crate) use platform::warm_up;
use platform::{WorkHandle, enqueue};

pub(crate) mod api;
mod platform;
pub(crate) mod tasks;

/// A request that can be offloaded to a background worker.
///
/// Internal: prefer typed helpers like `request_generate_puzzle` and `request_solvability`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum WorkRequest {
    /// Generate a Sudoku puzzle.
    GeneratePuzzle,
    /// Check solvability for a given puzzle state.
    CheckSolvability(SolvabilityRequestDto),
    /// Scan undo history for a solvable state.
    CheckSolvabilityUndoScan(SolvabilityUndoGridsDto),
}

/// A response produced by background work.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum WorkResponse {
    /// Generated puzzle data ready for a fresh game.
    GeneratedPuzzleReady(GeneratedPuzzleDto),
    /// Solvability result ready for display.
    SolvabilityReady(SolvabilityStateDto),
    /// Undo scan result ready for display.
    SolvabilityUndoScanReady(SolvabilityUndoScanResultDto),
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
                WorkResponse::GeneratedPuzzleReady(tasks::generate_puzzle())
            }
            WorkRequest::CheckSolvability(request) => {
                match tasks::handle_solvability_request(request) {
                    Ok(result) => WorkResponse::SolvabilityReady(result),
                    Err(_) => WorkResponse::Error(WorkError::DeserializationFailed),
                }
            }
            WorkRequest::CheckSolvabilityUndoScan(request) => {
                match tasks::handle_solvability_undo_scan(request) {
                    Ok(result) => WorkResponse::SolvabilityUndoScanReady(result),
                    Err(_) => WorkResponse::Error(WorkError::DeserializationFailed),
                }
            }
        }
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
        _ => Err(WorkError::UnexpectedResponse),
    }
}

/// Enqueue background work for solvability check and return the state.
pub(crate) async fn request_solvability(
    solvability_request: SolvabilityRequestDto,
) -> Result<SolvabilityStateDto, WorkError> {
    match request(WorkRequest::CheckSolvability(solvability_request)).await {
        WorkResponse::SolvabilityReady(state) => Ok(state),
        WorkResponse::Error(err) => Err(err),
        _ => Err(WorkError::UnexpectedResponse),
    }
}

pub(crate) async fn request_solvability_undo_scan(
    undo_grids: SolvabilityUndoGridsDto,
) -> Result<SolvabilityUndoScanResultDto, WorkError> {
    match request(WorkRequest::CheckSolvabilityUndoScan(undo_grids)).await {
        WorkResponse::SolvabilityUndoScanReady(result) => Ok(result),
        WorkResponse::Error(err) => Err(err),
        _ => Err(WorkError::UnexpectedResponse),
    }
}
