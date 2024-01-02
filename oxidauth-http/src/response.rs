use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
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
}

impl<P> Response<P>
where
    P: Serialize + DeserializeOwned,
{
    // pub fn new(success: bool) -> Self {
    //     Self {
    //         success,
    //         payload: None,
    //         errors: None,
    //         warnings: None,
    //         notices: None,
    //     }
    // }

    pub fn success() -> Self {
        Self {
            success: true,
            payload: None,
            errors: None,
            warnings: None,
            notices: None,
        }
    }

    pub fn fail() -> Self {
        Self {
            success: false,
            payload: None,
            errors: None,
            warnings: None,
            notices: None,
        }
    }

    pub fn payload(mut self, payload: P) -> Self {
        self.payload = Some(payload);

        self
    }

    pub fn error(mut self, err: impl TryInto<Value>) -> Self {
        let Ok(err) = err.try_into() else {
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

    // pub fn warning(mut self, err: impl TryInto<Value>) -> Self {
    //     let Ok(err) = err.try_into() else {
    //         return Response::fail().error("unable to serialize to value");
    //     };
    //
    //     if self.warnings.is_none() {
    //         self.warnings = Some(vec![err]);
    //     } else {
    //         self.warnings.as_mut().unwrap().push(err);
    //     }
    //
    //     self
    // }

    pub fn notice(mut self, err: impl TryInto<Value>) -> Self {
        let Ok(err) = err.try_into() else {
            return Response::fail().error("unable to serialize to value");
        };

        if self.notices.is_none() {
            self.notices = Some(vec![err]);
        } else {
            self.notices
                .as_mut()
                .unwrap()
                .push(err);
        }

        self
    }

    // pub fn to_json(self) -> impl IntoResponse {
    //     Json(self)
    // }
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
