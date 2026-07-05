//! Helpers for asserting SdkWorkApiResponse envelopes in HTTP integration tests.

use serde_json::Value;

pub fn success_code(value: &Value) -> bool {
    value.get("code").and_then(|code| code.as_i64()) == Some(0)
}

/// Unwrap a success `SdkWorkApiResponse` into the payload shape tests assert on.
///
/// - List responses return `data` (`items` + `pageInfo`).
/// - Single-resource responses return `data.item`.
/// - Other command payloads return bare `data`.
pub fn unwrap_success_payload(value: &Value) -> Value {
    assert!(
        success_code(value),
        "expected SdkWorkApiResponse code=0, got {value}"
    );
    let data = value["data"].clone();
    if data.get("items").is_some() {
        data
    } else if data.get("item").is_some() && !data["item"].is_null() {
        data["item"].clone()
    } else {
        data
    }
}

/// Like [`unwrap_success_payload`] but preserves non-success envelopes for error assertions.
pub fn unwrap_payload_or_envelope(value: &Value) -> Value {
    if success_code(value) {
        unwrap_success_payload(value)
    } else {
        value.clone()
    }
}

pub fn item<'a>(envelope: &'a Value) -> &'a Value {
    envelope
        .pointer("/data/item")
        .unwrap_or_else(|| panic!("expected SdkWorkApiResponse data.item, got {envelope}"))
}

pub fn items<'a>(envelope: &'a Value) -> &'a Value {
    envelope
        .pointer("/data/items")
        .unwrap_or_else(|| panic!("expected SdkWorkApiResponse data.items, got {envelope}"))
}

pub fn page_info<'a>(envelope: &'a Value) -> &'a Value {
    envelope
        .pointer("/data/pageInfo")
        .unwrap_or_else(|| panic!("expected SdkWorkApiResponse data.pageInfo, got {envelope}"))
}
