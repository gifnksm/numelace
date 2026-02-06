#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native::{WorkHandle, enqueue, warm_up};
#[cfg(target_arch = "wasm32")]
pub(crate) use wasm::{WorkHandle, enqueue, warm_up};
