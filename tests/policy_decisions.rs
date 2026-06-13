use tool_gate::{
    evaluate_invocation, Decision, InvocationEnvelope, PolicyConfig, RiskLevel, ToolMetadata,
};

fn envelope(tool_id: &str) -> InvocationEnvelope {
    InvocationEnvelope {
        invocation_id: "invoke-test".into(),
        tool_id: tool_id.into(),
        requested_at_utc: "2026-06-13T00:00:00Z".into(),
        caller: "agent.test".into(),
        input_summary: "test invocation".into(),
        trace_id: Some("trace-test".into()),
        target: "workspace/file.txt".into(),
    }
}

fn metadata(
    tool_id: &str,
    risk_level: RiskLevel,
    mutates_state: bool,
    idempotent: bool,
) -> ToolMetadata {
    ToolMetadata {
        tool_id: tool_id.into(),
        description: "test tool".into(),
        risk_level,
        mutates_state,
        idempotent,
        requires_review: false,
        allowed_targets: vec!["workspace/".into()],
    }
}

#[test]
fn readonly_allows() {
    let receipt = evaluate_invocation(
        &metadata("tool.read", RiskLevel::Low, false, true),
        &envelope("tool.read"),
        &PolicyConfig::default(),
    )
    .unwrap();
    assert_eq!(receipt.decision, Decision::Allow);
}

#[test]
fn non_idempotent_mutation_denies() {
    let receipt = evaluate_invocation(
        &metadata("tool.write", RiskLevel::Medium, true, false),
        &envelope("tool.write"),
        &PolicyConfig::default(),
    )
    .unwrap();
    assert_eq!(receipt.decision, Decision::Deny);
}

#[test]
fn critical_reviews() {
    let receipt = evaluate_invocation(
        &metadata("tool.deploy", RiskLevel::Critical, true, true),
        &envelope("tool.deploy"),
        &PolicyConfig::default(),
    )
    .unwrap();
    assert_eq!(receipt.decision, Decision::ReviewRequired);
}
