use crate::receipt::{new_policy_check, DecisionReceipt, PolicyCheck, PolicyCheckStatus};
use crate::types::{Decision, InvocationEnvelope, RiskLevel, ToolMetadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyConfig {
    pub require_trace_for_mutation: bool,
    pub require_trace_for_review: bool,
    pub critical_requires_review: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            require_trace_for_mutation: true,
            require_trace_for_review: true,
            critical_requires_review: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    EmptyField(&'static str),
    ToolIdMismatch,
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "required field is empty: {field}"),
            Self::ToolIdMismatch => write!(f, "invocation tool_id does not match metadata tool_id"),
        }
    }
}

impl std::error::Error for PolicyError {}

pub fn evaluate_invocation(
    metadata: &ToolMetadata,
    envelope: &InvocationEnvelope,
    policy: &PolicyConfig,
) -> Result<DecisionReceipt, PolicyError> {
    validate_metadata(metadata)?;
    validate_envelope(envelope)?;
    if metadata.tool_id != envelope.tool_id {
        return Err(PolicyError::ToolIdMismatch);
    }

    let mut checks = Vec::new();
    let trace_present = envelope
        .trace_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    push_check(&mut checks, "tool_id_match", true, None);

    let target_allowed = target_is_allowed(&metadata.allowed_targets, &envelope.target);
    push_check(
        &mut checks,
        "target_allowed",
        target_allowed,
        if target_allowed {
            None
        } else {
            Some("target was outside allowed_targets".to_string())
        },
    );

    let trace_required = (metadata.mutates_state && policy.require_trace_for_mutation)
        || ((metadata.requires_review || metadata.risk_level == RiskLevel::Critical)
            && policy.require_trace_for_review);
    push_check(
        &mut checks,
        "trace_present",
        !trace_required || trace_present,
        if trace_required && !trace_present {
            Some("trace_id is required for this invocation".to_string())
        } else {
            None
        },
    );

    push_check(
        &mut checks,
        "idempotency_safe",
        !metadata.mutates_state || metadata.idempotent,
        if metadata.mutates_state && !metadata.idempotent {
            Some("mutating tools must be marked idempotent to be auto-allowed".to_string())
        } else {
            None
        },
    );

    let requires_review = metadata.requires_review
        || (policy.critical_requires_review && metadata.risk_level == RiskLevel::Critical);
    push_check(
        &mut checks,
        "risk_allowed",
        !requires_review,
        if requires_review {
            Some("policy requires review before execution".to_string())
        } else {
            None
        },
    );

    let failed_hard_gate = checks.iter().any(|check| {
        matches!(check.status, PolicyCheckStatus::Fail)
            && check.check_id != "risk_allowed"
            && check.check_id != "idempotency_safe"
    });

    let (decision, reason) = if failed_hard_gate || (metadata.mutates_state && !metadata.idempotent)
    {
        (Decision::Deny, "policy denied invocation".to_string())
    } else if requires_review {
        (
            Decision::ReviewRequired,
            "policy requires human review".to_string(),
        )
    } else if metadata.mutates_state {
        (
            Decision::Allow,
            "mutating idempotent invocation with required trace evidence".to_string(),
        )
    } else {
        (
            Decision::Allow,
            "readonly invocation within policy".to_string(),
        )
    };

    Ok(DecisionReceipt {
        schema_version: "tool-gate.decision.v1".to_string(),
        decision,
        reason,
        risk_level: metadata.risk_level,
        requires_review,
        trace_id: envelope.trace_id.clone(),
        policy_checks: checks,
    })
}

fn validate_metadata(metadata: &ToolMetadata) -> Result<(), PolicyError> {
    required("tool_id", &metadata.tool_id)?;
    required("description", &metadata.description)?;
    Ok(())
}

fn validate_envelope(envelope: &InvocationEnvelope) -> Result<(), PolicyError> {
    required("invocation_id", &envelope.invocation_id)?;
    required("tool_id", &envelope.tool_id)?;
    required("requested_at_utc", &envelope.requested_at_utc)?;
    required("caller", &envelope.caller)?;
    required("input_summary", &envelope.input_summary)?;
    required("target", &envelope.target)?;
    Ok(())
}

fn required(field: &'static str, value: &str) -> Result<(), PolicyError> {
    if value.trim().is_empty() {
        Err(PolicyError::EmptyField(field))
    } else {
        Ok(())
    }
}

fn push_check(checks: &mut Vec<PolicyCheck>, check_id: &str, pass: bool, detail: Option<String>) {
    checks.push(new_policy_check(
        check_id,
        if pass {
            PolicyCheckStatus::Pass
        } else {
            PolicyCheckStatus::Fail
        },
        detail,
    ));
}

fn target_is_allowed(allowed_targets: &[String], target: &str) -> bool {
    if allowed_targets.is_empty() {
        return true;
    }
    allowed_targets.iter().any(|allowed| {
        allowed == "*" || target == allowed || target.starts_with(allowed.trim_end_matches('*'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> ToolMetadata {
        ToolMetadata {
            tool_id: "demo.readonly".to_string(),
            description: "read only demo".to_string(),
            risk_level: RiskLevel::Low,
            mutates_state: false,
            idempotent: true,
            requires_review: false,
            allowed_targets: vec!["workspace/".to_string()],
        }
    }

    fn envelope() -> InvocationEnvelope {
        InvocationEnvelope {
            invocation_id: "invoke-1".to_string(),
            tool_id: "demo.readonly".to_string(),
            requested_at_utc: "2026-06-13T00:00:00Z".to_string(),
            caller: "agent.demo".to_string(),
            input_summary: "inspect workspace".to_string(),
            trace_id: Some("trace-1".to_string()),
            target: "workspace/file.txt".to_string(),
        }
    }

    #[test]
    fn readonly_tool_is_allowed() {
        let receipt =
            evaluate_invocation(&metadata(), &envelope(), &PolicyConfig::default()).unwrap();
        assert_eq!(receipt.decision, Decision::Allow);
        assert!(!receipt.requires_review);
    }

    #[test]
    fn missing_trace_denies_mutation() {
        let mut metadata = metadata();
        metadata.mutates_state = true;
        metadata.risk_level = RiskLevel::Medium;
        let mut envelope = envelope();
        envelope.trace_id = None;
        let receipt = evaluate_invocation(&metadata, &envelope, &PolicyConfig::default()).unwrap();
        assert_eq!(receipt.decision, Decision::Deny);
    }

    #[test]
    fn critical_requires_review() {
        let mut metadata = metadata();
        metadata.risk_level = RiskLevel::Critical;
        let receipt =
            evaluate_invocation(&metadata, &envelope(), &PolicyConfig::default()).unwrap();
        assert_eq!(receipt.decision, Decision::ReviewRequired);
        assert!(receipt.requires_review);
    }
}
