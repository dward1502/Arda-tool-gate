use tool_gate::{
    evaluate_invocation, Decision, GatePolicy, InvocationEnvelope, RiskLevel, ToolMetadata,
};

fn metadata(
    risk_level: RiskLevel,
    mutates_state: bool,
    idempotent: bool,
    requires_review: bool,
) -> ToolMetadata {
    ToolMetadata {
        tool_id: "demo.tool".into(),
        description: "demo".into(),
        risk_level,
        mutates_state,
        idempotent,
        requires_review,
        allowed_targets: vec!["scope/*".into()],
    }
}

fn envelope(trace_id: Option<&str>) -> InvocationEnvelope {
    InvocationEnvelope {
        invocation_id: "inv-1".into(),
        tool_id: "demo.tool".into(),
        requested_at_utc: "2026-06-13T00:00:00Z".into(),
        caller: "agent".into(),
        input_summary: "demo".into(),
        trace_id: trace_id.map(str::to_string),
        target: "scope/item".into(),
    }
}

#[test]
fn readonly_with_scope_is_allowed() {
    let receipt = evaluate_invocation(
        &metadata(RiskLevel::Low, false, true, false),
        &envelope(None),
        &GatePolicy::default(),
    );
    assert_eq!(receipt.decision, Decision::Allow);
}

#[test]
fn mutating_without_trace_is_denied() {
    let receipt = evaluate_invocation(
        &metadata(RiskLevel::Medium, true, true, false),
        &envelope(None),
        &GatePolicy::default(),
    );
    assert_eq!(receipt.decision, Decision::Deny);
    assert!(receipt.reason.contains("trace"));
}

#[test]
fn critical_traceable_operation_requires_review() {
    let receipt = evaluate_invocation(
        &metadata(RiskLevel::Critical, true, true, true),
        &envelope(Some("trace-1")),
        &GatePolicy::default(),
    );
    assert_eq!(receipt.decision, Decision::ReviewRequired);
    assert!(receipt.requires_review);
}
