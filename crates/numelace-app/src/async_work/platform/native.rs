//! Native async work backend using a background thread and channel.
use std::sync::{OnceLock, mpsc};

use super::super::{WorkError, WorkRequest, WorkResponse};

struct WorkRequestEnvelope {
    request: WorkRequest,
    response_tx: mpsc::Sender<WorkResponse>,
}

// Shared worker thread sender reused across requests.
static WORKER_SENDER: OnceLock<mpsc::Sender<WorkRequestEnvelope>> = OnceLock::new();

/// A handle for polling background work completion.
pub(crate) struct WorkHandle {
    receiver: mpsc::Receiver<WorkResponse>,
}

impl std::fmt::Debug for WorkHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkHandle").finish()
    }
}

impl WorkHandle {
    /// Attempts to poll for a completed response.
    pub(crate) fn poll(&mut self) -> Result<Option<WorkResponse>, WorkError> {
        use mpsc::TryRecvError;

        match self.receiver.try_recv() {
            Ok(response) => Ok(Some(response)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(WorkError::WorkerDisconnected),
        }
    }
}

/// Starts the shared worker thread without sending a request.
#[expect(clippy::unnecessary_wraps)]
pub(crate) fn warm_up() -> Result<(), WorkError> {
    let _ = WORKER_SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<WorkRequestEnvelope>();
        std::thread::spawn(move || {
            while let Ok(envelope) = rx.recv() {
                let response = envelope.request.handle();
                let _ = envelope.response_tx.send(response);
            }
        });
        tx
    });
    Ok(())
}

/// Enqueues a background task on the shared worker thread and returns a handle for polling completion.
pub(crate) fn enqueue(request: WorkRequest) -> Result<WorkHandle, WorkError> {
    let worker_tx = WORKER_SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<WorkRequestEnvelope>();
        std::thread::spawn(move || {
            while let Ok(envelope) = rx.recv() {
                let response = envelope.request.handle();
                let _ = envelope.response_tx.send(response);
            }
        });
        tx
    });

    let (response_tx, response_rx) = mpsc::channel();
    worker_tx
        .send(WorkRequestEnvelope {
            request,
            response_tx,
        })
        .map_err(|_| WorkError::WorkerDisconnected)?;

    Ok(WorkHandle {
        receiver: response_rx,
    })
}
