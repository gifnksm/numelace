use numelace_core::Position;
use numelace_game::Game;

#[derive(Debug)]
pub struct AppState {
    pub game: Game,
    pub selected_cell: Option<Position>,
    pub input_mode: InputMode,
    pub settings: Settings,
}

impl AppState {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            selected_cell: None,
            input_mode: InputMode::Fill,
            settings: Settings::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub enum InputMode {
    Fill,
    Notes,
}

#[derive(Debug, Default, Clone)]
pub struct Settings {
    pub highlight: HighlightSettings,
    pub appearance: AppearanceSettings,
}

#[derive(Debug, Clone)]
#[expect(clippy::struct_excessive_bools)]
pub struct HighlightSettings {
    pub same_digit: bool,
    pub house_selected: bool,
    pub house_same_digit: bool,
    pub conflict: bool,
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

#[derive(Debug, Default, Clone)]
pub struct AppearanceSettings {
    pub theme: Theme,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

#[derive(Debug, Default)]
pub struct UiState {
    pub show_new_game_confirm_dialogue: bool,
}
