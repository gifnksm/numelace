use std::collections::BTreeMap;

use numelace_solver::{TechniqueTier, technique};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum DifficultyPreset {
    #[default]
    Basic,
    Intermediate,
    UpperIntermediate,
    Advanced,
    Expert,
    Custom,
}

impl From<TechniqueTier> for DifficultyPreset {
    fn from(tier: TechniqueTier) -> Self {
        match tier {
            TechniqueTier::Fundamental | TechniqueTier::Basic => Self::Basic,
            TechniqueTier::Intermediate => Self::Intermediate,
            TechniqueTier::UpperIntermediate => Self::UpperIntermediate,
            TechniqueTier::Advanced => Self::Advanced,
            TechniqueTier::Expert => Self::Expert,
        }
    }
}

impl DifficultyPreset {
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Intermediate => "Intermediate",
            Self::UpperIntermediate => "Upper Intermediate",
            Self::Advanced => "Advanced",
            Self::Expert => "Expert",
            Self::Custom => "Custom",
        }
    }

    #[must_use]
    pub(crate) const fn all() -> [DifficultyPreset; 6] {
        [
            Self::Basic,
            Self::Intermediate,
            Self::UpperIntermediate,
            Self::Advanced,
            Self::Expert,
            Self::Custom,
        ]
    }

    pub(crate) fn parse(label: &str) -> Option<Self> {
        let normalized = label.trim();
        if normalized.eq_ignore_ascii_case("Basic") {
            Some(Self::Basic)
        } else if normalized.eq_ignore_ascii_case("Intermediate") {
            Some(Self::Intermediate)
        } else if normalized.eq_ignore_ascii_case("Upper Intermediate") {
            Some(Self::UpperIntermediate)
        } else if normalized.eq_ignore_ascii_case("Advanced") {
            Some(Self::Advanced)
        } else if normalized.eq_ignore_ascii_case("Expert") {
            Some(Self::Expert)
        } else if normalized.eq_ignore_ascii_case("Custom") {
            Some(Self::Custom)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewGameOptions {
    pub(crate) difficulty: DifficultyPreset,
    pub(crate) techniques: BTreeMap<&'static str, bool>,
    pub(crate) seed: String,
    pub(crate) max_attempts: usize,
}

impl Default for NewGameOptions {
    fn default() -> Self {
        let mut options = Self {
            difficulty: DifficultyPreset::Basic,
            techniques: BTreeMap::new(),
            seed: String::new(),
            max_attempts: Self::default_max_attempts(),
        };
        options.apply_preset(DifficultyPreset::Basic);
        options
    }
}

impl NewGameOptions {
    pub(crate) fn default_max_attempts() -> usize {
        100
    }

    pub(crate) fn apply_preset(&mut self, preset: DifficultyPreset) {
        self.difficulty = preset;
        if preset == DifficultyPreset::Custom {
            return;
        }
        let tier = preset.tier();
        self.set_techniques_by_tier(tier);
    }

    pub(crate) fn set_technique_enabled(&mut self, technique_id: &'static str, mut enabled: bool) {
        let Some(technique) = technique::find_technique_by_id(technique_id) else {
            return;
        };
        if technique.tier().is_fundamental() {
            enabled = true;
        }
        self.techniques.insert(technique_id, enabled);
        self.difficulty = DifficultyPreset::Custom;
    }

    #[must_use]
    pub(crate) fn is_technique_enabled(&self, technique_id: &'static str) -> bool {
        self.techniques.get(&technique_id).copied().unwrap_or(false)
    }

    pub(crate) fn set_techniques_by_tier(&mut self, tier: TechniqueTier) {
        self.techniques.clear();
        for technique in technique::all_techniques() {
            self.techniques
                .insert(technique.id(), technique.tier() <= tier);
        }
    }

    pub(crate) fn set_enabled_techniques(
        &mut self,
        enabled: impl IntoIterator<Item = &'static str>,
    ) {
        self.techniques.clear();
        for technique in technique::all_techniques() {
            let enabled = technique.tier().is_fundamental();
            self.techniques.insert(technique.id(), enabled);
        }
        for key in enabled {
            self.techniques.insert(key, true);
        }
        self.difficulty = DifficultyPreset::Custom;
    }
}

impl DifficultyPreset {
    #[must_use]
    pub(crate) const fn tier(self) -> TechniqueTier {
        match self {
            Self::Basic => TechniqueTier::Basic,
            Self::Intermediate => TechniqueTier::Intermediate,
            Self::UpperIntermediate => TechniqueTier::UpperIntermediate,
            Self::Advanced => TechniqueTier::Advanced,
            Self::Expert | Self::Custom => TechniqueTier::Expert,
        }
    }
}
