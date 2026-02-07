use eframe::egui::{Context, Id, Modal, Spinner};

use crate::action::SpinnerKind;

pub(crate) fn show(ctx: &Context, spinner: SpinnerKind) {
    ctx.request_repaint();
    match spinner {
        SpinnerKind::NewGame => {
            Modal::new(Id::new("generating_new_game")).show(ctx, |ui| {
                ui.heading("Generating...");
                ui.add(Spinner::new());
                ui.label("Generating new game...");
            });
        }
        SpinnerKind::CheckSolvability => {
            Modal::new(Id::new("checking_solvability")).show(ctx, |ui| {
                ui.heading("Checking...");
                ui.add(Spinner::new());
                ui.label("Checking solvability...");
            });
        }
    }
}
