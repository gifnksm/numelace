use eframe::egui::{Color32, Visuals};

/// Color palette for Sudoku grid rendering.
///
/// This is intentionally independent from `egui::Visuals` so grid-specific
/// semantics (selection, house highlight, conflicts, hints) can be tuned
/// without being constrained by the global UI theme.
#[derive(Debug, Clone)]
pub struct GridPalette {
    pub cell_bg_default: Color32,
    pub cell_bg_selected: Color32,
    pub cell_bg_same_digit: Color32,
    pub cell_bg_house_selected: Color32,
    pub cell_bg_house_same_digit: Color32,

    pub note_bg_same_digit: Color32,
    pub note_bg_house_same_digit: Color32,

    pub border_inactive: Color32,
    pub border_selected: Color32,
    pub border_same_digit: Color32,
    pub border_conflict: Color32,

    pub text_normal: Color32,
    pub text_given: Color32,
    pub text_conflict: Color32,
}

impl GridPalette {
    /// Initialize the palette using the current visuals.
    ///
    /// This keeps behavior identical to the current visuals-based colors,
    /// while making the palette structure explicit for later customization.
    pub fn from_visuals(visuals: &Visuals) -> Self {
        let cell_bg_selected = visuals.selection.bg_fill;
        let cell_bg_house = visuals.widgets.hovered.bg_fill;

        Self {
            cell_bg_default: visuals.text_edit_bg_color(),
            cell_bg_selected,
            cell_bg_same_digit: cell_bg_selected,
            cell_bg_house_selected: cell_bg_house,
            cell_bg_house_same_digit: cell_bg_house,

            note_bg_same_digit: cell_bg_selected,
            note_bg_house_same_digit: cell_bg_house,

            border_inactive: visuals.widgets.inactive.fg_stroke.color,
            border_selected: visuals.selection.stroke.color,
            border_same_digit: visuals.selection.stroke.color,
            border_conflict: visuals.error_fg_color,

            text_normal: visuals.text_color(),
            text_given: visuals.strong_text_color(),
            text_conflict: visuals.error_fg_color,
        }
    }
}

/// Holds light/dark palettes and selects one based on current visuals.
#[derive(Debug, Clone)]
pub struct GridTheme {
    pub light: GridPalette,
    pub dark: GridPalette,
}

impl GridTheme {
    /// Create a theme using the current visuals for both palettes.
    ///
    /// This preserves existing colors today while allowing later divergence.
    pub fn from_visuals(visuals: &Visuals) -> Self {
        let palette = GridPalette::from_visuals(visuals);
        Self {
            light: palette.clone(),
            dark: palette,
        }
    }

    pub fn palette_for(&self, visuals: &Visuals) -> &GridPalette {
        if visuals.dark_mode {
            &self.dark
        } else {
            &self.light
        }
    }
}
