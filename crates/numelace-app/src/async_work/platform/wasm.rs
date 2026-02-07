//! WASM async work backend.
//!
//! This module owns the web worker integration and keeps the main thread responsive
//! during background puzzle generation. Failures are treated as internal errors and
//! trigger a panic via the worker error handler.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{Event, MessageEvent, Url, Worker};

use super::super::{WorkError, WorkRequest, WorkResponse};

/// A handle for polling background work completion.
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

struct PendingSlot {
    response: Rc<RefCell<Option<WorkResponse>>>,
    error: Rc<RefCell<Option<WorkError>>>,
}

struct SharedWorker {
    worker: Worker,
    pending: Rc<RefCell<VecDeque<PendingSlot>>>,
    _onmessage: Closure<dyn FnMut(MessageEvent)>,
    _onerror: Closure<dyn FnMut(Event)>,
}

impl SharedWorker {
    fn new() -> Result<Self, WorkError> {
        let worker_url = read_worker_url()?;
        let worker = Worker::new(&worker_url).map_err(|_| WorkError::WorkerInitFailed)?;

        let pending = Rc::new(RefCell::new(VecDeque::<PendingSlot>::new()));
        let pending_for_message = Rc::clone(&pending);
        let pending_for_error = Rc::clone(&pending);

        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            let Some(slot) = pending_for_message.borrow_mut().pop_front() else {
                return;
            };

            let value = event.data();
            match serde_wasm_bindgen::from_value::<WorkResponse>(value) {
                Ok(resp) => {
                    *slot.response.borrow_mut() = Some(resp);
                }
                Err(_) => {
                    *slot.error.borrow_mut() = Some(WorkError::DeserializationFailed);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        let onerror = Closure::wrap(Box::new(move |_event: Event| -> () {
            for slot in pending_for_error.borrow().iter() {
                *slot.error.borrow_mut() = Some(WorkError::WorkerDisconnected);
            }
        }) as Box<dyn FnMut(Event)>);

        worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        worker.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        Ok(Self {
            worker,
            pending,
            _onmessage: onmessage,
            _onerror: onerror,
        })
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
    with_worker(|_worker| Ok(()))
}

/// Enqueues a background task and returns a handle for polling completion.
#[expect(clippy::needless_pass_by_value)]
pub(crate) fn enqueue(request: WorkRequest) -> Result<WorkHandle, WorkError> {
    let response = Rc::new(RefCell::new(None));
    let error = Rc::new(RefCell::new(None));
    let slot = PendingSlot {
        response: Rc::clone(&response),
        error: Rc::clone(&error),
    };

    with_worker(|worker| {
        worker.send(&request)?;
        worker.pending.borrow_mut().push_back(slot);

        Ok(WorkHandle { response, error })
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
