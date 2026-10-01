use serde_json::Value;

/// A JSON document as the form textareas and the detail `<pre>` blocks both
/// show it: pretty-printed, or the raw rendering when it will not reformat.
pub fn json_text(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}
