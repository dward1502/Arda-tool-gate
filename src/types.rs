use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Allow,
    Deny,
    ReviewRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolMetadata {
    pub tool_id: String,
    pub description: String,
    pub risk_level: RiskLevel,
    pub mutates_state: bool,
    pub idempotent: bool,
    pub requires_review: bool,
    pub allowed_targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvocationEnvelope {
    pub invocation_id: String,
    pub tool_id: String,
    pub requested_at_utc: String,
    pub caller: String,
    pub input_summary: String,
    pub trace_id: Option<String>,
    pub target: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GateError {
    #[error("metadata field is blank: {0}")]
    BlankMetadataField(&'static str),
    #[error("invocation field is blank: {0}")]
    BlankInvocationField(&'static str),
    #[error("tool id mismatch: metadata={metadata_tool_id}, invocation={invocation_tool_id}")]
    ToolIdMismatch {
        metadata_tool_id: String,
        invocation_tool_id: String,
    },
}

impl ToolMetadata {
    pub fn validate(&self) -> Result<(), GateError> {
        if self.tool_id.trim().is_empty() {
            return Err(GateError::BlankMetadataField("tool_id"));
        }
        if self.description.trim().is_empty() {
            return Err(GateError::BlankMetadataField("description"));
        }
        Ok(())
    }
}

impl InvocationEnvelope {
    pub fn validate_for(&self, metadata: &ToolMetadata) -> Result<(), GateError> {
        metadata.validate()?;
        for (name, value) in [
            ("invocation_id", self.invocation_id.as_str()),
            ("tool_id", self.tool_id.as_str()),
            ("requested_at_utc", self.requested_at_utc.as_str()),
            ("caller", self.caller.as_str()),
            ("input_summary", self.input_summary.as_str()),
            ("target", self.target.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(GateError::BlankInvocationField(name));
            }
        }
        if self.tool_id != metadata.tool_id {
            return Err(GateError::ToolIdMismatch {
                metadata_tool_id: metadata.tool_id.clone(),
                invocation_tool_id: self.tool_id.clone(),
            });
        }
        Ok(())
    }
}
