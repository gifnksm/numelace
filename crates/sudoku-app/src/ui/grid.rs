use std::sync::Arc;

use eframe::egui::{Button, Color32, Grid, RichText, Stroke, StrokeKind, Ui, Vec2, Visuals};
use sudoku_core::{Digit, Position};
use sudoku_game::{CellState, Game};

use crate::ui::Action;

#[derive(Debug, Clone)]
pub struct GridViewModel<'a> {
    game: &'a Game,
    selected_cell: Option<Position>,
    selected_digit: Option<Digit>,
}

impl<'a> GridViewModel<'a> {
    pub fn new(
        game: &'a Game,
        selected_cell: Option<Position>,
        selected_digit: Option<Digit>,
    ) -> Self {
        Self {
            game,
            selected_cell,
            selected_digit,
        }
    }

    fn cell_highlight(&self, cell_pos: Position) -> CellHighlight {
        let cell_digit = self.game.cell(cell_pos).as_digit();
        if Some(cell_pos) == self.selected_cell {
            CellHighlight::Selected
        } else if self.selected_digit.is_some_and(|d| Some(d) == cell_digit) {
            CellHighlight::SameDigit
        } else if self
            .selected_cell
            .is_some_and(|p| is_same_home(p, cell_pos))
        {
            CellHighlight::SameHome
        } else {
            CellHighlight::None
        }
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

    fn grid_thick_border(visuals: &Visuals) -> Stroke {
        Stroke::new(3.0, Self::inactive_border_color(visuals))
    }
}

fn is_same_home(pos1: Position, pos2: Position) -> bool {
    pos1.x() == pos2.x() || pos1.y() == pos2.y() || pos1.box_index() == pos2.box_index()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellHighlight {
    Selected,
    SameDigit,
    SameHome,
    None,
}

impl CellHighlight {
    fn fill_color(self, visuals: &Visuals) -> Color32 {
        match self {
            Self::Selected | CellHighlight::SameDigit => visuals.selection.bg_fill,
            Self::SameHome => visuals.widgets.hovered.bg_fill,
            Self::None => visuals.text_edit_bg_color(),
        }
    }

    fn border(self, visuals: &Visuals) -> Stroke {
        match self {
            Self::Selected => Stroke::new(6.0, visuals.selection.stroke.color),
            Self::SameDigit => Stroke::new(2.0, visuals.selection.stroke.color),
            Self::SameHome => Stroke::new(1.5, visuals.widgets.hovered.fg_stroke.color),
            Self::None => Stroke::new(1.0, GridViewModel::inactive_border_color(visuals)),
        }
    }
}

pub fn show(ui: &mut Ui, vm: &GridViewModel<'_>) -> Vec<Action> {
    let mut actions = vec![];

    let style = Arc::clone(ui.style());
    let visuals = &style.visuals;
    let thick_border = GridViewModel::grid_thick_border(visuals);

    let grid_size = ui.available_size().min_elem();
    let cell_size = grid_size / 9.0;

    Grid::new(ui.id().with("outer_board"))
        .spacing((0.0, 0.0))
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
                                        highlight.border(visuals),
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
                        StrokeKind::Inside,
                    );
                }
                ui.end_row();
            }
        });

    actions
}
