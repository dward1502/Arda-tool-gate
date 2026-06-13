# tool-gate

Policy-gated tool invocation receipts for autonomous agents.

`tool-gate` answers one question before an agent runs a tool: should this invocation be allowed, denied, or held for review? It accepts generic JSON metadata and invocation envelopes, evaluates deterministic policy rules, and emits a machine-readable decision receipt.

It does not execute tools, sandbox processes, call model providers, or require a specific agent framework.

## Quick start

```bash
cargo run -- check examples/readonly-tool.metadata.json examples/readonly-tool.invocation.json
```

Expected result: a `tool-gate.decision.v1` JSON receipt with `"decision": "allow"`.

Review-required and denied examples use distinct exit codes:

```bash
cargo run -- check examples/critical-review.metadata.json examples/critical-review.invocation.json
cargo run -- check examples/mutating-idempotent.metadata.json examples/missing-trace-denied.invocation.json
```

## CLI

```bash
tool-gate check <tool-metadata.json> <invocation-envelope.json>
tool-gate schema tool-metadata
tool-gate schema invocation-envelope
tool-gate schema decision-receipt
```

Exit codes:

- `0`: decision generated and decision is `allow`
- `2`: decision generated and decision is `review_required`
- `3`: decision generated and decision is `deny`
- `64`: invalid CLI usage
- `65`: invalid JSON or invalid policy input
- `70`: internal error

## Library usage

```rust
use tool_gate::{evaluate_invocation, PolicyConfig};

let receipt = evaluate_invocation(&metadata, &envelope, &PolicyConfig::default())?;
```

## Core concepts

- Tool metadata: stable description of a tool's risk, side effects, idempotency, review requirement, and allowed targets.
- Invocation envelope: caller-supplied request context, including caller, target, trace id, and input summary.
- Decision receipt: JSON output that records `allow`, `deny`, or `review_required` plus policy checks.

## Security posture

`tool-gate` is a policy preflight helper, not a sandbox. The caller remains responsible for authentication, authorization, process isolation, secret handling, and execution.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
```
