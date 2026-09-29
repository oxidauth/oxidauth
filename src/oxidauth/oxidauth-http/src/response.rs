use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

    pub fn fail() -> Self {
        Self {
            success: false,
            payload: None,
            errors: None,
            warnings: None,
            notices: None,
            status_code: StatusCode::BAD_REQUEST,
        }
    }

    pub fn unauthorized() -> Self {
        Self::fail().status(StatusCode::UNAUTHORIZED)
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
            return Response::fail()
                .error("unable to serialize error to value");
        };

        if self.errors.is_none() {
            self.errors = Some(vec![err]);
        } else {
            self.errors
                .as_mut()
                .unwrap()
                .push(err);
        }

        self
    }

    pub fn warning(mut self, warning: impl Serialize) -> Self {
        let Ok(warning) = serde_json::to_value(warning) else {
            return Response::fail()
                .error("unable to serialize warning to value");
        };

        if self.warnings.is_none() {
            self.warnings = Some(vec![warning]);
        } else {
            self.warnings
                .as_mut()
                .unwrap()
                .push(warning);
        }

        self
    }

    pub fn notice(mut self, notice: impl Serialize) -> Self {
        let Ok(notice) = serde_json::to_value(notice) else {
            return Response::fail()
                .error("unable to serialize notice to value");
        };

        if self.notices.is_none() {
            self.notices = Some(vec![notice]);
        } else {
            self.notices
                .as_mut()
                .unwrap()
                .push(notice);
        }

        self
    }
}

impl<P> IntoResponse for Response<P>
where
    P: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        if self.errors.is_some() {
            return (
                StatusCode::BAD_REQUEST,
                Json(self),
            )
                .into_response();
        }

        (StatusCode::OK, Json(self)).into_response()
    }
}
