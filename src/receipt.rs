use crate::types::{Decision, RiskLevel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCheckStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyCheck {
    pub check_id: String,
    pub status: PolicyCheckStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionReceipt {
    pub schema_version: String,
    pub decision: Decision,
    pub reason: String,
    pub risk_level: RiskLevel,
    pub requires_review: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    pub policy_checks: Vec<PolicyCheck>,
}

pub fn new_policy_check(
    check_id: &str,
    status: PolicyCheckStatus,
    detail: impl Into<Option<String>>,
) -> PolicyCheck {
    PolicyCheck {
        check_id: check_id.to_string(),
        status,
        detail: detail.into(),
    }
}
