/// Options that control digit input behavior.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct InputDigitOptions {
    pub(crate) rule_check_policy: RuleCheckPolicy,
    pub(crate) note_cleanup_policy: NoteCleanupPolicy,
}

impl InputDigitOptions {
    /// Creates options from explicit rule-check and note-cleanup policies.
    #[must_use]
    pub fn new(rule_check: RuleCheckPolicy, note_cleanup: NoteCleanupPolicy) -> Self {
        Self {
            rule_check_policy: rule_check,
            note_cleanup_policy: note_cleanup,
        }
    }

    /// Sets the rule check policy for this input.
    #[must_use]
    pub fn rule_check_policy(self, rule_check_policy: RuleCheckPolicy) -> Self {
        Self {
            rule_check_policy,
            ..self
        }
    }

    /// Sets the note cleanup policy for this input.
    #[must_use]
    pub fn note_cleanup_policy(self, note_cleanup_policy: NoteCleanupPolicy) -> Self {
        Self {
            note_cleanup_policy,
            ..self
        }
    }
}

/// Controls whether rule-violating inputs are permitted.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub enum RuleCheckPolicy {
    /// Allow inputs even if they conflict with existing digits.
    #[default]
    Permissive,
    /// Reject inputs that conflict with existing digits.
    Strict,
}

/// Controls how notes are cleaned up after digit entry.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub enum NoteCleanupPolicy {
    /// Do not modify notes in peer cells.
    #[default]
    None,
    /// Remove the placed digit from notes in peer cells.
    RemovePeers,
}

/// Indicates whether an input is currently allowed and, if not, why it is blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub enum InputCapability {
    /// The input is allowed.
    Allowed,
    /// The target cell is a given cell and cannot be modified.
    BlockedByGivenCell,
    /// The target cell is filled and cannot accept notes.
    BlockedByFilledCell,
    /// The input conflicts with an existing digit under strict rules.
    BlockedByConflict,
}
