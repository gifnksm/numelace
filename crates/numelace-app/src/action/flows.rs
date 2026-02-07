use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use portable_atomic::AtomicU64;

use crate::{
    action::{Action, SpinnerId, SpinnerKind},
    flow_executor::FlowHandle,
};

static NEXT_SPINNER_ID: AtomicU64 = AtomicU64::new(1);

#[must_use]
pub(crate) fn with_spinner<F>(
    handle: &FlowHandle,
    kind: SpinnerKind,
    future: F,
) -> WithSpinnerFuture<F>
where
    F: Future,
{
    WithSpinnerFuture::new(handle.clone(), kind, future)
}

/// Awaitable wrapper that toggles a flow spinner while the inner future runs.
pub(crate) struct WithSpinnerFuture<F>
where
    F: Future,
{
    handle: FlowHandle,
    id: SpinnerId,
    kind: SpinnerKind,
    started: bool,
    stopped: bool,
    inner: Pin<Box<F>>,
}

impl<F> WithSpinnerFuture<F>
where
    F: Future,
{
    #[must_use]
    pub(crate) fn new(handle: FlowHandle, kind: SpinnerKind, future: F) -> Self {
        let id = SpinnerId::new(NEXT_SPINNER_ID.fetch_add(1, portable_atomic::Ordering::Relaxed));
        Self {
            handle,
            id,
            kind,
            started: false,
            stopped: false,
            inner: Box::pin(future),
        }
    }
}

impl<F> Future for WithSpinnerFuture<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            self.started = true;
            self.handle.request_action(Action::StartSpinner {
                id: self.id,
                kind: self.kind,
            });
        }

        let result = self.inner.as_mut().poll(cx);

        if result.is_ready() {
            self.stopped = true;
            self.handle
                .request_action(Action::StopSpinner { id: self.id });
        }

        result
    }
}

impl<F> Drop for WithSpinnerFuture<F>
where
    F: Future,
{
    fn drop(&mut self) {
        if self.started && !self.stopped {
            self.handle
                .request_action(Action::StopSpinner { id: self.id });
        }
    }
}
