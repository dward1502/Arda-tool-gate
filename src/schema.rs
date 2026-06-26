pub fn schema(name: &str) -> Option<&'static str> {
    match name {
        "tool-metadata" => Some(include_str!("../schemas/tool-metadata.schema.json")),
        "invocation-envelope" => Some(include_str!("../schemas/invocation-envelope.schema.json")),
        "decision-receipt" => Some(include_str!("../schemas/decision-receipt.schema.json")),
        _ => None,
    }
}
