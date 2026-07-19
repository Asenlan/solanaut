//! Known Solana program address → human name mapping.
//!
//! Each entry maps a well-known program address to a human-readable name,
//! an optional label for the program type, and optionally the token/symbol
//! it is associated with.

use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

pub struct ProgramInfo {
    pub name: &'static str,
    pub kind: &'static str,
}

/// Build a lookup from program address → (human name, kind label).
pub fn known_programs() -> HashMap<Pubkey, ProgramInfo> {
    let mut map = HashMap::new();

    macro_rules! add {
        ($addr:literal, $name:literal, $kind:literal) => {
            map.insert(
                $addr.parse::<Pubkey>().expect("invalid program address"),
                ProgramInfo {
                    name: $name,
                    kind: $kind,
                },
            );
        };
    }

    // --- System & Native ---
    add!(
        "11111111111111111111111111111111",
        "System Program",
        "Native"
    );
    add!(
        "Vote111111111111111111111111111111111111111",
        "Vote Program",
        "Native"
    );
    add!(
        "Stake11111111111111111111111111111111111111",
        "Stake Program",
        "Native"
    );
    add!(
        "ComputeBudget111111111111111111111111111111",
        "Compute Budget",
        "Native"
    );
    add!(
        "AddressLookupTab1e1111111111111111111111111",
        "Address Lookup Table",
        "Native"
    );

    // --- SPL Token ---
    add!(
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "SPL Token",
        "Token"
    );
    add!(
        "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
        "SPL Token-2022",
        "Token"
    );
    add!(
        "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
        "Associated Token Account",
        "Token"
    );

    // --- DeFi ---
    add!(
        "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
        "Jupiter V6",
        "DEX Aggregator"
    );
    add!(
        "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB",
        "Jupiter V4",
        "DEX Aggregator"
    );
    add!(
        "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc",
        "Orca Whirlpool",
        "DEX"
    );
    add!(
        "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP",
        "Orca Token Swap V2",
        "DEX"
    );
    add!(
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
        "Raydium AMM V4",
        "DEX"
    );
    add!(
        "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK",
        "Raydium CLMM",
        "DEX"
    );
    add!(
        "6EF8rrecthR5DqzonqLNuT4NvvX6L9cMT7xU5gEFBHwb",
        "Pump.fun",
        "Memecoin Launchpad"
    );
    add!(
        "Meteo1wTbBUm7GKu9S4JMpnLJrkSAhRAeDYc4eBxK3J",
        "Metaplex Token Metadata",
        "NFT"
    );
    add!(
        "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
        "Metaplex Token Metadata (old)",
        "NFT"
    );
    add!(
        "MEisE1HzehtrDpAAT8LH8yJnQ5Tghj4bXiqsP9VMMU8",
        "Magic Eden",
        "NFT Marketplace"
    );

    // --- Lending / Staking ---
    add!(
        "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD",
        "Marinade Finance",
        "Liquid Staking"
    );
    add!(
        "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb",
        "Jito",
        "Liquid Staking / MEV"
    );
    add!(
        "LendZx1HyFPPJNSi2vRRbsTLT5nbLHWPwB1J8qU9RRr",
        "Solend",
        "Lending"
    );
    add!(
        "KLend2g3cP87fffoy8q1mQqGK6RXwX6oohUFQuf4VWT",
        "Kamino Lending",
        "Lending"
    );
    add!(
        "DriftBLoR5MEspPQ3hLpZNjHB8jcnRAhr5RBmXMiiVjR",
        "Drift Protocol",
        "Perpetuals"
    );

    // --- Bridge / Wormhole ---
    add!(
        "worm2ZoG2kUd4vFXhvjh93UUH596ayRfgQ2MgjNMTth",
        "Wormhole Bridge",
        "Bridge"
    );
    add!(
        "wormDTUJ6AWPNvk59vGQbDvGJmqbDTdgWgA2LqMshn",
        "Wormhole Token Bridge",
        "Bridge"
    );

    // --- Utility ---
    add!(
        "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "SPL Memo",
        "Utility"
    );
    add!(
        "Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo",
        "SPL Memo v1",
        "Utility"
    );
    add!(
        "noopb9bkMVfRPU8AsbpTUg8FWAxkVtDGkuY6AtG6kd9",
        "Noop (log only)",
        "Utility"
    );
    add!(
        "nameAxQRRBnd4KLmB3QBK7nDJaQ3Vc4QpCRDjD5WX4V",
        "SPL Name Service",
        "Utility"
    );

    map
}

/// Look up a program by address and return a display-friendly label.
pub fn lookup_program(pubkey: &Pubkey) -> String {
    let map = known_programs();
    match map.get(pubkey) {
        Some(info) => format!("{} ({})", info.name, info.kind),
        None => pubkey.to_string(),
    }
}

/// Return just the program name if known, otherwise the address.
pub fn program_name(pubkey: &Pubkey) -> String {
    let map = known_programs();
    match map.get(pubkey) {
        Some(info) => info.name.to_string(),
        None => pubkey.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_program_known() {
        let sys: Pubkey = "11111111111111111111111111111111".parse().unwrap();
        let name = program_name(&sys);
        assert_eq!(name, "System Program");
    }

    #[test]
    fn test_jupiter_known() {
        let jup: Pubkey = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
            .parse()
            .unwrap();
        let info = lookup_program(&jup);
        assert!(info.contains("Jupiter"));
    }

    #[test]
    fn test_unknown_program_returns_address() {
        let unknown = solana_sdk::pubkey::Pubkey::new_unique();
        let name = program_name(&unknown);
        assert_eq!(name, unknown.to_string());
    }
}
