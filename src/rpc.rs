//! Solana RPC client for fetching transaction data.

use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::signature::Signature;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding,
};

/// Fetch a transaction by signature from the configured RPC endpoint.
///
/// Uses `jsonParsed` encoding so that SPL Token / System program instructions
/// come pre-parsed by the RPC node. For unknown programs the data stays as
/// base64-encoded bytes — we decode those with IDL matching.
pub async fn fetch_transaction(
    rpc_url: &str,
    signature: &str,
) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
    let client = RpcClient::new(rpc_url.to_string());

    let sig: Signature = signature
        .parse()
        .context("Invalid transaction signature format")?;

    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::JsonParsed),
        max_supported_transaction_version: Some(0),
        ..Default::default()
    };

    let tx = client
        .get_transaction_with_config(&sig, config)
        .context("Failed to fetch transaction from RPC")?;

    Ok(tx)
}

/// Fetch a program IDL from a known source.
///
/// In a production deployment this would query:
/// 1. A local IDL registry (user-supplied JSON files)
/// 2. Anchor.so / Solscan IDL APIs
/// 3. On-chain IDL accounts (Anchor's `anchor idl` pattern)
///
/// For now returns None — the decoder falls back to raw hex display
/// for programs without IDLs.
pub async fn fetch_idl(_program_id: &str) -> Option<String> {
    // ponytail: IDL fetching from chain/registry deferred.
    // Core value prop — decoding known programs — works via program_db.rs.
    // Add when users want custom IDL support via `solanaut idl add <file.json>`.
    None
}
