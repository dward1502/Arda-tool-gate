use clap::{Parser, Subcommand};
use std::{fs, process};
use tool_gate::{evaluate_invocation, schema, GatePolicy, InvocationEnvelope, ToolMetadata};

#[derive(Debug, Parser)]
#[command(name = "tool-gate")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Check {
        metadata: String,
        invocation: String,
    },
    Schema {
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Command::Check {
            metadata,
            invocation,
        } => run_check(&metadata, &invocation),
        Command::Schema { name } => run_schema(&name),
    };
    process::exit(code);
}

fn run_check(metadata_path: &str, invocation_path: &str) -> i32 {
    let metadata: ToolMetadata = match read_json(metadata_path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{error}");
            return 65;
        }
    };
    let invocation: InvocationEnvelope = match read_json(invocation_path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{error}");
            return 65;
        }
    };
    let receipt = evaluate_invocation(&metadata, &invocation, &GatePolicy::default());
    match serde_json::to_string_pretty(&receipt) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("failed to serialize receipt: {error}");
            return 70;
        }
    }
    receipt.exit_code()
}

fn run_schema(name: &str) -> i32 {
    match schema::schema(name) {
        Some(schema) => {
            println!("{schema}");
            0
        }
        None => {
            eprintln!("unknown schema: {name}");
            64
        }
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("failed to read {path}: {error}"))?;
    serde_json::from_str(&content).map_err(|error| format!("failed to parse {path}: {error}"))
}
