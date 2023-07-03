use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<Payload, Errors>
where
    Payload: Serialize,
    Errors: Serialize,
{
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Payload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<Errors>>,
}

impl<Payload, Errors> Response<Payload, Errors>
where
    Payload: Serialize,
    Errors: Serialize,
{
    pub fn success(payload: Payload) -> Self {
        Self {
            success: true,
            payload: Some(payload),
            errors: None,
        }
    }

    pub fn fail(errors: Errors) -> Self {
        let errors = Some(vec![errors]);

        Self {
            success: false,
            payload: None,
            errors,
        }
    }

    pub fn json(self) -> impl IntoResponse {
        Json(self)
    }
}

impl<Payload, Errors> Default for Response<Payload, Errors>
where
    Payload: Serialize,
    Errors: Serialize,
{
    fn default() -> Self {
        Self {
            success: true,
            payload: None,
            errors: None,
        }
    }
}
