use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_channel::oneshot;
use numelace_game::Game;
use portable_atomic::AtomicU64;

use crate::{
    action::{
        AlertKind, AlertResult, ConfirmKind, ConfirmResult, ModalRequest, SpinnerId, SpinnerKind,
        StateQueryAction, UiAction,
    },
    flow::FlowHandle,
};

pub(super) async fn show_confirm_dialog(handle: &FlowHandle, kind: ConfirmKind) -> ConfirmResult {
    let (responder, receiver) = oneshot::channel();
    handle.request_action(
        UiAction::OpenModal(ModalRequest::Confirm {
            kind,
            responder: Some(responder),
        })
        .into(),
    );
    match receiver.await {
        Ok(result) => result,
        Err(_) => ConfirmResult::Cancelled,
    }
}

pub(super) async fn show_alert_dialog(handle: &FlowHandle, kind: AlertKind) -> AlertResult {
    let (responder, receiver) = oneshot::channel();
    handle.request_action(
        UiAction::OpenModal(ModalRequest::Alert {
            kind,
            responder: Some(responder),
        })
        .into(),
    );
    match receiver.await {
        Ok(result) => result,
        Err(_) => AlertResult::Ok,
    }
}

pub(super) async fn request_undo_games(handle: &FlowHandle) -> Option<Vec<Game>> {
    let (responder, receiver) = oneshot::channel();
    handle.request_action(StateQueryAction::BuildUndoGames { responder }.into());
    receiver.await.ok()
}

#[must_use]
pub(super) fn with_spinner<F>(
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
pub(super) struct WithSpinnerFuture<F>
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
    fn new(handle: FlowHandle, kind: SpinnerKind, future: F) -> Self {
        static NEXT_SPINNER_ID: AtomicU64 = AtomicU64::new(1);

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
            self.handle.request_action(
                UiAction::StartSpinner {
                    id: self.id,
                    kind: self.kind,
                }
                .into(),
            );
        }

        let result = self.inner.as_mut().poll(cx);

        if result.is_ready() {
            self.stopped = true;
            self.handle
                .request_action(UiAction::StopSpinner { id: self.id }.into());
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
                .request_action(UiAction::StopSpinner { id: self.id }.into());
        }
    }
}
