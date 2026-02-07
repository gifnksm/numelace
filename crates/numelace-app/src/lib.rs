//! Shared library module for the Numelace app crate.
#![allow(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub(crate) const DEFAULT_MAX_HISTORY_LENGTH: usize = 5000;

pub(crate) mod action;
pub(crate) mod app;
pub(crate) mod flow;
pub(crate) mod game_factory;
pub(crate) mod history;
pub(crate) mod persistence;
pub(crate) mod state;
pub(crate) mod ui;
pub(crate) mod view_model_builder;
pub(crate) mod worker;
pub mod worker_api;

pub use self::app::NumelaceApp;
