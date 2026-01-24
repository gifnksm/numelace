//! Sudoku desktop application using egui/eframe.
//!
//! This is the main entry point for the desktop Sudoku application.

use eframe::{
    NativeOptions,
    egui::{self, Vec2},
};

use crate::app::SudokuApp;

mod app;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(true)
            .with_inner_size(Vec2::new(800.0, 600.0))
            .with_min_inner_size(Vec2::new(400.0, 300.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Sudoku",
        options,
        Box::new(|cc| Ok(Box::new(SudokuApp::new(cc)))),
    )
}
