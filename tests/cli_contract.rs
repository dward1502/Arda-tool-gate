use std::process::Command;

#[test]
fn readonly_example_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_tool-gate"))
        .args([
            "check",
            "examples/readonly-tool.metadata.json",
            "examples/readonly-tool.invocation.json",
        ])
        .output()
        .expect("run tool-gate");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("\"decision\": \"allow\""));
}

#[test]
fn critical_example_exits_two() {
    let output = Command::new(env!("CARGO_BIN_EXE_tool-gate"))
        .args([
            "check",
            "examples/critical-review.metadata.json",
            "examples/critical-review.invocation.json",
        ])
        .output()
        .expect("run tool-gate");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn missing_trace_example_exits_three() {
    let output = Command::new(env!("CARGO_BIN_EXE_tool-gate"))
        .args([
            "check",
            "examples/mutating-idempotent.metadata.json",
            "examples/missing-trace-denied.invocation.json",
        ])
        .output()
        .expect("run tool-gate");
    assert_eq!(output.status.code(), Some(3));
}
