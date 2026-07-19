//! `solanaut` — Human-readable Solana transaction decoder.
//!
//! ```text
//! solanaut decode <SIGNATURE>
//! solanaut decode <SIGNATURE> --json
//! solanaut decode <SIGNATURE> --rpc https://api.mainnet-beta.solana.com
//! ```

use clap::{Parser, Subcommand};
use std::collections::HashMap;

mod program_db;
mod rpc;
mod decoder;
mod idl;
mod display;

use idl::Idl;

/// Solanaut — decode Solana transactions into human-readable form.
#[derive(Parser)]
#[command(name = "solanaut", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Decode a transaction by its signature.
    Decode {
        /// Transaction signature (base58-encoded).
        signature: String,

        /// Output as JSON instead of terminal-formatted text.
        #[arg(long, short)]
        json: bool,

        /// Solana RPC endpoint.
        #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc: String,
    },

    /// List known Solana program addresses.
    Programs,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Decode { signature, json, rpc } => {
            handle_decode(&signature, json, &rpc).await?;
        }
        Command::Programs => {
            handle_programs();
        }
    }

    Ok(())
}

async fn handle_decode(signature: &str, json: bool, rpc_url: &str) -> anyhow::Result<()> {
    println!(
        "{} Fetching {} ...",
        "→".bright_blue(),
        signature.yellow()
    );

    let tx = rpc::fetch_transaction(rpc_url, signature).await?;

    // ponytail: IDL map is empty for now — all programs decode as Raw.
    // Add IDL load step when custom IDL support lands.
    let idl_map: HashMap<solana_sdk::pubkey::Pubkey, Idl> = HashMap::new();

    let decoded = decoder::decode_transaction(&tx, &idl_map);

    if json {
        display::print_json(&decoded);
    } else {
        display::print_transaction(&decoded);
    }

    Ok(())
}

fn handle_programs() {
    let map = program_db::known_programs();
    println!("{}", "Known Solana Programs:".bright_blue().bold());
    println!();

    // Group by kind
    let mut programs: Vec<_> = map.iter().collect();
    programs.sort_by_key(|(_, info)| info.kind);

    let mut current_kind = "";
    for (addr, info) in &programs {
        if info.kind != current_kind {
            current_kind = info.kind;
            println!("  {}", current_kind.bright_white().bold());
        }
        println!(
            "    {:<44}  {}",
            info.name.cyan(),
            addr.to_string().dimmed()
        );
    }

    println!();
    println!(
        "{}",
        format!("{} programs indexed.", map.len()).dimmed()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_decode() {
        let args = Cli::try_parse_from(["solanaut", "decode", "abc123"]);
        assert!(args.is_ok());
    }

    #[test]
    fn test_cli_parse_decode_json() {
        let args = Cli::try_parse_from(["solanaut", "decode", "abc123", "--json"]);
        assert!(args.is_ok());
    }

    #[test]
    fn test_cli_parse_programs() {
        let args = Cli::try_parse_from(["solanaut", "programs"]);
        assert!(args.is_ok());
    }
}
