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
    pub(crate) same_digit: bool,
    pub(crate) house_selected: bool,
    pub(crate) house_same_digit: bool,
    pub(crate) conflict: bool,
}

impl Default for HighlightSettings {
    fn default() -> Self {
        Self {
            same_digit: true,
            house_selected: true,
            house_same_digit: true,
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
