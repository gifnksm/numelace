//! Shared library module for the Numelace app crate.
#![allow(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub const DEFAULT_MAX_HISTORY_LENGTH: usize = 200;

pub mod action;
pub mod action_handler;
pub mod app;
pub mod async_work;
pub mod flow;
pub mod game_factory;
pub mod history;
pub mod persistence;
pub mod state;
pub mod ui;
pub mod view_model_builder;
