#[derive(Debug, Default, Clone)]
pub(crate) struct Settings {
    pub(crate) assist: AssistSettings,
}

#[derive(Debug, Clone)]
pub(crate) struct AssistSettings {
    pub(crate) block_rule_violations: bool,
    pub(crate) highlight: HighlightSettings,
    pub(crate) notes: NotesSettings,
}

impl Default for AssistSettings {
    fn default() -> Self {
        Self {
            block_rule_violations: true,
            highlight: HighlightSettings::default(),
            notes: NotesSettings::default(),
        }
    }
}

#[derive(Debug, Clone)]
#[expect(clippy::struct_excessive_bools)]
pub(crate) struct HighlightSettings {
    pub(crate) selected_digit: bool,
    pub(crate) selected_cell_peer: bool,
    pub(crate) selected_digit_peer: bool,
    pub(crate) conflict: bool,
}

impl Default for HighlightSettings {
    fn default() -> Self {
        Self {
            selected_digit: true,
            selected_cell_peer: false,
            selected_digit_peer: true,
            conflict: true,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NotesSettings {
    pub(crate) auto_remove_peer_notes_on_fill: bool,
    pub(crate) auto_fill_notes_on_new_or_reset: bool,
}

impl Default for NotesSettings {
    fn default() -> Self {
        Self {
            auto_remove_peer_notes_on_fill: true,
            auto_fill_notes_on_new_or_reset: true,
        }
    }
}
