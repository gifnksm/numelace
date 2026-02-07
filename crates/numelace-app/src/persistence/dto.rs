use std::fmt::Write;

use numelace_core::{DigitGrid, DigitGridParseError, Position, PositionNewError};
use numelace_game::{CellState, Game, GameError};
use serde::{Deserialize, Serialize};

use crate::state::{
    AppState, AssistSettings, HighlightSettings, InputMode, NotesSettings, Settings,
};

// DTO defaulting guidance:
// - When a DTO has a sensible default, use container-level #[serde(default)].
// - Implement Default by delegating to the corresponding state Default,
//   so missing fields preserve non-false defaults on deserialization.

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PersistedState {
    game: GameDto,
    #[serde(default)]
    selected_cell: Option<PositionDto>,
    #[serde(default)]
    input_mode: InputModeDto,
    #[serde(default)]
    settings: SettingsDto,
}

impl From<&AppState> for PersistedState {
    fn from(value: &AppState) -> Self {
        Self {
            game: GameDto::from(&value.game),
            selected_cell: value.selected_cell.map(PositionDto::from),
            input_mode: value.input_mode.into(),
            settings: SettingsDto::from(&value.settings),
        }
    }
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub(crate) enum AppStateConversionError {
    #[display("failed to parse game data: {_0}")]
    GameParse(DigitGridParseError),
    #[display("failed to apply saved game data: {_0}")]
    GameRestore(GameError),
    #[display("failed to construct selected position: {_0}")]
    PositionNew(PositionNewError),
}

impl TryFrom<PersistedState> for AppState {
    type Error = AppStateConversionError;

    fn try_from(value: PersistedState) -> Result<Self, Self::Error> {
        Ok(Self {
            game: value.game.try_into()?,
            selected_cell: value.selected_cell.map(Position::try_from).transpose()?,
            input_mode: value.input_mode.into(),
            settings: value.settings.into(),
            dirty: false,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct GameDto {
    problem: String,
    solution: String,
    filled: String,
    #[serde(default)]
    notes: [[u16; 9]; 9],
}

impl From<&Game> for GameDto {
    fn from(value: &Game) -> Self {
        let mut problem = String::with_capacity(81);
        let solution = value.solution().to_string();
        let mut filled = String::with_capacity(81);
        let mut notes = [[0; 9]; 9];

        for pos in Position::ALL {
            match value.cell(pos) {
                CellState::Given(digit) => {
                    let _ = write!(problem, "{digit}");
                    filled.push('.');
                }
                CellState::Filled(digit) => {
                    problem.push('.');
                    let _ = write!(filled, "{digit}");
                }
                CellState::Notes(digits) => {
                    notes[usize::from(pos.y())][usize::from(pos.x())] = digits.bits();
                    problem.push('.');
                    filled.push('.');
                }
                CellState::Empty => {
                    problem.push('.');
                    filled.push('.');
                }
            }
        }

        Self {
            problem,
            solution,
            filled,
            notes,
        }
    }
}

impl From<Game> for GameDto {
    fn from(value: Game) -> Self {
        GameDto::from(&value)
    }
}

impl TryFrom<GameDto> for Game {
    type Error = AppStateConversionError;

    fn try_from(value: GameDto) -> Result<Self, Self::Error> {
        let problem: DigitGrid = value.problem.parse()?;
        let solution: DigitGrid = value.solution.parse()?;
        let filled: DigitGrid = value.filled.parse()?;
        Ok(Game::from_problem_filled_notes(
            &problem,
            &solution,
            &filled,
            &value.notes,
        )?)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PositionDto {
    x: u8,
    y: u8,
}

impl From<Position> for PositionDto {
    fn from(value: Position) -> Self {
        Self {
            x: value.x(),
            y: value.y(),
        }
    }
}

impl TryFrom<PositionDto> for Position {
    type Error = PositionNewError;

    fn try_from(value: PositionDto) -> Result<Self, Self::Error> {
        Position::try_new(value.x, value.y)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) enum InputModeDto {
    #[default]
    Fill,
    Notes,
}

impl From<InputMode> for InputModeDto {
    fn from(value: InputMode) -> Self {
        match value {
            InputMode::Fill => Self::Fill,
            InputMode::Notes => Self::Notes,
        }
    }
}

impl From<InputModeDto> for InputMode {
    fn from(value: InputModeDto) -> Self {
        match value {
            InputModeDto::Fill => Self::Fill,
            InputModeDto::Notes => Self::Notes,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct SettingsDto {
    assist: AssistSettingsDto,
}

impl Default for SettingsDto {
    fn default() -> Self {
        Settings::default().into()
    }
}

impl From<&Settings> for SettingsDto {
    fn from(value: &Settings) -> Self {
        Self {
            assist: AssistSettingsDto::from(&value.assist),
        }
    }
}

impl From<Settings> for SettingsDto {
    fn from(value: Settings) -> Self {
        Self::from(&value)
    }
}

impl From<SettingsDto> for Settings {
    fn from(value: SettingsDto) -> Self {
        Self {
            assist: value.assist.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct AssistSettingsDto {
    pub(crate) block_rule_violations: bool,
    pub(crate) highlight: HighlightSettingsDto,
    pub(crate) notes: NotesSettingsDto,
}

impl Default for AssistSettingsDto {
    fn default() -> Self {
        AssistSettings::default().into()
    }
}

impl From<&AssistSettings> for AssistSettingsDto {
    fn from(value: &AssistSettings) -> Self {
        Self {
            block_rule_violations: value.block_rule_violations,
            highlight: HighlightSettingsDto::from(&value.highlight),
            notes: NotesSettingsDto::from(&value.notes),
        }
    }
}

impl From<AssistSettings> for AssistSettingsDto {
    fn from(value: AssistSettings) -> Self {
        Self::from(&value)
    }
}

impl From<AssistSettingsDto> for AssistSettings {
    fn from(value: AssistSettingsDto) -> Self {
        Self {
            block_rule_violations: value.block_rule_violations,
            highlight: value.highlight.into(),
            notes: value.notes.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[expect(clippy::struct_excessive_bools)]
pub(crate) struct HighlightSettingsDto {
    pub(crate) same_digit: bool,
    pub(crate) house_selected: bool,
    pub(crate) house_same_digit: bool,
    pub(crate) conflict: bool,
}

impl Default for HighlightSettingsDto {
    fn default() -> Self {
        HighlightSettings::default().into()
    }
}

impl From<&HighlightSettings> for HighlightSettingsDto {
    fn from(value: &HighlightSettings) -> Self {
        Self {
            same_digit: value.same_digit,
            house_selected: value.house_selected,
            house_same_digit: value.house_same_digit,
            conflict: value.conflict,
        }
    }
}

impl From<HighlightSettings> for HighlightSettingsDto {
    fn from(value: HighlightSettings) -> Self {
        Self::from(&value)
    }
}

impl From<HighlightSettingsDto> for HighlightSettings {
    fn from(value: HighlightSettingsDto) -> Self {
        Self {
            same_digit: value.same_digit,
            house_selected: value.house_selected,
            house_same_digit: value.house_same_digit,
            conflict: value.conflict,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct NotesSettingsDto {
    pub(crate) auto_remove_peer_notes_on_fill: bool,
    pub(crate) auto_fill_notes_on_new_or_reset: bool,
}

impl Default for NotesSettingsDto {
    fn default() -> Self {
        AssistSettings::default().notes.into()
    }
}

impl From<&NotesSettings> for NotesSettingsDto {
    fn from(value: &NotesSettings) -> Self {
        Self {
            auto_remove_peer_notes_on_fill: value.auto_remove_peer_notes_on_fill,
            auto_fill_notes_on_new_or_reset: value.auto_fill_notes_on_new_or_reset,
        }
    }
}

impl From<NotesSettingsDto> for NotesSettings {
    fn from(value: NotesSettingsDto) -> Self {
        Self {
            auto_remove_peer_notes_on_fill: value.auto_remove_peer_notes_on_fill,
            auto_fill_notes_on_new_or_reset: value.auto_fill_notes_on_new_or_reset,
        }
    }
}

impl From<NotesSettings> for NotesSettingsDto {
    fn from(value: NotesSettings) -> Self {
        Self::from(&value)
    }
}
