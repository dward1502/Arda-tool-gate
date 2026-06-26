use crate::receipt::{DecisionReceipt, PolicyCheck, PolicyCheckStatus};
use crate::types::{Decision, InvocationEnvelope, RiskLevel, ToolMetadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GatePolicy {
    pub require_trace_for_mutation: bool,
    pub require_trace_for_review: bool,
    pub require_idempotency_for_mutation: bool,
    pub critical_always_review: bool,
}

impl Default for GatePolicy {
    fn default() -> Self {
        Self {
            require_trace_for_mutation: true,
            require_trace_for_review: true,
            require_idempotency_for_mutation: true,
            critical_always_review: true,
        }
    }
}

pub fn evaluate_invocation(
    metadata: &ToolMetadata,
    envelope: &InvocationEnvelope,
    policy: &GatePolicy,
) -> DecisionReceipt {
    let mut checks = Vec::new();
    if let Err(error) = envelope.validate_for(metadata) {
        checks.push(check(
            "input_valid",
            PolicyCheckStatus::Fail,
            error.to_string(),
        ));
        return receipt(
            Decision::Deny,
            format!("invalid input: {error}"),
            metadata,
            envelope,
            checks,
        );
    }
    checks.push(check(
        "input_valid",
        PolicyCheckStatus::Pass,
        "metadata and invocation are structurally valid",
    ));

    if !target_allowed(&metadata.allowed_targets, &envelope.target) {
        checks.push(check(
            "target_allowed",
            PolicyCheckStatus::Fail,
            "target is outside allowed target patterns",
        ));
        return receipt(
            Decision::Deny,
            "target outside allowed scope",
            metadata,
            envelope,
            checks,
        );
    }
    checks.push(check(
        "target_allowed",
        PolicyCheckStatus::Pass,
        "target matches allowed scope",
    ));

    let trace_missing = envelope
        .trace_id
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty();
    if metadata.mutates_state && policy.require_trace_for_mutation && trace_missing {
        checks.push(check(
            "trace_present",
            PolicyCheckStatus::Fail,
            "mutating invocation requires trace_id",
        ));
        return receipt(
            Decision::Deny,
            "mutating invocation lacks trace evidence",
            metadata,
            envelope,
            checks,
        );
    }
    if metadata.requires_review && policy.require_trace_for_review && trace_missing {
        checks.push(check(
            "trace_present",
            PolicyCheckStatus::Fail,
            "reviewable invocation requires trace_id",
        ));
        return receipt(
            Decision::Deny,
            "reviewable invocation lacks trace evidence",
            metadata,
            envelope,
            checks,
        );
    }
    checks.push(check(
        "trace_present",
        PolicyCheckStatus::Pass,
        "trace requirement satisfied",
    ));

    if metadata.mutates_state && policy.require_idempotency_for_mutation && !metadata.idempotent {
        checks.push(check(
            "idempotency",
            PolicyCheckStatus::Fail,
            "mutating tool is not declared idempotent",
        ));
        return receipt(
            Decision::Deny,
            "mutating invocation is not idempotent",
            metadata,
            envelope,
            checks,
        );
    }
    checks.push(check(
        "idempotency",
        PolicyCheckStatus::Pass,
        "idempotency requirement satisfied",
    ));

    let review = metadata.requires_review
        || matches!(metadata.risk_level, RiskLevel::Critical) && policy.critical_always_review;
    if review {
        checks.push(check(
            "review_gate",
            PolicyCheckStatus::Review,
            "policy requires human review",
        ));
        return receipt(
            Decision::ReviewRequired,
            "policy requires human review",
            metadata,
            envelope,
            checks,
        );
    }

    checks.push(check(
        "review_gate",
        PolicyCheckStatus::Pass,
        "no review gate matched",
    ));
    receipt(
        Decision::Allow,
        "invocation passed policy checks",
        metadata,
        envelope,
        checks,
    )
}

fn target_allowed(patterns: &[String], target: &str) -> bool {
    patterns.iter().any(|pattern| {
        let pattern = pattern.trim();
        pattern == "*"
            || pattern == target
            || pattern
                .strip_suffix('*')
                .is_some_and(|prefix| target.starts_with(prefix))
    })
}

fn check(check_id: &str, status: PolicyCheckStatus, message: impl Into<String>) -> PolicyCheck {
    PolicyCheck {
        check_id: check_id.to_string(),
        status,
        message: message.into(),
    }
}

fn receipt(
    decision: Decision,
    reason: impl Into<String>,
    metadata: &ToolMetadata,
    envelope: &InvocationEnvelope,
    policy_checks: Vec<PolicyCheck>,
) -> DecisionReceipt {
    DecisionReceipt {
        schema_version: "tool-gate.decision.v1".to_string(),
        decision,
        reason: reason.into(),
        risk_level: metadata.risk_level,
        requires_review: matches!(decision, Decision::ReviewRequired),
        trace_id: envelope.trace_id.clone(),
        policy_checks,
    }
}
