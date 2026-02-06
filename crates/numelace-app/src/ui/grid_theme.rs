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
    #[must_use]
    pub fn from_visuals(visuals: &Visuals) -> Self {
        // Palette guidance:
        // - Prefer meaning-based colors that do not overlap with existing semantics.
        // - Keep light/dark modes on the same semantic hue, adjust luminance only.
        // - Keep backgrounds subtle so digits/notes stay primary.
        // - For visuals-derived values, keep a single value comment at the source line.
        let cell_bg_selected = visuals.selection.bg_fill; // dark=(0, 92, 128) light=(144, 209, 255)
        let (cell_bg_house_selected, cell_bg_house_same_digit, note_bg_house_same_digit) =
            if visuals.dark_mode {
                (
                    Color32::from_gray(80), // base: widgets.hovered.bg_fill = (70, 70, 70) (dark)
                    Color32::from_gray(45), // tuned from widgets.hovered.bg_fill (dark), higher contrast
                    Color32::from_gray(45), // tuned from widgets.hovered.bg_fill (dark), higher contrast
                )
            } else {
                (
                    Color32::from_gray(170), // tuned from widgets.hovered.bg_fill (light), higher contrast
                    Color32::from_gray(210), // base: widgets.hovered.bg_fill = (220, 220, 220) (light)
                    Color32::from_gray(210), // base: widgets.hovered.bg_fill = (220, 220, 220) (light)
                )
            };

        Self {
            cell_bg_default: visuals.text_edit_bg_color(), // dark=(10, 10, 10) light=(255, 255, 255)
            cell_bg_selected,
            cell_bg_same_digit: cell_bg_selected,
            cell_bg_house_selected,
            cell_bg_house_same_digit,

            note_bg_same_digit: cell_bg_selected,
            note_bg_house_same_digit,

            border_inactive: visuals.widgets.inactive.fg_stroke.color, // dark=(180, 180, 180) light=(60, 60, 60)
            border_selected: visuals.selection.stroke.color, // dark=(192, 222, 255) light=(0, 83, 125)
            border_same_digit: visuals.selection.stroke.color,
            border_conflict: visuals.error_fg_color, // dark/light=(255, 0, 0)

            text_normal: visuals.text_color(), // dark=(140, 140, 140) light=(80, 80, 80)
            text_given: visuals.strong_text_color(), // dark=(255, 255, 255) light=(0, 0, 0)
            text_conflict: visuals.error_fg_color, // dark/light=(255, 0, 0)
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
    #[must_use]
    pub fn from_visuals(visuals: &Visuals) -> Self {
        let palette = GridPalette::from_visuals(visuals);
        Self {
            light: palette.clone(),
            dark: palette,
        }
    }

    #[must_use]
    pub fn palette_for(&self, visuals: &Visuals) -> &GridPalette {
        if visuals.dark_mode {
            &self.dark
        } else {
            &self.light
        }
    }
}
