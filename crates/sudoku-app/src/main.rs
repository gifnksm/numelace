//! Sudoku desktop application using egui/eframe.
//!
//! This is the main entry point for the desktop Sudoku application.

use eframe::{NativeOptions, egui};

use crate::app::SudokuApp;

mod app;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "Sudoku",
        options,
        Box::new(|cc| Ok(Box::new(SudokuApp::new(cc)))),
    )
}
