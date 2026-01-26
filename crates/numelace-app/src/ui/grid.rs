use std::sync::Arc;

use eframe::egui::{Button, Color32, Grid, RichText, Stroke, StrokeKind, Ui, Vec2, Visuals};
use numelace_core::{Digit, Position};
use numelace_game::{CellState, Game};

use crate::{app::HighlightConfig, ui::Action};

#[derive(Debug, Clone)]
pub struct GridViewModel<'a> {
    game: &'a Game,
    selected_cell: Option<Position>,
    selected_digit: Option<Digit>,
    highlight_config: &'a HighlightConfig,
}

impl<'a> GridViewModel<'a> {
    pub fn new(
        game: &'a Game,
        selected_cell: Option<Position>,
        selected_digit: Option<Digit>,
        highlight_config: &'a HighlightConfig,
    ) -> Self {
        Self {
            game,
            selected_cell,
            selected_digit,
            highlight_config,
        }
    }

    fn cell_highlight(&self, cell_pos: Position) -> CellHighlight {
        let cell_digit = self.game.cell(cell_pos).as_digit();
        if Some(cell_pos) == self.selected_cell {
            return CellHighlight::Selected;
        }

        let hlc = &self.highlight_config;
        if hlc.same_digit && self.selected_digit.is_some_and(|d| Some(d) == cell_digit) {
            return CellHighlight::SameDigit;
        }

        if hlc.rcb_selected
            && self
                .selected_cell
                .is_some_and(|p| is_same_home(p, cell_pos))
        {
            return CellHighlight::RcbSelected;
        }

        if hlc.rcb_same_digit
            && self.selected_digit.is_some_and(|d| {
                Position::ROWS[cell_pos.y()]
                    .into_iter()
                    .chain(Position::COLUMNS[cell_pos.x()])
                    .chain(Position::BOXES[cell_pos.box_index()])
                    .any(|p| self.game.cell(p).as_digit() == Some(d))
            })
        {
            return CellHighlight::RcbSameDigit;
        }

        CellHighlight::None
    }

    fn cell_text(&self, pos: Position, visuals: &Visuals) -> RichText {
        match self.game.cell(pos) {
            CellState::Given(digit) => {
                RichText::new(digit.as_str()).color(visuals.strong_text_color())
            }
            CellState::Filled(digit) => RichText::new(digit.as_str()).color(visuals.text_color()),
            CellState::Empty => RichText::new(""),
        }
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

fn is_same_home(pos1: Position, pos2: Position) -> bool {
    pos1.x() == pos2.x() || pos1.y() == pos2.y() || pos1.box_index() == pos2.box_index()
}

const CELL_BORDER_BASE_RATIO: f32 = 0.03;
const THICK_BORDER_RATIO: f32 = 2.0;
const THIN_BORDER_RATIO: f32 = 1.0;
const SELECTED_BORDER_RATIO: f32 = 3.0;
const SAME_DIGIT_BORDER_RATIO: f32 = 1.0;
const RCB_BORDER_RATIO: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellHighlight {
    Selected,
    SameDigit,
    RcbSelected,
    RcbSameDigit,
    None,
}

impl CellHighlight {
    fn fill_color(self, visuals: &Visuals) -> Color32 {
        match self {
            Self::Selected | Self::SameDigit => visuals.selection.bg_fill,
            Self::RcbSelected | Self::RcbSameDigit => visuals.widgets.hovered.bg_fill,
            Self::None => visuals.text_edit_bg_color(),
        }
    }

    fn border(self, visuals: &Visuals, cell_size: f32) -> Stroke {
        let (ratio, color) = match self {
            Self::Selected => (SELECTED_BORDER_RATIO, visuals.selection.stroke.color),
            Self::SameDigit => (SAME_DIGIT_BORDER_RATIO, visuals.selection.stroke.color),
            Self::RcbSelected | Self::RcbSameDigit => (
                RCB_BORDER_RATIO,
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
