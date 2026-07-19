//! Output formatters for decoded transaction data.

use crate::decoder::{DecodedData, DecodedInstruction, DecodedTransaction};
use colored::*;

/// Print a decoded transaction in a human-readable format.
pub fn print_transaction(tx: &DecodedTransaction) {
    print_header(tx);
    println!();

    for ix in &tx.instructions {
        print_instruction(ix);
    }

    if !tx.logs.is_empty() {
        println!();
        println!("{}", "── Logs ──".dimmed());
        // Only show program logs, not the verbose system messages
        let filtered: Vec<&String> = tx
            .logs
            .iter()
            .filter(|l| {
                !l.starts_with("Program ")
                    || l.contains("invoke")
                    || l.contains("success")
                    || l.contains("consumed")
            })
            .collect();

        if filtered.len() <= 20 {
            for log in &filtered {
                println!("  {}", log.dimmed());
            }
        } else {
            for log in filtered.iter().take(10) {
                println!("  {}", log.dimmed());
            }
            println!(
                "  {}",
                format!("... and {} more log lines", filtered.len() - 10).dimmed()
            );
        }
    }
}

/// Print a decoded transaction as JSON.
pub fn print_json(tx: &DecodedTransaction) {
    let json = serde_json::json!({
        "signature": tx.signature,
        "slot": tx.slot,
        "fee": tx.fee,
        "status": tx.status,
        "signer": tx.signer,
        "instructions": tx.instructions.iter().map(|ix| {
            let data_json = match &ix.data {
                DecodedData::Parsed { kind, info } => {
                    let info_map: serde_json::Map<String, serde_json::Value> = info
                        .iter()
                        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                        .collect();
                    serde_json::json!({
                        "type": "parsed",
                        "kind": kind,
                        "info": info_map,
                    })
                }
                DecodedData::Raw { base64, hex, discriminator } => {
                    serde_json::json!({
                        "type": "raw",
                        "base64": base64,
                        "hex": hex,
                        "discriminator": discriminator,
                    })
                }
                DecodedData::IdlDecoded { instruction_name, args } => {
                    let args_map: serde_json::Map<String, serde_json::Value> = args
                        .iter()
                        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                        .collect();
                    serde_json::json!({
                        "type": "idl_decoded",
                        "instruction": instruction_name,
                        "args": args_map,
                    })
                }
            };

            serde_json::json!({
                "index": ix.index,
                "program": ix.program,
                "instruction": ix.instruction_name,
                "accounts": ix.accounts.iter().map(|a| {
                    serde_json::json!({
                        "pubkey": a.pubkey,
                        "label": a.label,
                        "is_signer": a.is_signer,
                        "is_writable": a.is_writable,
                    })
                }).collect::<Vec<_>>(),
                "data": data_json,
            })
        }).collect::<Vec<_>>(),
    });

    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}

fn print_header(tx: &DecodedTransaction) {
    println!("{}", "══════════════════════════════════════════".bright_blue());
    println!(
        "{}  {}",
        "Transaction:".bright_blue().bold(),
        tx.signature.yellow()
    );
    println!("{}     {}", "Slot:".dimmed(), tx.slot);
    println!("{}    {}", "Fee:".dimmed(), format_lamports(tx.fee));
    println!(
        "{} {}",
        "Status:".dimmed(),
        match tx.status.as_str() {
            "Ok" => "✓ Success".green(),
            s if s.contains("err") => format!("✗ {}", s).red(),
            _ => tx.status.normal(),
        }
    );
    println!("{}  {}", "Signer:".dimmed(), tx.signer);
    println!("{}", "──────────────────────────────────────────".bright_blue());
}

fn print_instruction(ix: &DecodedInstruction) {
    println!();
    println!(
        "  {}  {}",
        format!("#{}", ix.index + 1).bright_blue().bold(),
        ix.instruction_name.bright_white().bold()
    );
    println!("  {}    {}", "Program:".dimmed(), ix.program.cyan());

    match &ix.data {
        DecodedData::Parsed { kind: _, info } => {
            if !info.is_empty() {
                println!("  {}", "Data:".dimmed());
                for (key, val) in info {
                    // Pretty-print common fields
                    let display_val = match key.as_str() {
                        "amount" | "tokenAmount" => format_token_amount(val),
                        "lamports" => format!("{} lamports", val),
                        "source" | "destination" | "mint" | "owner" | "authority" => {
                            format!("{} (addr)", address(&val))
                        }
                        _ => val.clone(),
                    };
                    println!("    {:<16} {}", format!("{}:", key).dimmed(), display_val);
                }
            }
        }
        DecodedData::Raw { base64, hex, discriminator } => {
            println!("  {}", "Data (raw):".dimmed());
            if let Some(disc) = discriminator {
                println!("    {:<16} {}", "discriminator:".dimmed(), disc);
            }
            println!("    {:<16} {}", "hex:".dimmed(), hex);
            if hex.len() > 64 {
                println!(
                    "    {:<16} {}",
                    "...".dimmed(),
                    format!("({} bytes total)", hex.len() / 3 + 1).dimmed()
                );
            }
        }
        DecodedData::IdlDecoded { instruction_name, args } => {
            if !args.is_empty() {
                println!("  {}", "Args:".dimmed());
                for (name, val) in args {
                    println!("    {:<20} {}", format!("{}:", name).dimmed(), val);
                }
            }
        }
    }

    // Print accounts
    if !ix.accounts.is_empty() {
        println!("  {}", "Accounts:".dimmed());
        for acc in &ix.accounts {
            let label_str = if !acc.label.is_empty() && acc.label != acc.pubkey {
                format!(" {}", acc.label.dimmed())
            } else {
                String::new()
            };
            let flags = if acc.is_signer { "[S]" } else { "   " };
            let wflag = if acc.is_writable { "[W]" } else { "   " };
            println!(
                "    {} {} {} {}{}",
                flags.dimmed(),
                wflag.dimmed(),
                short_addr(&acc.pubkey).yellow(),
                acc.pubkey.dimmed(),
                label_str,
            );
        }
    }
}

fn format_lamports(lamports: u64) -> String {
    if lamports >= 1_000_000_000 {
        format!("{:.4} SOL ({} lamports)", lamports as f64 / 1_000_000_000.0, lamports)
    } else if lamports >= 1_000 {
        format!("{} lamports", lamports)
    } else {
        format!("{} lamports", lamports)
    }
}

fn format_token_amount(val: &str) -> String {
    val.to_string()
}

fn address(s: &str) -> String {
    s.to_string()
}

fn short_addr(addr: &str) -> String {
    if addr.len() > 12 {
        format!("{}..{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}
