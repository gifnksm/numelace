use eframe::{
    CreationContext, Frame,
    egui::{CentralPanel, Context},
};

#[derive(Default, Debug)]
pub struct SudokuApp {}

impl SudokuApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {}
    }
}

impl eframe::App for SudokuApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello");
        });
    }
}
