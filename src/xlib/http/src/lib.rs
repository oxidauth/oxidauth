//! Vendored from freshbrewlabs project-template @ c38ec0a (2026-09); re-diff when templates change.
pub use axum::response::Response as AxumResponse;
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use data::{CreateState, DeleteState, LoadingState, SaveState};

pub mod data;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<P>
where
    P: Serialize,
{
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<P>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notices: Option<Vec<Value>>,

    #[serde(skip)]
    status_code: StatusCode,
}

impl<P> Response<P>
where
    P: Serialize,
{
    pub fn success() -> Self {
        Self {
            success: true,
            payload: None,
            errors: None,
            warnings: None,
            notices: None,
            status_code: StatusCode::OK,
        }
    }

    pub fn fail(status_code: StatusCode) -> Self {
        Self {
            success: false,
            payload: None,
            errors: None,
            warnings: None,
            notices: None,
            status_code,
        }
    }

    pub fn unauthorized() -> Self {
        Self::fail(StatusCode::UNAUTHORIZED)
    }

    pub fn internal_error() -> Self {
        Self::fail(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn bad_request() -> Self {
        Self::fail(StatusCode::BAD_REQUEST)
    }

    pub fn status(mut self, status_code: StatusCode) -> Self {
        self.status_code = status_code;
        self
    }

    pub fn payload(mut self, payload: P) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn error(mut self, err: impl Serialize) -> Self {
        let Ok(err) = serde_json::to_value(err) else {
            return Response::fail(StatusCode::INTERNAL_SERVER_ERROR).error("unable to serialize error to value");
        };

        match self.errors.is_none() {
            true => {
                self.errors = Some(vec![err]);
            }
            false => {
                self.errors.as_mut().unwrap().push(err);
            }
        }
        self
    }

    pub fn warning(mut self, warning: impl Serialize) -> Self {
        let Ok(warning) = serde_json::to_value(warning) else {
            return Response::fail(StatusCode::INTERNAL_SERVER_ERROR).error("unable to serialize warning to value");
        };

        match self.warnings.is_none() {
            true => {
                self.warnings = Some(vec![warning]);
            }
            false => {
                self.warnings.as_mut().unwrap().push(warning);
            }
        }
        self
    }

    pub fn notice(mut self, notice: impl Serialize) -> Self {
        let Ok(notice) = serde_json::to_value(notice) else {
            return Response::fail(StatusCode::INTERNAL_SERVER_ERROR).error("unable to serialize notice to value");
        };

        match self.notices.is_none() {
            true => {
                self.notices = Some(vec![notice]);
            }
            false => {
                self.notices.as_mut().unwrap().push(notice);
            }
        }
        self
    }

    pub fn get_status(&self) -> StatusCode {
        self.status_code
    }
}

impl<P> IntoResponse for Response<P>
where
    P: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code, Json(self)).into_response()
    }
}