use numelace_core::{Digit, Position};
use numelace_solver::technique::BoxedTechniqueStep;

use crate::{
    action::{ModalRequest, SpinnerId, SpinnerKind},
    flow::FlowExecutor,
};

// UiState holds ephemeral UI-only state (modals, spinners, ghosts). It is not persisted.
#[derive(Debug)]
pub(crate) struct UiState {
    pub(crate) active_modal: Option<ModalRequest>,
    pub(crate) conflict_ghost: Option<(Position, GhostType)>,
    pub(crate) hint_state: Option<HintState>,
    pub(crate) executor: FlowExecutor,
    pub(crate) spinner_state: SpinnerState,
}

impl UiState {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            active_modal: None,
            conflict_ghost: None,
            hint_state: None,
            executor: FlowExecutor::new(),
            spinner_state: SpinnerState::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HintStage {
    Stage1,
    Stage2,
    #[expect(dead_code)]
    Stage3,
}

#[derive(Debug, Clone)]
pub(crate) struct HintState {
    pub(crate) stage: HintStage,
    pub(crate) step: BoxedTechniqueStep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GhostType {
    Digit(Digit),
    Note(Digit),
}

#[derive(Debug, Default)]
pub(crate) struct SpinnerState {
    active: Vec<SpinnerEntry>,
}

impl SpinnerState {
    pub(crate) fn start(&mut self, id: SpinnerId, kind: SpinnerKind) {
        self.active.push(SpinnerEntry { id, kind });
    }

    pub(crate) fn stop(&mut self, id: SpinnerId) {
        if let Some(index) = self.active.iter().position(|entry| entry.id == id) {
            self.active.remove(index);
        }
    }

    #[must_use]
    pub(crate) fn is_active(&self) -> bool {
        !self.active.is_empty()
    }

    #[must_use]
    pub(crate) fn active_kind(&self) -> Option<SpinnerKind> {
        self.active.first().map(|entry| entry.kind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpinnerEntry {
    pub(crate) id: SpinnerId,
    pub(crate) kind: SpinnerKind,
}
