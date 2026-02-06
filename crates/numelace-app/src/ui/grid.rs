use std::sync::Arc;

use eframe::egui::{
    Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use numelace_core::{Digit, DigitSet, Position, containers::Array81, index::PositionSemantics};
use numelace_game::CellState;

use crate::{
    action::{Action, ActionRequestQueue},
    state::HighlightSettings,
    ui::{
        grid_theme::{GridPalette, GridTheme},
        layout::{ComponentUnits, LayoutScale},
    },
};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct GridVisualState: u8 {
        const SELECTED = 0b0000_0001;
        const SAME_DIGIT = 0b0000_0010;
        const HOUSE_SELECTED = 0b0000_0100;
        const HOUSE_SAME_DIGIT = 0b0000_1000;
        const CONFLICT = 0b0001_0000;
        const GHOST = 0b0010_0000;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GridCell {
    pub(crate) content: CellState,
    pub(crate) visual_state: GridVisualState,
    pub(crate) note_visual_state: NoteVisualState,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(crate) struct NoteVisualState {
    pub(crate) same_digit: DigitSet,
    pub(crate) conflict: DigitSet,
    pub(crate) ghost: DigitSet,
}

impl NoteVisualState {
    #[must_use]
    pub(crate) fn digit_highlight(&self, digit: Digit) -> GridVisualState {
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
pub(crate) struct GridViewModel {
    grid: Array81<GridCell, PositionSemantics>,
    enabled_highlights: GridVisualState,
}

impl GridViewModel {
    #[must_use]
    pub(crate) fn new(
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

    fn grid_thick_border(palette: &GridPalette, cell_size: f32) -> Stroke {
        let base_width = f32::max(cell_size * CELL_BORDER_WIDTH_BASE_RATIO, 1.0);
        Stroke::new(
            base_width * THICK_BORDER_WIDTH_RATIO,
            palette.border_inactive,
        )
    }

    fn effective_visual_state(&self, state: GridVisualState) -> EffectiveGridVisualState {
        EffectiveGridVisualState(self.enabled_highlights & state)
    }
}

pub(crate) const GRID_CELLS: f32 = 9.0;

#[must_use]
pub(crate) fn grid_side_with_border(cell_size: f32) -> f32 {
    let thick_border = thick_border_width(cell_size);
    GRID_CELLS * cell_size + thick_border * 4.0
}

#[must_use]
pub(crate) const fn required_units() -> ComponentUnits {
    let len = GRID_CELLS + CELL_BORDER_WIDTH_BASE_RATIO * (THICK_BORDER_WIDTH_RATIO * 4.0);
    ComponentUnits::new(len, len)
}

fn thick_border_width(cell_size: f32) -> f32 {
    let base_width = f32::max(cell_size * CELL_BORDER_WIDTH_BASE_RATIO, 1.0);
    base_width * THICK_BORDER_WIDTH_RATIO
}

const CELL_BORDER_WIDTH_BASE_RATIO: f32 = 0.03;
const THICK_BORDER_WIDTH_RATIO: f32 = 3.0;
const THIN_BORDER_WIDTH_RATIO: f32 = 1.0;
const SELECTED_BORDER_WIDTH_RATIO: f32 = 3.0;
const SAME_DIGIT_BORDER_WIDTH_RATIO: f32 = 1.0;
const HOUSE_BORDER_WIDTH_RATIO: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EffectiveGridVisualState(GridVisualState);

impl EffectiveGridVisualState {
    fn text_color(self, is_given: bool, palette: &GridPalette) -> Color32 {
        if self.0.intersects(GridVisualState::CONFLICT) {
            return palette.text_conflict;
        }
        if is_given {
            palette.text_given
        } else {
            palette.text_normal
        }
    }

    fn cell_fill_color(self, palette: &GridPalette) -> Color32 {
        if self.0.intersects(GridVisualState::SELECTED) {
            return palette.cell_bg_selected;
        }
        if self.0.intersects(GridVisualState::SAME_DIGIT) {
            return palette.cell_bg_same_digit;
        }
        if self.0.intersects(GridVisualState::HOUSE_SELECTED) {
            return palette.cell_bg_house_selected;
        }
        if self.0.intersects(GridVisualState::HOUSE_SAME_DIGIT) {
            return palette.cell_bg_house_same_digit;
        }
        palette.cell_bg_default
    }

    fn note_fill_color(self, palette: &GridPalette) -> Option<Color32> {
        if self.0.intersects(GridVisualState::SAME_DIGIT) {
            return Some(palette.note_bg_same_digit);
        }
        if self.0.intersects(GridVisualState::HOUSE_SAME_DIGIT) {
            return Some(palette.note_bg_house_same_digit);
        }
        None
    }

    fn cell_border_color(self, palette: &GridPalette) -> Color32 {
        if self.0.intersects(GridVisualState::CONFLICT) {
            return palette.border_conflict;
        }
        if self.0.intersects(GridVisualState::SELECTED) {
            return palette.border_selected;
        }
        if self.0.intersects(GridVisualState::SAME_DIGIT) {
            return palette.border_same_digit;
        }
        palette.border_inactive
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

    fn cell_border(self, palette: &GridPalette, cell_size: f32) -> Stroke {
        let color = self.cell_border_color(palette);
        let ratio = self.cell_border_width_ratio();
        let base_width = f32::max(cell_size * CELL_BORDER_WIDTH_BASE_RATIO, 1.0);
        Stroke::new(base_width * ratio, color)
    }
}

pub(crate) fn show(
    ui: &mut Ui,
    vm: &GridViewModel,
    scale: &LayoutScale,
    action_queue: &mut ActionRequestQueue,
) {
    let cell_size = scale.cell_size;
    let style = Arc::clone(ui.style());
    let visuals = &style.visuals;
    let grid_theme = GridTheme::from_visuals(visuals);
    let palette = grid_theme.palette_for(visuals);
    let grid_side = grid_side_with_border(cell_size);

    let (rect, _response) = ui.allocate_exact_size(Vec2::splat(grid_side), Sense::hover());

    let thick_border = GridViewModel::grid_thick_border(palette, cell_size);
    let base_border = f32::max(cell_size * CELL_BORDER_WIDTH_BASE_RATIO, 1.0);
    let inner_rect = rect.shrink(thick_border.width);

    let painter = ui.painter();
    draw_outer_border(painter, rect, thick_border);

    for y in 0..9 {
        for x in 0..9 {
            let pos = Position::new(x, y);
            let cell = &vm.grid[pos];
            let vs = vm.effective_visual_state(cell.visual_state);

            let xf = f32::from(x);
            let yf = f32::from(y);
            let cell_min = inner_rect.min
                + Vec2::new(
                    cell_size * xf + (xf / 3.0).floor() * thick_border.width,
                    cell_size * yf + (yf / 3.0).floor() * thick_border.width,
                );
            let cell_max = cell_min + Vec2::splat(cell_size);
            let cell_rect = Rect::from_min_max(cell_min, cell_max);

            painter.rect_filled(cell_rect, 0.0, vs.cell_fill_color(palette));

            if let Some(digits) = cell.content.as_notes() {
                let notes_rect = cell_rect.shrink(base_border);
                draw_notes(
                    painter,
                    vm,
                    notes_rect,
                    digits,
                    &cell.note_visual_state,
                    palette,
                );
            } else if let Some(digit) = cell.content.as_digit() {
                painter.text(
                    cell_rect.center(),
                    Align2::CENTER_CENTER,
                    digit.as_str(),
                    FontId::proportional(cell_size * 0.8),
                    vs.text_color(cell.content.is_given(), palette),
                );
            }

            painter.rect_stroke(
                cell_rect,
                0.0,
                vs.cell_border(palette, cell_size),
                StrokeKind::Inside,
            );

            let response = ui.interact(cell_rect, ui.id().with((x, y)), Sense::click());
            if response.clicked() {
                action_queue.request(Action::SelectCell(pos));
            }
        }
    }

    draw_box_borders(painter, inner_rect, cell_size, thick_border);
}

fn draw_outer_border(painter: &Painter, rect: Rect, stroke: Stroke) {
    let thickness = stroke.width.max(1.0);

    let left = Rect::from_min_max(
        Pos2::new(rect.left(), rect.top()),
        Pos2::new(rect.left() + thickness, rect.bottom()),
    );
    let right = Rect::from_min_max(
        Pos2::new(rect.right() - thickness, rect.top()),
        Pos2::new(rect.right(), rect.bottom()),
    );
    let top = Rect::from_min_max(
        Pos2::new(rect.left(), rect.top()),
        Pos2::new(rect.right(), rect.top() + thickness),
    );
    let bottom = Rect::from_min_max(
        Pos2::new(rect.left(), rect.bottom() - thickness),
        Pos2::new(rect.right(), rect.bottom()),
    );

    painter.rect_filled(left, 0.0, stroke.color);
    painter.rect_filled(right, 0.0, stroke.color);
    painter.rect_filled(top, 0.0, stroke.color);
    painter.rect_filled(bottom, 0.0, stroke.color);
}

fn draw_box_borders(painter: &Painter, inner_rect: Rect, cell_size: f32, stroke: Stroke) {
    let start = inner_rect.min;
    let end = inner_rect.max;
    let thickness = stroke.width.max(1.0);
    let half = thickness * 0.5;

    for i in [1.0, 2.0] {
        let offset = cell_size * 3.0 * i + thickness * (i - 0.5);
        let x = start.x + offset;
        let v_rect = Rect::from_min_max(Pos2::new(x - half, start.y), Pos2::new(x + half, end.y));
        painter.rect_filled(v_rect, 0.0, stroke.color);

        let y = start.y + offset;
        let h_rect = Rect::from_min_max(Pos2::new(start.x, y - half), Pos2::new(end.x, y + half));
        painter.rect_filled(h_rect, 0.0, stroke.color);
    }
}

fn draw_notes(
    painter: &Painter,
    vm: &GridViewModel,
    rect: Rect,
    digits: DigitSet,
    note_visual_state: &NoteVisualState,
    palette: &GridPalette,
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
        let text_color = vs.text_color(false, palette);
        if let Some(fill_color) = vs.note_fill_color(palette) {
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
