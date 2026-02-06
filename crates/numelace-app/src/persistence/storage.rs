use eframe::Storage;

use crate::{persistence::dto::PersistedState, state::AppState};

const STATE_KEY: &str = "numelace.state";

pub(crate) fn load_state(storage: &dyn Storage) -> Option<AppState> {
    let state = eframe::get_value::<PersistedState>(storage, STATE_KEY)?;
    state.try_into().ok()
}

pub(crate) fn save_state(storage: &mut dyn Storage, state: &AppState) {
    let state = PersistedState::from(state);
    eframe::set_value(storage, STATE_KEY, &state);
}
