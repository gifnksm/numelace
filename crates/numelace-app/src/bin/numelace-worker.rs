//! Numelace web worker entry point for background puzzle generation.
//!
//! This binary is built only for WASM targets and handles offloaded work requests.
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

#[cfg(target_arch = "wasm32")]
mod wasm32 {
    use numelace_app::worker_api::{WorkRequest, WorkResponse};
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

    /// Initializes the worker event loop for background requests.
    #[wasm_bindgen(start)]
    pub(crate) fn start() {
        let global = js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>();
        let global_for_callback = global.clone();

        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            let response = match serde_wasm_bindgen::from_value::<WorkRequest>(event.data()) {
                Ok(req) => req.handle(),
                Err(_) => WorkResponse::deserialization_error(),
            };
            let _ =
                global_for_callback.post_message(&serde_wasm_bindgen::to_value(&response).unwrap());
        }) as Box<dyn FnMut(MessageEvent)>);

        global.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
    }
}

fn main() {}
