use std::sync::Arc;

use eframe::egui::{
    Align2, Button, Color32, FontId, Grid, Painter, Rect, RichText, Stroke, StrokeKind, Ui, Vec2,
    Visuals,
};
use numelace_core::{Digit, DigitSet, Position, containers::Array81, index::PositionSemantics};
use numelace_game::CellState;

use crate::{
    action::{Action, ActionRequestQueue},
    state::HighlightSettings,
};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct GridVisualState: u8 {
        const SELECTED = 0b0000_0001;
        const SAME_DIGIT = 0b0000_0010;
        const HOUSE_SELECTED = 0b0000_0100;
        const HOUSE_SAME_DIGIT = 0b0000_1000;
        const CONFLICT = 0b0001_0000;
        const GHOST = 0b0010_0000;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridCell {
    pub content: CellState,
    pub visual_state: GridVisualState,
    pub note_visual_state: NoteVisualState,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct NoteVisualState {
    pub same_digit: DigitSet,
    pub conflict: DigitSet,
    pub ghost: DigitSet,
}

impl NoteVisualState {
    pub fn digit_highlight(&self, digit: Digit) -> GridVisualState {
        let Self {
            same_digit,
            conflict,
            ghost,
        } = self;
        let mut vs = GridVisualState::empty();
        if same_digit.contains(digit) {
            vs |= GridVisualState::SAME_DIGIT;
        }
        if conflict.contains(digit) {
            vs |= GridVisualState::CONFLICT;
        }
        if ghost.contains(digit) {
            vs |= GridVisualState::GHOST;
        }
        vs
    }
}

#[derive(Debug, Clone)]
pub struct GridViewModel {
    grid: Array81<GridCell, PositionSemantics>,
    enabled_highlights: GridVisualState,
}

impl GridViewModel {
    pub fn new(
        grid: Array81<GridCell, PositionSemantics>,
        highlight_settings: &HighlightSettings,
    ) -> Self {
        let mut enabled_highlights = GridVisualState::SELECTED;
        let HighlightSettings {
            same_digit,
            house_selected,
            house_same_digit,
            conflict,
        } = highlight_settings;
        if *house_same_digit {
            enabled_highlights |= GridVisualState::HOUSE_SAME_DIGIT;
        }
        if *house_selected {
            enabled_highlights |= GridVisualState::HOUSE_SELECTED;
        }
        if *same_digit {
            enabled_highlights |= GridVisualState::SAME_DIGIT;
        }
        if *conflict {
            enabled_highlights |= GridVisualState::CONFLICT;
        }
        Self {
            grid,
            enabled_highlights,
        }
    }

    fn inactive_border_color(visuals: &Visuals) -> Color32 {
        visuals.widgets.inactive.fg_stroke.color
    }

    fn grid_thick_border(visuals: &Visuals, cell_size: f32) -> Stroke {
        let base_width = f32::max(cell_size * CELL_BORDER_WIDTH_BASE_RATIO, 1.0);
        Stroke::new(
            base_width * THICK_BORDER_RATIO,
            Self::inactive_border_color(visuals),
        )
    }

    fn effective_visual_state(&self, state: GridVisualState) -> EffectiveGridVisualState {
        EffectiveGridVisualState(self.enabled_highlights & state)
    }
}

const CELL_BORDER_WIDTH_BASE_RATIO: f32 = 0.03;
const THICK_BORDER_RATIO: f32 = 2.0;
const THIN_BORDER_WIDTH_RATIO: f32 = 1.0;
const SELECTED_BORDER_WIDTH_RATIO: f32 = 3.0;
const SAME_DIGIT_BORDER_WIDTH_RATIO: f32 = 1.0;
const HOUSE_BORDER_WIDTH_RATIO: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EffectiveGridVisualState(GridVisualState);

impl EffectiveGridVisualState {
    fn text_color(self, is_given: bool, visuals: &Visuals) -> Color32 {
        if self.0.intersects(GridVisualState::CONFLICT) {
            return visuals.error_fg_color;
        }
        if is_given {
            visuals.strong_text_color()
        } else {
            visuals.text_color()
        }
    }

    fn cell_fill_color(self, visuals: &Visuals) -> Color32 {
        if self
            .0
            .intersects(GridVisualState::SELECTED | GridVisualState::SAME_DIGIT)
        {
            return visuals.selection.bg_fill;
        }
        if self
            .0
            .intersects(GridVisualState::HOUSE_SELECTED | GridVisualState::HOUSE_SAME_DIGIT)
        {
            return visuals.widgets.hovered.bg_fill;
        }
        visuals.text_edit_bg_color()
    }

    fn note_fill_color(self, visuals: &Visuals) -> Option<Color32> {
        if self
            .0
            .intersects(GridVisualState::SAME_DIGIT | GridVisualState::HOUSE_SAME_DIGIT)
        {
            return Some(self.cell_fill_color(visuals));
        }
        None
    }

    fn cell_border_color(self, visuals: &Visuals) -> Color32 {
        if self.0.intersects(GridVisualState::CONFLICT) {
            return visuals.error_fg_color;
        }

        if self
            .0
            .intersects(GridVisualState::SELECTED | GridVisualState::SAME_DIGIT)
        {
            return visuals.selection.stroke.color;
        }
        GridViewModel::inactive_border_color(visuals)
    }

    fn cell_border_width_ratio(self) -> f32 {
        if self.0.intersects(GridVisualState::SELECTED) {
            SELECTED_BORDER_WIDTH_RATIO
        } else if self.0.intersects(GridVisualState::SAME_DIGIT) {
            SAME_DIGIT_BORDER_WIDTH_RATIO
        } else if self
            .0
            .intersects(GridVisualState::HOUSE_SELECTED | GridVisualState::HOUSE_SAME_DIGIT)
        {
            HOUSE_BORDER_WIDTH_RATIO
        } else {
            THIN_BORDER_WIDTH_RATIO
        }
    }

    fn cell_border(self, visuals: &Visuals, cell_size: f32) -> Stroke {
        let color = self.cell_border_color(visuals);
        let ratio = self.cell_border_width_ratio();
        let base_width = f32::max(cell_size * CELL_BORDER_WIDTH_BASE_RATIO, 1.0);
        Stroke::new(base_width * ratio, color)
    }
}

pub fn show(ui: &mut Ui, vm: &GridViewModel, action_queue: &mut ActionRequestQueue) {
    let style = Arc::clone(ui.style());
    let visuals = &style.visuals;

    let grid_size = ui.available_size().min_elem();
    // - 9 cells across
    // - 2 inner 3x3 borders
    // - outer borders are drawn outside layout (not included here)
    let cell_size = grid_size / (9.0 + 2.0 * CELL_BORDER_WIDTH_BASE_RATIO * THICK_BORDER_RATIO);
    let thick_border = GridViewModel::grid_thick_border(visuals, cell_size);

    Grid::new(ui.id().with("outer_board"))
        .spacing((thick_border.width, thick_border.width))
        .min_col_width(cell_size * 3.0)
        .min_row_height(cell_size * 3.0)
        .show(ui, |ui| {
            for box_row in 0..3 {
                for box_col in 0..3 {
                    let box_index = box_row * 3 + box_col;
                    let grid = Grid::new(ui.id().with(format!("inner_box_{box_row}_{box_col}")))
                        .spacing((0.0, 0.0))
                        .min_col_width(cell_size)
                        .min_row_height(cell_size)
                        .show(ui, |ui| {
                            for cell_row in 0..3 {
                                for cell_col in 0..3 {
                                    let cell_index = cell_row * 3 + cell_col;
                                    let pos = Position::from_box(box_index, cell_index);
                                    let cell = &vm.grid[pos];
                                    let vs = vm.effective_visual_state(cell.visual_state);
                                    let text_color =
                                        vs.text_color(cell.content.is_given(), visuals);
                                    let text = if let Some(digit) = cell.content.as_digit() {
                                        RichText::new(digit.as_str())
                                    } else {
                                        RichText::new("")
                                    }
                                    .color(text_color)
                                    .size(cell_size * 0.8);
                                    let button = Button::new(text)
                                        .min_size(Vec2::splat(cell_size))
                                        .fill(vs.cell_fill_color(visuals));
                                    let button = ui.add(button);
                                    if let Some(digits) = cell.content.as_notes() {
                                        let rect = button.rect.shrink(thick_border.width);
                                        draw_notes(
                                            ui.painter(),
                                            vm,
                                            rect,
                                            digits,
                                            &cell.note_visual_state,
                                            visuals,
                                        );
                                    }
                                    ui.painter().rect_stroke(
                                        button.rect,
                                        0.0,
                                        vs.cell_border(visuals, cell_size),
                                        StrokeKind::Inside,
                                    );
                                    if button.clicked() {
                                        action_queue.request(Action::SelectCell(pos));
                                    }
                                }
                                ui.end_row();
                            }
                        });
                    ui.painter().rect_stroke(
                        grid.response.rect,
                        0.0,
                        thick_border,
                        StrokeKind::Outside,
                    );
                }
                ui.end_row();
            }
        });
}

fn draw_notes(
    painter: &Painter,
    vm: &GridViewModel,
    rect: Rect,
    digits: DigitSet,
    note_visual_state: &NoteVisualState,
    visuals: &Visuals,
) {
    let note_font = FontId::proportional(rect.height() / 3.0);

    let cell_w = rect.width() / 3.0;
    let cell_h = rect.height() / 3.0;

    for digit in Digit::ALL {
        if !digits.contains(digit) {
            continue;
        }
        let idx = digit.value() - 1;
        let y = f32::from(idx / 3);
        let x = f32::from(idx % 3);

        let center = rect.min + Vec2::new((x + 0.5) * cell_w, (y + 0.5) * cell_h);
        let vs = vm.effective_visual_state(note_visual_state.digit_highlight(digit));
        let text_color = vs.text_color(false, visuals);
        if let Some(fill_color) = vs.note_fill_color(visuals) {
            let fill_rect =
                Rect::from_center_size(center, Vec2::splat(f32::min(cell_w, cell_h)) * 0.9);
            painter.rect_filled(fill_rect, 0.0, fill_color);
        }
        painter.text(
            center,
            Align2::CENTER_CENTER,
            digit.as_str(),
            note_font.clone(),
            text_color,
        );
    }
}
