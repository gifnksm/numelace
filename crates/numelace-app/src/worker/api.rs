use serde::{Deserialize, Serialize};

use super::{WorkError, WorkRequest as InnerWorkRequest, WorkResponse as InnerWorkResponse};

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct WorkRequest(InnerWorkRequest);

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct WorkResponse(InnerWorkResponse);

impl WorkRequest {
    #[must_use]
    pub fn handle(self) -> WorkResponse {
        WorkResponse(self.0.handle())
    }
}

impl WorkResponse {
    #[must_use]
    pub fn deserialization_error() -> Self {
        Self(InnerWorkResponse::Error(WorkError::DeserializationFailed))
    }
}
