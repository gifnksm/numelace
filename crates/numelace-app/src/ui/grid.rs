use std::sync::Arc;

use eframe::egui::{
    Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use numelace_core::{Digit, DigitSet, Position, PositionIndexedArray};
use numelace_game::CellState;

use crate::{
    action::{ActionRequestQueue, BoardMutationAction, SelectionAction},
    state::HighlightSettings,
    ui::{
        grid_theme::{GridPalette, GridTheme},
        input::InputContext,
        layout::{ComponentUnits, LayoutScale},
    },
};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct GridVisualState: u16 {
        const SELECTED_CELL = 0x0001;
        const SELECTED_DIGIT = 0x0002;
        const SELECTED_CELL_PEER = 0x0004;
        const SELECTED_DIGIT_PEER = 0x0008;
        const CONFLICT = 0x0010;
        const GHOST = 0x0020;
        const HINT_CONDITION_CELL = 0x0040;
        const HINT_CONDITION_DIGIT = 0x0080;
        const HINT_CONDITION_TEMPORARY = 0x0100;
        const HINT_APPLICATION_PLACEMENT = 0x0200;
        const HINT_APPLICATION_ELIMINATION = 0x0400;
        const HINT_APPLICATION_TEMPORARY = 0x0800;
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
    pub(crate) selected_digit: DigitSet,
    pub(crate) conflict: DigitSet,
    pub(crate) ghost: DigitSet,
    pub(crate) hint_condition_digit: DigitSet,
    pub(crate) hint_condition_temporary: DigitSet,
    pub(crate) hint_application_elimination: DigitSet,
    pub(crate) hint_application_temporary: DigitSet,
}

impl NoteVisualState {
    #[must_use]
    pub(crate) fn digit_highlight(&self, digit: Digit) -> GridVisualState {
        let Self {
            selected_digit,
            conflict,
            ghost,
            hint_condition_digit,
            hint_condition_temporary,
            hint_application_elimination,
            hint_application_temporary,
        } = self;
        let mut vs = GridVisualState::empty();
        if selected_digit.contains(digit) {
            vs |= GridVisualState::SELECTED_DIGIT;
        }
        if conflict.contains(digit) {
            vs |= GridVisualState::CONFLICT;
        }
        if ghost.contains(digit) {
            vs |= GridVisualState::GHOST;
        }
        if hint_condition_digit.contains(digit) {
            vs |= GridVisualState::HINT_CONDITION_DIGIT;
        }
        if hint_condition_temporary.contains(digit) {
            vs |= GridVisualState::HINT_CONDITION_TEMPORARY;
        }
        if hint_application_elimination.contains(digit) {
            vs |= GridVisualState::HINT_APPLICATION_ELIMINATION;
        }
        if hint_application_temporary.contains(digit) {
            vs |= GridVisualState::HINT_APPLICATION_TEMPORARY;
        }
        vs
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GridViewModel<'a> {
    grid: PositionIndexedArray<GridCell>,
    enabled_highlights: GridVisualState,
    input_context: &'a InputContext,
}

impl<'a> GridViewModel<'a> {
    #[must_use]
    pub(crate) fn new(
        grid: PositionIndexedArray<GridCell>,
        highlight_settings: &HighlightSettings,
        input_context: &'a InputContext,
    ) -> Self {
        let mut enabled_highlights = GridVisualState::SELECTED_CELL
            | GridVisualState::HINT_CONDITION_CELL
            | GridVisualState::HINT_CONDITION_DIGIT
            | GridVisualState::HINT_CONDITION_TEMPORARY
            | GridVisualState::HINT_APPLICATION_PLACEMENT
            | GridVisualState::HINT_APPLICATION_ELIMINATION
            | GridVisualState::HINT_APPLICATION_TEMPORARY;
        let HighlightSettings {
            selected_digit,
            selected_cell_peer,
            selected_digit_peer,
            conflict,
        } = highlight_settings;
        if *selected_digit_peer {
            enabled_highlights |= GridVisualState::SELECTED_DIGIT_PEER;
        }
        if *selected_cell_peer {
            enabled_highlights |= GridVisualState::SELECTED_CELL_PEER;
        }
        if *selected_digit {
            enabled_highlights |= GridVisualState::SELECTED_DIGIT;
        }
        if *conflict {
            enabled_highlights |= GridVisualState::CONFLICT;
        }
        Self {
            grid,
            enabled_highlights,
            input_context,
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
const SELECTED_CELL_BORDER_WIDTH_RATIO: f32 = 3.0;
const SELECTED_DIGIT_BORDER_WIDTH_RATIO: f32 = 1.0;
const SELECTED_CELL_PEER_BORDER_WIDTH_RATIO: f32 = 0.5;
const HINT_CORNER_WIDTH_RATIO: f32 = 3.0;

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
        if self.0.intersects(GridVisualState::SELECTED_DIGIT) {
            return palette.cell_bg_selected_digit;
        }
        if self.0.intersects(GridVisualState::SELECTED_DIGIT_PEER) {
            return palette.cell_bg_selected_digit_peer;
        }
        palette.cell_bg_default
    }

    fn note_fill_color(self, palette: &GridPalette) -> Option<Color32> {
        if self.0.intersects(GridVisualState::SELECTED_DIGIT) {
            return Some(palette.note_bg_selected_digit);
        }
        None
    }

    #[expect(clippy::unused_self)]
    fn cell_base_border_color(self, palette: &GridPalette) -> Color32 {
        palette.border_inactive
    }

    fn cell_overlay_border_color(self, palette: &GridPalette) -> Option<Color32> {
        if self.0.intersects(GridVisualState::SELECTED_CELL) {
            return Some(palette.border_selected_cell);
        }
        if self.0.intersects(GridVisualState::SELECTED_CELL_PEER) {
            return Some(palette.border_selected_cell_peer);
        }
        if self.0.intersects(GridVisualState::SELECTED_DIGIT) {
            return Some(palette.border_selected_digit);
        }
        None
    }

    #[expect(clippy::unused_self)]
    fn cell_base_border_width_ratio(self) -> f32 {
        THIN_BORDER_WIDTH_RATIO
    }

    fn cell_overlay_border_width_ratio(self) -> Option<f32> {
        if self.0.intersects(GridVisualState::SELECTED_CELL) {
            return Some(SELECTED_CELL_BORDER_WIDTH_RATIO);
        }
        if self.0.intersects(GridVisualState::SELECTED_DIGIT) {
            return Some(SELECTED_DIGIT_BORDER_WIDTH_RATIO);
        }
        if self.0.intersects(GridVisualState::SELECTED_CELL_PEER) {
            return Some(SELECTED_CELL_PEER_BORDER_WIDTH_RATIO);
        }
        None
    }

    fn cell_base_border(self, palette: &GridPalette, cell_size: f32) -> Stroke {
        let color = self.cell_base_border_color(palette);
        let ratio = self.cell_base_border_width_ratio();
        let base_width = f32::max(cell_size * CELL_BORDER_WIDTH_BASE_RATIO, 1.0);
        Stroke::new(base_width * ratio, color)
    }

    fn cell_overlay_border(self, palette: &GridPalette, cell_size: f32) -> Option<Stroke> {
        let color = self.cell_overlay_border_color(palette)?;
        let ratio = self.cell_overlay_border_width_ratio()?;
        let base_width = f32::max(cell_size * CELL_BORDER_WIDTH_BASE_RATIO, 1.0);
        Some(Stroke::new(base_width * ratio, color))
    }

    fn hint_corner_border(self, palette: &GridPalette, base_border: f32) -> Option<Stroke> {
        if self.0.intersects(GridVisualState::HINT_CONDITION_CELL) {
            return Some(Stroke::new(
                base_border * HINT_CORNER_WIDTH_RATIO,
                palette.border_hint_condition,
            ));
        }
        None
    }

    fn hint_digit_pill_color(self, palette: &GridPalette) -> Option<Color32> {
        if self.0.contains(GridVisualState::HINT_CONDITION_DIGIT) {
            return Some(palette.pill_hint);
        }
        None
    }

    fn cell_underline_stroke(self, rect: Rect, palette: &GridPalette) -> Option<Stroke> {
        if self
            .0
            .intersects(GridVisualState::HINT_APPLICATION_PLACEMENT)
        {
            return Some(Stroke::new(
                rect.height() * 0.1,
                palette.underline_hint_application,
            ));
        }
        None
    }

    fn note_underline_stroke(self, rect: Rect, palette: &GridPalette) -> Option<Stroke> {
        if self
            .0
            .intersects(GridVisualState::HINT_APPLICATION_TEMPORARY)
        {
            return Some(Stroke::new(
                rect.height() * 0.2,
                palette.underline_hint_application,
            ));
        }
        if self.0.intersects(GridVisualState::HINT_CONDITION_TEMPORARY) {
            return Some(Stroke::new(
                rect.height() * 0.2,
                palette.underline_hint_condition,
            ));
        }
        None
    }

    fn note_elimination_stroke(self, rect: Rect, palette: &GridPalette) -> Option<Stroke> {
        if self
            .0
            .intersects(GridVisualState::HINT_APPLICATION_ELIMINATION)
        {
            return Some(Stroke::new(rect.width() * 0.2, palette.elimination_stroke));
        }
        None
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

    for row in 0..9 {
        for col in 0..9 {
            let pos = Position::new(row, col);
            let cell = &vm.grid[pos];
            let vs = vm.effective_visual_state(cell.visual_state);

            let col_f = f32::from(col);
            let row_f = f32::from(row);
            let cell_min = inner_rect.min
                + Vec2::new(
                    cell_size * col_f + (col_f / 3.0).floor() * thick_border.width,
                    cell_size * row_f + (row_f / 3.0).floor() * thick_border.width,
                );
            let cell_max = cell_min + Vec2::splat(cell_size);
            let cell_rect = Rect::from_min_max(cell_min, cell_max);

            draw_cell_fill(painter, cell_rect, vs.cell_fill_color(palette));
            draw_cell_border(painter, cell_rect, vs.cell_base_border(palette, cell_size));
            if let Some(stroke) = vs.cell_overlay_border(palette, cell_size) {
                draw_cell_border(painter, cell_rect, stroke);
            }

            if let Some(stroke) = vs.hint_corner_border(palette, base_border) {
                draw_corners(painter, cell_rect, stroke);
            }

            if let Some(digits) = cell.content.as_notes() {
                let notes_rect = cell_rect.shrink(base_border * SELECTED_CELL_BORDER_WIDTH_RATIO);
                draw_notes(
                    painter,
                    vm,
                    notes_rect,
                    digits,
                    &cell.note_visual_state,
                    palette,
                );
            } else if let Some(digit) = cell.content.as_digit() {
                if let Some(color) = vs.hint_digit_pill_color(palette) {
                    draw_digit_pill(painter, cell_rect.center(), cell_size, color);
                }
                draw_cell_digit(
                    painter,
                    cell_rect.center(),
                    cell_size,
                    digit,
                    vs.text_color(cell.content.is_given(), palette),
                );
                let digit_rect = cell_rect.shrink(base_border);
                if let Some(stroke) = vs.cell_underline_stroke(digit_rect, palette) {
                    let offset = digit_rect.height() * 0.15;
                    let y = digit_rect.bottom() - stroke.width;
                    let start = Pos2::new(digit_rect.left() + offset, y);
                    let end = Pos2::new(digit_rect.right() - offset, y);
                    painter.line_segment([start, end], stroke);
                }
            }

            let response = ui.interact(cell_rect, ui.id().with((col, row)), Sense::click());
            if response.secondary_clicked() {
                action_queue.request(
                    BoardMutationAction::RequestDigit {
                        digit: None,
                        swap_input_mode: vm.input_context.swap_input_mode,
                        position: Some(pos),
                    }
                    .into(),
                );
            } else if response.double_clicked() {
                action_queue.request(
                    BoardMutationAction::AdvanceCell {
                        position: Some(pos),
                    }
                    .into(),
                );
            } else if response.clicked() {
                action_queue.request(SelectionAction::SelectOrClearCell(pos).into());
            }
        }
    }

    draw_box_borders(painter, inner_rect, cell_size, thick_border);
}

fn draw_cell_fill(painter: &Painter, rect: Rect, color: Color32) {
    painter.rect_filled(rect, 0.0, color);
}

fn draw_cell_border(painter: &Painter, rect: Rect, stroke: Stroke) {
    painter.rect_stroke(rect, 0.0, stroke, StrokeKind::Inside);
}

fn draw_cell_digit(painter: &Painter, center: Pos2, cell_size: f32, digit: Digit, color: Color32) {
    painter.text(
        center,
        Align2::CENTER_CENTER,
        digit.as_str(),
        FontId::proportional(cell_size * 0.8),
        color,
    );
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

fn draw_digit_pill(painter: &Painter, center: Pos2, cell_size: f32, color: Color32) {
    let radius = cell_size * 0.55 * 0.5;
    painter.circle_filled(center, radius, color);
}

fn draw_corners(painter: &Painter, rect: Rect, stroke: Stroke) {
    let corner_len = rect.width().min(rect.height()) * 0.25;
    let thickness = stroke.width.max(1.0);
    let min = rect.min;
    let max = rect.max;

    let top_left_h = Rect::from_min_size(min, Vec2::new(corner_len, thickness));
    let top_left_v = Rect::from_min_size(min, Vec2::new(thickness, corner_len));

    let top_right_h = Rect::from_min_size(
        Pos2::new(max.x - corner_len, min.y),
        Vec2::new(corner_len, thickness),
    );
    let top_right_v = Rect::from_min_size(
        Pos2::new(max.x - thickness, min.y),
        Vec2::new(thickness, corner_len),
    );

    let bottom_left_h = Rect::from_min_size(
        Pos2::new(min.x, max.y - thickness),
        Vec2::new(corner_len, thickness),
    );
    let bottom_left_v = Rect::from_min_size(
        Pos2::new(min.x, max.y - corner_len),
        Vec2::new(thickness, corner_len),
    );

    let bottom_right_h = Rect::from_min_size(
        Pos2::new(max.x - corner_len, max.y - thickness),
        Vec2::new(corner_len, thickness),
    );
    let bottom_right_v = Rect::from_min_size(
        Pos2::new(max.x - thickness, max.y - corner_len),
        Vec2::new(thickness, corner_len),
    );

    painter.rect_filled(top_left_h, 0.0, stroke.color);
    painter.rect_filled(top_left_v, 0.0, stroke.color);
    painter.rect_filled(top_right_h, 0.0, stroke.color);
    painter.rect_filled(top_right_v, 0.0, stroke.color);
    painter.rect_filled(bottom_left_h, 0.0, stroke.color);
    painter.rect_filled(bottom_left_v, 0.0, stroke.color);
    painter.rect_filled(bottom_right_h, 0.0, stroke.color);
    painter.rect_filled(bottom_right_v, 0.0, stroke.color);
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
        let fill_rect = Rect::from_center_size(center, Vec2::splat(f32::min(cell_w, cell_h)) * 0.9);
        if let Some(fill_color) = vs.note_fill_color(palette) {
            painter.rect_filled(fill_rect, 0.0, fill_color);
        }
        if let Some(stroke) = vs.note_underline_stroke(fill_rect, palette) {
            let y = fill_rect.bottom() - stroke.width / 2.0;
            let start = Pos2::new(fill_rect.left(), y);
            let end = Pos2::new(fill_rect.right(), y);
            painter.line_segment([start, end], stroke);
        }
        if let Some(pill_color) = vs.hint_digit_pill_color(palette) {
            let pill_radius = f32::min(cell_w, cell_h) * 0.8 * 0.5;
            painter.circle_filled(center, pill_radius, pill_color);
        }
        painter.text(
            center,
            Align2::CENTER_CENTER,
            digit.as_str(),
            note_font.clone(),
            text_color,
        );
        if let Some(stroke) = vs.note_elimination_stroke(fill_rect, palette) {
            let offset = fill_rect.width() * 0.15;
            let start = Pos2::new(fill_rect.left() + offset, fill_rect.top() + offset);
            let end = Pos2::new(fill_rect.right() - offset, fill_rect.bottom() - offset);
            painter.line_segment([start, end], stroke);
        }
    }
}
