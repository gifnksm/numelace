use eframe::egui::{Color32, Visuals};

/// Color palette for Sudoku grid rendering.
///
/// This is intentionally independent from `egui::Visuals` so grid-specific
/// semantics (selection, house highlight, conflicts, hints) can be tuned
/// without being constrained by the global UI theme.
#[derive(Debug, Clone)]
pub(crate) struct GridPalette {
    pub(crate) cell_bg_default: Color32,
    pub(crate) cell_bg_selected_digit: Color32,
    pub(crate) cell_bg_selected_digit_peer: Color32,

    pub(crate) note_bg_selected_digit: Color32,

    pub(crate) pill_hint: Color32,

    pub(crate) border_inactive: Color32,
    pub(crate) border_selected_cell: Color32,
    pub(crate) border_selected_cell_peer: Color32,
    pub(crate) border_selected_digit: Color32,
    pub(crate) border_hint_condition: Color32,

    pub(crate) underline_hint_condition: Color32,
    pub(crate) underline_hint_application: Color32,

    pub(crate) elimination_stroke: Color32,

    pub(crate) text_normal: Color32,
    pub(crate) text_given: Color32,
    pub(crate) text_conflict: Color32,
}

impl GridPalette {
    /// Initialize the palette using the current visuals.
    ///
    /// This keeps behavior identical to the current visuals-based colors,
    /// while making the palette structure explicit for later customization.
    #[must_use]
    pub(crate) fn from_visuals(visuals: &Visuals) -> Self {
        // Palette guidance:
        // - Prefer meaning-based colors that do not overlap with existing semantics.
        // - Keep light/dark modes on the same semantic hue, adjust luminance only.
        // - Keep backgrounds subtle so digits/notes stay primary.
        // - For visuals-derived values, keep a single value comment at the source line.
        let cell_bg_selected_digit = visuals.selection.bg_fill; // dark=(0, 92, 128) light=(144, 209, 255)
        let cell_bg_selected_digit_peer = if visuals.dark_mode {
            //  widgets.hovered.bg_fill = (70, 70, 70) (dark)
            Color32::from_gray(70)
        } else {
            // widgets.hovered.bg_fill = (220, 220, 220) (light)
            Color32::from_gray(220)
        };
        let hint_accent = if visuals.dark_mode {
            Color32::from_rgb(255, 165, 0)
        } else {
            Color32::from_rgb(255, 110, 0)
        };
        let hint_accent_soft = if visuals.dark_mode {
            Color32::from_rgb(255, 200, 130)
        } else {
            Color32::from_rgb(255, 190, 120)
        };

        let border_inactive = visuals.widgets.inactive.fg_stroke.color; // dark=(180, 180, 180) light=(60, 60, 60)
        let border_selected_cell = visuals.error_fg_color; // dark/light=(255, 0, 0)
        let border_selected_cell_peer = border_selected_cell;
        let border_selected_digit = visuals.selection.stroke.color; // dark=(192, 222, 255) light=(0, 83, 125)

        Self {
            cell_bg_default: visuals.text_edit_bg_color(), // dark=(10, 10, 10) light=(255, 255, 255)
            cell_bg_selected_digit,
            cell_bg_selected_digit_peer,

            note_bg_selected_digit: cell_bg_selected_digit,

            pill_hint: hint_accent,

            border_inactive,
            border_selected_cell,
            border_selected_cell_peer,
            border_selected_digit,
            border_hint_condition: hint_accent,

            underline_hint_condition: hint_accent_soft,
            underline_hint_application: hint_accent_soft,

            elimination_stroke: visuals.error_fg_color, // dark/light=(255, 0, 0)

            text_normal: visuals.text_color(), // dark=(140, 140, 140) light=(80, 80, 80)
            text_given: visuals.strong_text_color(), // dark=(255, 255, 255) light=(0, 0, 0)
            text_conflict: visuals.error_fg_color, // dark/light=(255, 0, 0)
        }
    }
}

/// Holds light/dark palettes and selects one based on current visuals.
#[derive(Debug, Clone)]
pub(crate) struct GridTheme {
    pub(crate) light: GridPalette,
    pub(crate) dark: GridPalette,
}

impl GridTheme {
    /// Create a theme using the current visuals for both palettes.
    ///
    /// This preserves existing colors today while allowing later divergence.
    #[must_use]
    pub(crate) fn from_visuals(visuals: &Visuals) -> Self {
        let palette = GridPalette::from_visuals(visuals);
        Self {
            light: palette.clone(),
            dark: palette,
        }
    }

    #[must_use]
    pub(crate) fn palette_for(&self, visuals: &Visuals) -> &GridPalette {
        if visuals.dark_mode {
            &self.dark
        } else {
            &self.light
        }
    }
}
