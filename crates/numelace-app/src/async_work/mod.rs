//! Async work module split by platform.
//!
//! This module defines shared request/response types and delegates the
//! implementation to platform-specific modules to keep `#[cfg]` usage
//! centralized. The `native` module uses threads/channels, while the
//! `wasm` module uses a Web Worker with message passing.

use crate::game_factory;

pub mod new_game_dto;
mod platform;
pub mod work_actions;
pub mod work_flow;

use new_game_dto::NewGameDto;

/// A request that can be offloaded to a background worker.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WorkRequest {
    /// Generate a new Sudoku puzzle.
    GenerateNewGame,
}

/// A response produced by background work.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WorkResponse {
    /// New puzzle data ready for a fresh game.
    NewGameReady(NewGameDto),
    /// An error occurred while performing background work.
    Error(WorkError),
}

/// Errors that can occur while scheduling or receiving background work.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WorkError {
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
    pub fn handle(&self) -> WorkResponse {
        match self {
            WorkRequest::GenerateNewGame => {
                WorkResponse::NewGameReady(game_factory::generate_new_game_dto())
            }
        }
    }
}

pub use platform::{WorkHandle, enqueue, warm_up};
