use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use tool_gate::{
    evaluate_invocation, schema::schema_by_name, Decision, InvocationEnvelope, PolicyConfig,
    ToolMetadata,
};

#[derive(Debug, Parser)]
#[command(
    name = "tool-gate",
    about = "Evaluate policy gates for agent tool invocations"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Check {
        tool_metadata: PathBuf,
        invocation_envelope: PathBuf,
    },
    Schema {
        name: String,
    },
}

fn main() {
    let code = run();
    std::process::exit(code);
}

fn run() -> i32 {
    let args = Args::parse();
    match args.command {
        Command::Check {
            tool_metadata,
            invocation_envelope,
        } => check(tool_metadata, invocation_envelope),
        Command::Schema { name } => match schema_by_name(&name) {
            Some(schema) => {
                println!("{schema}");
                0
            }
            None => {
                eprintln!("unknown schema: {name}");
                64
            }
        },
    }
}

fn check(tool_metadata: PathBuf, invocation_envelope: PathBuf) -> i32 {
    let metadata: ToolMetadata = match read_json(&tool_metadata) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid tool metadata: {error}");
            return 65;
        }
    };
    let envelope: InvocationEnvelope = match read_json(&invocation_envelope) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid invocation envelope: {error}");
            return 65;
        }
    };
    match evaluate_invocation(&metadata, &envelope, &PolicyConfig::default()) {
        Ok(receipt) => {
            match serde_json::to_string_pretty(&receipt) {
                Ok(json) => println!("{json}"),
                Err(error) => {
                    eprintln!("failed to serialize decision receipt: {error}");
                    return 70;
                }
            }
            match receipt.decision {
                Decision::Allow => 0,
                Decision::ReviewRequired => 2,
                Decision::Deny => 3,
            }
        }
        Err(error) => {
            eprintln!("policy input error: {error}");
            65
        }
    }
}

fn read_json<T: serde::de::DeserializeOwned>(
    path: &PathBuf,
) -> Result<T, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}
