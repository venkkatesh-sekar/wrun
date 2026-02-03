use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The WAT file to run
    wat_file: PathBuf,

    /// The method to call in the wasm module
    method: String,

    /// The instance type to use for running the wasm
    #[arg(short, long, value_enum)]
    instance_type: InstanceType,

    /// The URL of the testnet to use (only for the 'testnet' instance type).
    #[arg(long)]
    url: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum InstanceType {
    /// Use the wasmtime engine directly
    Wasmtime,
    /// Use the IC wasmtime embedder
    Embedder,
    /// Use the IC execution environment
    Execenv,
    /// Use PocketIc
    PocketIc,
    /// Use a testnet
    Testnet,
}

fn main() {
    let cli = Cli::parse();

    let wasm_bytes = match wat::parse_file(&cli.wat_file) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!(
                "Failed to parse WAT file '{}': {}",
                cli.wat_file.display(),
                e
            );
            std::process::exit(1);
        }
    };

    match cli.instance_type {
        InstanceType::Wasmtime => wrun::wasmtime::run_instance(wasm_bytes, cli.method),
        InstanceType::Embedder => wrun::embedder::run_instance(wasm_bytes, cli.method),
        InstanceType::Execenv => wrun::execenv::run_instance(wasm_bytes, cli.method),
        InstanceType::PocketIc => wrun::pocket_ic::run_instance(wasm_bytes, cli.method),
        InstanceType::Testnet => {
            // Use provided URL or default to a hardcoded testnet URL.
            let url = cli
                .url
                .expect("No URL provided for 'testnet' instance type");
            let use_mainnet = false;
            wrun::testnet::run_instance(url, use_mainnet, wasm_bytes, cli.method);
        }
    }
}
