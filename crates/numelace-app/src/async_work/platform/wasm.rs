//! WASM async work backend.
//!
//! This module owns the web worker integration and keeps the main thread responsive
//! during background puzzle generation. Failures are treated as internal errors and
//! trigger a panic via the worker error handler.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{Event, MessageEvent, Url, Worker};

use super::super::{WorkError, WorkRequest, WorkResponse};

/// A handle for polling background work completion.
///
/// Note: the WASM backend assumes a single in-flight request at a time.
pub(crate) struct WorkHandle {
    response: Rc<RefCell<Option<WorkResponse>>>,
    error: Rc<RefCell<Option<WorkError>>>,
}

impl std::fmt::Debug for WorkHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkHandle").finish()
    }
}

impl WorkHandle {
    /// Attempts to poll for a completed response.
    pub(crate) fn poll(&mut self) -> Result<Option<WorkResponse>, WorkError> {
        if let Some(err) = self.error.borrow_mut().take() {
            return Err(err);
        }

        Ok(self.response.borrow_mut().take())
    }
}

struct SharedWorker {
    worker: Worker,
    response: Rc<RefCell<Option<WorkResponse>>>,
    error: Rc<RefCell<Option<WorkError>>>,
    _onmessage: Closure<dyn FnMut(MessageEvent)>,
    _onerror: Closure<dyn FnMut(Event)>,
}

impl SharedWorker {
    fn new() -> Result<Self, WorkError> {
        let worker_url = read_worker_url()?;
        let worker = Worker::new(&worker_url).map_err(|_| WorkError::WorkerInitFailed)?;

        let response = Rc::new(RefCell::new(None));
        let error = Rc::new(RefCell::new(None));

        let response_cell = Rc::clone(&response);
        let error_cell_for_message = Rc::clone(&error);
        let error_cell_for_error = Rc::clone(&error);

        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            let value = event.data();
            match serde_wasm_bindgen::from_value::<WorkResponse>(value) {
                Ok(resp) => {
                    *response_cell.borrow_mut() = Some(resp);
                }
                Err(_) => {
                    *error_cell_for_message.borrow_mut() = Some(WorkError::DeserializationFailed);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        let onerror = Closure::wrap(Box::new(move |_event: Event| -> () {
            *error_cell_for_error.borrow_mut() = Some(WorkError::WorkerDisconnected);
        }) as Box<dyn FnMut(Event)>);

        worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        worker.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        Ok(Self {
            worker,
            response,
            error,
            _onmessage: onmessage,
            _onerror: onerror,
        })
    }

    fn reset(&mut self) {
        *self.response.borrow_mut() = None;
        *self.error.borrow_mut() = None;
    }

    fn send(&mut self, request: &WorkRequest) -> Result<(), WorkError> {
        let payload =
            serde_wasm_bindgen::to_value(&request).map_err(|_| WorkError::SerializationFailed)?;
        self.worker
            .post_message(&payload)
            .map_err(|_| WorkError::SerializationFailed)?;
        Ok(())
    }
}

thread_local! {
    static SHARED_WORKER: RefCell<Option<SharedWorker>> = const { RefCell::new(None) };
}

fn with_worker<F, R>(f: F) -> Result<R, WorkError>
where
    F: FnOnce(&mut SharedWorker) -> Result<R, WorkError>,
{
    SHARED_WORKER.with(|cell| {
        let mut guard = cell.borrow_mut();
        if guard.is_none() {
            *guard = Some(SharedWorker::new()?);
        }
        let worker = guard.as_mut().expect("worker should be initialized");
        f(worker)
    })
}

/// Starts the shared worker without sending a request.
pub(crate) fn warm_up() -> Result<(), WorkError> {
    with_worker(|worker| {
        worker.reset();
        Ok(())
    })
}

/// Enqueues a background task and returns a handle for polling completion.
#[expect(clippy::needless_pass_by_value)]
pub(crate) fn enqueue(request: WorkRequest) -> Result<WorkHandle, WorkError> {
    with_worker(|worker| {
        worker.reset();
        worker.send(&request)?;

        Ok(WorkHandle {
            response: Rc::clone(&worker.response),
            error: Rc::clone(&worker.error),
        })
    })
}

fn read_worker_url() -> Result<String, WorkError> {
    let document = web_sys::window()
        .ok_or(WorkError::WorkerUrlMissing)?
        .document()
        .ok_or(WorkError::WorkerUrlMissing)?;
    let base_uri = document
        .base_uri()
        .map_err(|_| WorkError::WorkerUrlMissing)?
        .ok_or(WorkError::WorkerUrlMissing)?;
    let url = Url::new_with_base("numelace-worker-bootstrap.js", &base_uri)
        .map_err(|_| WorkError::WorkerUrlMissing)?;
    Ok(url.href())
}
