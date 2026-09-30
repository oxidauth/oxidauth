//! Vendored from freshbrewlabs project-template @ c38ec0a (2026-09); re-diff when templates change.
pub use axum::response::Response as AxumResponse;
use axum::{Json, http::StatusCode, response::IntoResponse};
pub use data::{CreateState, DeleteState, LoadingState, SaveState};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
            return Response::fail(StatusCode::INTERNAL_SERVER_ERROR)
                .error("unable to serialize error to value");
        };

        match self.errors.is_none() {
            true => {
                self.errors = Some(vec![err]);
            },
            false => {
                self.errors
                    .as_mut()
                    .unwrap()
                    .push(err);
            },
        }
        self
    }

    pub fn warning(mut self, warning: impl Serialize) -> Self {
        let Ok(warning) = serde_json::to_value(warning) else {
            return Response::fail(StatusCode::INTERNAL_SERVER_ERROR)
                .error("unable to serialize warning to value");
        };

        match self.warnings.is_none() {
            true => {
                self.warnings = Some(vec![warning]);
            },
            false => {
                self.warnings
                    .as_mut()
                    .unwrap()
                    .push(warning);
            },
        }
        self
    }

    pub fn notice(mut self, notice: impl Serialize) -> Self {
        let Ok(notice) = serde_json::to_value(notice) else {
            return Response::fail(StatusCode::INTERNAL_SERVER_ERROR)
                .error("unable to serialize notice to value");
        };

        match self.notices.is_none() {
            true => {
                self.notices = Some(vec![notice]);
            },
            false => {
                self.notices
                    .as_mut()
                    .unwrap()
                    .push(notice);
            },
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

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    /// Drives the `to_value`-fails branch. `f64::NAN` does NOT trigger it:
    /// `serde_json::to_value` maps NaN to `Null` instead of erroring.
    struct Unserializable;

    impl Serialize for Unserializable {
        fn serialize<S: serde::Serializer>(&self, _s: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("always fails"))
        }
    }

    #[test]
    fn success_defaults_to_200_with_bare_envelope() {
        let res = Response::<Value>::success();

        assert!(res.success);
        assert_eq!(res.get_status(), StatusCode::OK);
        assert_eq!(
            serde_json::to_value(&res).unwrap(),
            json!({ "success": true }),
            "payload/errors/warnings/notices must all be omitted when absent"
        );
    }

    #[test]
    fn fail_carries_the_given_status_and_no_other_fields() {
        let res = Response::<Value>::fail(StatusCode::IM_A_TEAPOT);

        assert!(!res.success);
        assert_eq!(res.get_status(), StatusCode::IM_A_TEAPOT);
        assert_eq!(
            serde_json::to_value(&res).unwrap(),
            json!({ "success": false })
        );
    }

    #[test]
    fn unauthorized_is_401_fail_envelope() {
        let res = Response::<Value>::unauthorized();

        assert_eq!(res.get_status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            serde_json::to_value(&res).unwrap(),
            json!({ "success": false })
        );
    }

    #[test]
    fn internal_error_is_500_fail_envelope() {
        let res = Response::<Value>::internal_error();

        assert_eq!(res.get_status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            serde_json::to_value(&res).unwrap(),
            json!({ "success": false })
        );
    }

    #[test]
    fn bad_request_is_400_fail_envelope() {
        let res = Response::<Value>::bad_request();

        assert_eq!(res.get_status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            serde_json::to_value(&res).unwrap(),
            json!({ "success": false })
        );
    }

    #[test]
    fn error_appends_under_plural_errors_key_in_order() {
        let res = Response::<Value>::bad_request()
            .error("first")
            .error(json!({ "code": "dup" }));

        assert_eq!(
            serde_json::to_value(&res).unwrap(),
            json!({
                "success": false,
                "errors": ["first", { "code": "dup" }]
            })
        );
    }

    #[test]
    fn warning_and_notice_append_under_plural_keys() {
        let res = Response::<Value>::success()
            .warning("check quota")
            .warning("second")
            .notice(json!({ "msg": "scheduled maintenance" }));

        assert_eq!(
            serde_json::to_value(&res).unwrap(),
            json!({
                "success": true,
                "warnings": ["check quota", "second"],
                "notices": [{ "msg": "scheduled maintenance" }]
            })
        );
    }

    #[test]
    fn full_envelope_keeps_all_five_keys() {
        let res = Response::<Value>::success()
            .payload(json!({ "id": 7 }))
            .error("boom")
            .warning("careful")
            .notice("fyi");

        assert_eq!(
            serde_json::to_value(&res).unwrap(),
            json!({
                "success": true,
                "payload": { "id": 7 },
                "errors": ["boom"],
                "warnings": ["careful"],
                "notices": ["fyi"]
            })
        );
    }

    #[test]
    fn status_override_does_not_touch_success_flag() {
        let res = Response::<Value>::success().status(StatusCode::CREATED);

        assert_eq!(res.get_status(), StatusCode::CREATED);
        assert_eq!(
            serde_json::to_value(&res).unwrap(),
            json!({ "success": true })
        );
    }

    #[test]
    fn payload_serializes_under_payload_key_for_typed_generic() {
        let res = Response::<Vec<String>>::success().payload(vec!["a".into(), "b".into()]);

        assert_eq!(
            serde_json::to_value(&res).unwrap(),
            json!({ "success": true, "payload": ["a", "b"] })
        );
    }

    #[test]
    fn error_that_cannot_serialize_resets_builder_to_500() {
        // BUG(pinned): when `error()`'s argument fails to serialize, the builder
        // discards everything accumulated so far (payload, prior errors, success
        // flag) and returns a brand-new fail(500) envelope.
        let res = Response::<Value>::success()
            .payload(json!({ "kept": false }))
            .error("first")
            .error(Unserializable);

        assert_eq!(res.get_status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            serde_json::to_value(&res).unwrap(),
            json!({
                "success": false,
                "errors": ["unable to serialize error to value"]
            })
        );
    }

    #[test]
    fn warning_and_notice_serialize_failures_reset_to_500_too() {
        // Same pinned reset path as `error()` above, with their own messages.
        let w = Response::<Value>::success().warning(Unserializable);
        assert_eq!(
            serde_json::to_value(&w).unwrap(),
            json!({
                "success": false,
                "errors": ["unable to serialize warning to value"]
            })
        );
        assert_eq!(w.get_status(), StatusCode::INTERNAL_SERVER_ERROR);

        let n = Response::<Value>::success().notice(Unserializable);
        assert_eq!(
            serde_json::to_value(&n).unwrap(),
            json!({
                "success": false,
                "errors": ["unable to serialize notice to value"]
            })
        );
        assert_eq!(n.get_status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
