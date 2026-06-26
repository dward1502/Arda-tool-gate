use crate::types::{Decision, RiskLevel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCheckStatus {
    Pass,
    Fail,
    Review,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyCheck {
    pub check_id: String,
    pub status: PolicyCheckStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionReceipt {
    pub schema_version: String,
    pub decision: Decision,
    pub reason: String,
    pub risk_level: RiskLevel,
    pub requires_review: bool,
    pub trace_id: Option<String>,
    pub policy_checks: Vec<PolicyCheck>,
}

impl DecisionReceipt {
    pub fn exit_code(&self) -> i32 {
        match self.decision {
            Decision::Allow => 0,
            Decision::ReviewRequired => 2,
            Decision::Deny => 3,
        }
    }
}
