use std::{collections::BTreeMap, fmt::Write, num::NonZero};

use numelace_core::{Digit, DigitGrid, DigitGridParseError, Position, PositionNewError};
use numelace_game::{CellState, Game, GameError};
use numelace_solver::technique;
use serde::{Deserialize, Serialize};

use crate::state::{
    AppState, AssistSettings, DifficultyPreset, HighlightSettings, History, HistorySnapshot,
    InputMode, NewGameOptions, NotesSettings, Settings,
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
    selected_digit: Option<DigitDto>,
    #[serde(default)]
    input_mode: InputModeDto,
    #[serde(default)]
    new_game_options: NewGameOptionsDto,
    #[serde(default)]
    settings: SettingsDto,
    #[serde(default)]
    history: HistoryDto,
}

impl From<&AppState> for PersistedState {
    fn from(value: &AppState) -> Self {
        Self {
            game: GameDto::from(&value.game),
            selected_cell: value.selected_cell().map(PositionDto::from),
            selected_digit: value.selected_digit().map(DigitDto::from),
            input_mode: value.input_mode.into(),
            new_game_options: NewGameOptionsDto::from(&value.new_game_options),
            settings: SettingsDto::from(&value.settings),
            history: HistoryDto::from(value.history()),
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
    #[display("failed to parse selected digit: {_0}")]
    DigitParse(DigitParseError),
}

impl TryFrom<PersistedState> for AppState {
    type Error = AppStateConversionError;

    fn try_from(value: PersistedState) -> Result<Self, Self::Error> {
        Ok(AppState::from_parts(
            value.game.try_into()?,
            value.selected_cell.map(Position::try_from).transpose()?,
            value.selected_digit.map(Digit::try_from).transpose()?,
            value.input_mode.into(),
            value.new_game_options.into(),
            value.settings.into(),
            value.history.try_into()?,
        ))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct GameDto {
    problem: String,
    solution: String,
    filled: String,
    #[serde(default)]
    notes: [[u16; 9]; 9],
    #[serde(default)]
    initialized: bool,
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
            initialized: value.is_initialized(),
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
        if value.initialized {
            let problem: DigitGrid = value.problem.parse()?;
            let solution: DigitGrid = value.solution.parse()?;
            let filled: DigitGrid = value.filled.parse()?;
            Ok(Game::from_problem_filled_notes(
                &problem,
                &solution,
                &filled,
                &value.notes,
            )?)
        } else {
            // Uninitialized games are treated as empty, ignoring problem/solution/notes.
            Ok(Game::new_empty())
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct HistorySnapshotDto {
    filled: String,
    #[serde(default)]
    notes: [[u16; 9]; 9],
    #[serde(default)]
    selected_cell: Option<PositionDto>,
}

impl From<&HistorySnapshot> for HistorySnapshotDto {
    fn from(value: &HistorySnapshot) -> Self {
        Self {
            filled: value.filled.to_string(),
            notes: value.notes,
            selected_cell: value.selected_at_change.map(PositionDto::from),
        }
    }
}

impl TryFrom<HistorySnapshotDto> for HistorySnapshot {
    type Error = AppStateConversionError;

    fn try_from(value: HistorySnapshotDto) -> Result<Self, Self::Error> {
        let filled: DigitGrid = value.filled.parse()?;
        Ok(Self {
            filled,
            notes: value.notes,
            selected_at_change: value.selected_cell.map(Position::try_from).transpose()?,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct HistoryDto {
    #[serde(default = "History::default_capacity")]
    capacity: NonZero<usize>,
    entries: Vec<HistorySnapshotDto>,
    cursor: usize,
}

impl Default for HistoryDto {
    fn default() -> Self {
        Self::from(&History::default())
    }
}

impl From<&History> for HistoryDto {
    fn from(value: &History) -> Self {
        Self {
            capacity: value.capacity(),
            entries: value.entries().map(Into::into).collect(),
            cursor: value.cursor(),
        }
    }
}

impl TryFrom<HistoryDto> for History {
    type Error = AppStateConversionError;

    fn try_from(value: HistoryDto) -> Result<Self, Self::Error> {
        let entries = value
            .entries
            .into_iter()
            .map(HistorySnapshot::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::from_parts(value.capacity, entries, value.cursor))
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
        Position::try_from_xy(value.x, value.y)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct DigitDto(u8);

impl From<Digit> for DigitDto {
    fn from(value: Digit) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub(crate) enum DigitParseError {
    #[display("digit value {_0} is out of range (must be 1-9)")]
    InvalidDigit(#[error(not(source))] u8),
}

impl TryFrom<DigitDto> for Digit {
    type Error = DigitParseError;

    fn try_from(value: DigitDto) -> Result<Self, Self::Error> {
        if !(1..=9).contains(&value.0) {
            return Err(Self::Error::InvalidDigit(value.0));
        }
        Ok(Digit::from_value(value.0))
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
pub(crate) struct NewGameOptionsDto {
    difficulty: String,
    #[serde(default)]
    techniques: BTreeMap<String, bool>,
    #[serde(default)]
    seed: String,
    #[serde(default = "NewGameOptions::default_max_attempts")]
    max_attempts: usize,
}

impl Default for NewGameOptionsDto {
    fn default() -> Self {
        NewGameOptions::default().into()
    }
}

impl From<&NewGameOptions> for NewGameOptionsDto {
    fn from(value: &NewGameOptions) -> Self {
        let NewGameOptions {
            difficulty,
            techniques,
            seed,
            max_attempts,
        } = value;
        let techniques = techniques
            .iter()
            .map(|(id, enabled)| (id.to_string(), *enabled))
            .collect();
        Self {
            difficulty: difficulty.label().to_string(),
            techniques,
            seed: seed.clone(),
            max_attempts: *max_attempts,
        }
    }
}

impl From<NewGameOptions> for NewGameOptionsDto {
    fn from(value: NewGameOptions) -> Self {
        Self::from(&value)
    }
}

impl From<NewGameOptionsDto> for NewGameOptions {
    fn from(value: NewGameOptionsDto) -> Self {
        let difficulty = DifficultyPreset::parse(&value.difficulty).unwrap_or_default();
        let mut options = NewGameOptions {
            difficulty,
            seed: value.seed,
            techniques: BTreeMap::new(),
            max_attempts: value.max_attempts,
        };
        let enabled = value
            .techniques
            .iter()
            .filter_map(|(id, enabled)| {
                if *enabled {
                    technique::find_technique_by_id(id).map(|technique| technique.id())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        options.set_enabled_techniques(enabled);
        options.apply_preset(difficulty);
        options
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
    pub(crate) selected_digit: bool,
    pub(crate) house_selected_cell: bool,
    pub(crate) house_selected_digit: bool,
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
            selected_digit: value.selected_digit,
            house_selected_cell: value.house_selected_cell,
            house_selected_digit: value.house_selected_digit,
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
            selected_digit: value.selected_digit,
            house_selected_cell: value.house_selected_cell,
            house_selected_digit: value.house_selected_digit,
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
