use serde_json::Value;
use std::process::Command;

#[test]
fn check_outputs_allow_receipt_json() {
    let exe = env!("CARGO_BIN_EXE_tool-gate");
    let output = Command::new(exe)
        .args([
            "check",
            "examples/readonly-tool.metadata.json",
            "examples/readonly-tool.invocation.json",
        ])
        .output()
        .expect("run tool-gate");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("receipt json");
    assert_eq!(json["schema_version"], "tool-gate.decision.v1");
    assert_eq!(json["decision"], "allow");
}

#[test]
fn missing_trace_uses_deny_exit_code() {
    let exe = env!("CARGO_BIN_EXE_tool-gate");
    let output = Command::new(exe)
        .args([
            "check",
            "examples/mutating-idempotent.metadata.json",
            "examples/missing-trace-denied.invocation.json",
        ])
        .output()
        .expect("run tool-gate");
    assert_eq!(output.status.code(), Some(3));
    let json: Value = serde_json::from_slice(&output.stdout).expect("receipt json");
    assert_eq!(json["decision"], "deny");
}
