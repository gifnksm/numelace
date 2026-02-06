use serde::{Deserialize, Serialize};

use crate::async_work;

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct WorkRequest(async_work::WorkRequest);
#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct WorkResponse(async_work::WorkResponse);

impl WorkRequest {
    #[must_use]
    pub fn handle(&self) -> WorkResponse {
        WorkResponse(self.0.handle())
    }
}

impl WorkResponse {
    #[must_use]
    pub fn deserialization_error() -> Self {
        Self(async_work::WorkResponse::Error(
            async_work::WorkError::DeserializationFailed,
        ))
    }
}
