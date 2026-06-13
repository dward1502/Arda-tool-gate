use crate::{DECISION_RECEIPT_SCHEMA, INVOCATION_ENVELOPE_SCHEMA, TOOL_METADATA_SCHEMA};

pub fn schema_by_name(name: &str) -> Option<&'static str> {
    match name {
        "tool-metadata" => Some(TOOL_METADATA_SCHEMA),
        "invocation-envelope" => Some(INVOCATION_ENVELOPE_SCHEMA),
        "decision-receipt" => Some(DECISION_RECEIPT_SCHEMA),
        _ => None,
    }
}
