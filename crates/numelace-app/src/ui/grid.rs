use std::sync::Arc;

use eframe::egui::{
    Align2, Button, Color32, FontId, Grid, Painter, Rect, RichText, Stroke, StrokeKind, Ui, Vec2,
    Visuals,
};
use numelace_core::{Digit, DigitSet, Position, containers::Array81, index::PositionSemantics};
use numelace_game::CellState;

use crate::{state::HighlightSettings, ui::Action};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CellVisualState: u8 {
        const SELECTED = 0b0000_0001;
        const SAME_DIGIT = 0b0000_0010;
        const HOUSE_SELECTED = 0b0000_0100;
        const HOUSE_SAME_DIGIT = 0b0000_1000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCell {
    pub content: CellState,
    pub visual_state: CellVisualState,
}

#[derive(Debug, Clone)]
pub struct GridViewModel<'a> {
    grid: &'a Array81<GridCell, PositionSemantics>,
    selected_digit: Option<Digit>,
    highlight_settings: &'a HighlightSettings,
}

impl<'a> GridViewModel<'a> {
    pub fn new(
        grid: &'a Array81<GridCell, PositionSemantics>,
        selected_digit: Option<Digit>,
        highlight_settings: &'a HighlightSettings,
    ) -> Self {
        Self {
            grid,
            selected_digit,
            highlight_settings,
        }
    }

    fn cell_highlight(&self, cell_pos: Position) -> CellHighlight {
        let hlc = &self.highlight_settings;
        let vs = self.grid[cell_pos].visual_state;

        if vs.contains(CellVisualState::SELECTED) {
            return CellHighlight::Selected;
        }
        if hlc.same_digit && vs.contains(CellVisualState::SAME_DIGIT) {
            return CellHighlight::SameDigit;
        }
        if hlc.house_selected && vs.contains(CellVisualState::HOUSE_SELECTED) {
            return CellHighlight::HouseSelected;
        }
        if hlc.house_same_digit && vs.contains(CellVisualState::HOUSE_SAME_DIGIT) {
            return CellHighlight::HouseSameDigit;
        }
        CellHighlight::None
    }

    fn cell_text(&self, pos: Position, visuals: &Visuals) -> RichText {
        match self.grid[pos].content {
            CellState::Given(digit) => {
                RichText::new(digit.as_str()).color(visuals.strong_text_color())
            }
            CellState::Filled(digit) => RichText::new(digit.as_str()).color(visuals.text_color()),
            CellState::Notes(_) | CellState::Empty => RichText::new(""),
        }
    }

    fn note_text_color(visuals: &Visuals) -> Color32 {
        visuals.text_color()
    }

    fn inactive_border_color(visuals: &Visuals) -> Color32 {
        visuals.widgets.inactive.fg_stroke.color
    }

    fn grid_thick_border(visuals: &Visuals, cell_size: f32) -> Stroke {
        let base_width = f32::max(cell_size * CELL_BORDER_BASE_RATIO, 1.0);
        Stroke::new(
            base_width * THICK_BORDER_RATIO,
            Self::inactive_border_color(visuals),
        )
    }
}

const CELL_BORDER_BASE_RATIO: f32 = 0.03;
const THICK_BORDER_RATIO: f32 = 2.0;
const THIN_BORDER_RATIO: f32 = 1.0;
const SELECTED_BORDER_RATIO: f32 = 3.0;
const SAME_DIGIT_BORDER_RATIO: f32 = 1.0;
const HOUSE_BORDER_RATIO: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellHighlight {
    Selected,
    SameDigit,
    HouseSelected,
    HouseSameDigit,
    None,
}

impl CellHighlight {
    fn fill_color(self, visuals: &Visuals) -> Color32 {
        match self {
            Self::Selected | Self::SameDigit => visuals.selection.bg_fill,
            Self::HouseSelected | Self::HouseSameDigit => visuals.widgets.hovered.bg_fill,
            Self::None => visuals.text_edit_bg_color(),
        }
    }

    fn border(self, visuals: &Visuals, cell_size: f32) -> Stroke {
        let (ratio, color) = match self {
            Self::Selected => (SELECTED_BORDER_RATIO, visuals.selection.stroke.color),
            Self::SameDigit => (SAME_DIGIT_BORDER_RATIO, visuals.selection.stroke.color),
            Self::HouseSelected | Self::HouseSameDigit => (
                HOUSE_BORDER_RATIO,
                GridViewModel::inactive_border_color(visuals),
            ),
            Self::None => (
                THIN_BORDER_RATIO,
                GridViewModel::inactive_border_color(visuals),
            ),
        };
        let base_width = f32::max(cell_size * CELL_BORDER_BASE_RATIO, 1.0);
        Stroke::new(base_width * ratio, color)
    }
}

pub fn show(ui: &mut Ui, vm: &GridViewModel<'_>) -> Vec<Action> {
    let mut actions = vec![];

    let style = Arc::clone(ui.style());
    let visuals = &style.visuals;

    let grid_size = ui.available_size().min_elem();
    // - 9 cells across
    // - 2 outer borders + 2 inner 3x3 borders (total 4 border widths)
    // This keeps the 3x3 separator thickness consistent with the outer border.
    let cell_size = grid_size / (9.0 + 4.0 * CELL_BORDER_BASE_RATIO * THICK_BORDER_RATIO);
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
                                    let text = vm.cell_text(pos, visuals).size(cell_size * 0.8);
                                    let highlight = vm.cell_highlight(pos);
                                    let button = Button::new(text)
                                        .min_size(Vec2::splat(cell_size))
                                        .fill(highlight.fill_color(visuals));
                                    let button = ui.add(button);
                                    if let Some(digits) = vm.grid[pos].content.as_notes() {
                                        draw_notes(
                                            ui.painter(),
                                            vm,
                                            button.rect.shrink(thick_border.width),
                                            digits,
                                            visuals,
                                        );
                                    }
                                    ui.painter().rect_stroke(
                                        button.rect,
                                        0.0,
                                        highlight.border(visuals, cell_size),
                                        StrokeKind::Inside,
                                    );
                                    if button.clicked() {
                                        actions.push(Action::SelectCell(pos));
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

    actions
}

fn draw_notes(
    painter: &Painter,
    vm: &GridViewModel,
    rect: Rect,
    digits: DigitSet,
    visuals: &Visuals,
) {
    let note_font = FontId::proportional(rect.height() / 3.0);
    let color = GridViewModel::note_text_color(visuals);

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

        if vm.highlight_settings.same_digit && Some(digit) == vm.selected_digit {
            let highlight_rect =
                Rect::from_center_size(center, Vec2::splat(f32::min(cell_w, cell_h)) * 0.9);
            painter.rect_filled(highlight_rect, 0.0, visuals.selection.bg_fill);
        }

        painter.text(
            center,
            Align2::CENTER_CENTER,
            digit.as_str(),
            note_font.clone(),
            color,
        );
    }
}
