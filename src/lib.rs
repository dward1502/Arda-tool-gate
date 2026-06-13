//! Policy-gated tool invocation receipts for autonomous agents.
//!
//! `tool-gate` evaluates caller-supplied tool metadata and invocation envelopes.
//! It returns a structured decision receipt, but never executes the tool itself.

pub mod policy;
pub mod receipt;
pub mod schema;
pub mod types;

pub use policy::{evaluate_invocation, PolicyConfig, PolicyError};
pub use receipt::{new_policy_check, DecisionReceipt, PolicyCheck, PolicyCheckStatus};
pub use types::{Decision, InvocationEnvelope, RiskLevel, ToolMetadata};

pub const TOOL_METADATA_SCHEMA: &str = include_str!("../schemas/tool-metadata.schema.json");
pub const INVOCATION_ENVELOPE_SCHEMA: &str =
    include_str!("../schemas/invocation-envelope.schema.json");
pub const DECISION_RECEIPT_SCHEMA: &str = include_str!("../schemas/decision-receipt.schema.json");

pub fn crate_identity() -> &'static str {
    "tool-gate"
}
