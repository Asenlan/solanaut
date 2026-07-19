# solanaut

Human-readable Solana transaction decoder. Paste a transaction signature, see what actually happened — no more squinting at base64 blobs in block explorers.

```
$ solanaut decode 5nS9NpJ...xK3

══════════════════════════════════════════
Transaction:  5nS9NpJ...xK3
Slot:     312,458,921
Fee:      5,000 lamports
Status:   ✓ Success
Signer:   7xK2V...sender
──────────────────────────────────────────

  #1  Transfer
  Program:    SPL Token
  Data:
    source:       7xK2V... (addr)
    destination:  9yM8F... (addr)
    amount:       1,500

  #2  swap
  Program:    Jupiter V6 (DEX Aggregator)
  Data (raw):
    discriminator:   5f81e9ab fedc2103
    hex:             5f81e9ab fedc2103 ...
                     (264 bytes total)

  #3  transfer
  Program:    System Program (Native)
  Data:
    lamports:       100,000 lamports
```

## Features

- **Known program database** — 25+ major Solana programs identified by name (Jupiter, Orca, Raydium, Pump.fun, Jito, Metaplex, etc.)
- **Parsed instruction support** — SPL Token / System program instructions come pre-parsed
- **Anchor discriminator matching** — maps 8-byte discriminators to instruction names when IDL available
- **Borsh field decoding** — decodes common Anchor instruction args (u8-u128, bool, Pubkey, String)
- **JSON output mode** — `--json` flag for piping to `jq` or scripts
- **Colored terminal output** — addresses, amounts, status clearly differentiated

## Installation

```bash
cargo install --git https://github.com/user/solanaut
```

Or build from source:

```bash
git clone https://github.com/user/solanaut
cd solanaut
cargo build --release
```

## Usage

```bash
# Decode a transaction
solanaut decode <SIGNATURE>

# JSON output
solanaut decode <SIGNATURE> --json

# Custom RPC endpoint
solanaut decode <SIGNATURE> --rpc https://your-rpc.com

# List known programs
solanaut programs
```

## Architecture

```
src/
├── main.rs       # CLI entry point (clap)
├── rpc.rs        # Solana RPC client
├── decoder.rs    # Core decoding engine
├── idl.rs        # Anchor IDL types + discriminator
├── program_db.rs # Known program address → name lookup
└── display.rs    # Terminal-formatted output
```

## Known Programs

25+ programs indexed with name and category:

| Category | Programs |
|----------|----------|
| Native | System, Vote, Stake, Compute Budget, Address Lookup Table |
| Token | SPL Token, Token-2022, Associated Token Account |
| DEX Aggregator | Jupiter V4/V6 |
| DEX | Orca Whirlpool, Orca Token Swap V2, Raydium AMM V4, Raydium CLMM |
| Memecoin | Pump.fun |
| NFT | Metaplex Token Metadata, Magic Eden |
| Liquid Staking | Marinade Finance, Jito |
| Lending | Solend, Kamino |
| Perpetuals | Drift Protocol |
| Bridge | Wormhole, Wormhole Token Bridge |
| Utility | SPL Memo, SPL Name Service |

## Tech Stack

- **Rust** — zero-cost abstractions, great for CLI tools
- **solana-client / solana-sdk** — official Solana crates (v2)
- **clap** — declarative CLI argument parsing
- **colored** — terminal color support
- **sha2** — Anchor discriminator computation
- **tokio** — async runtime for RPC calls

## Roadmap

- [ ] IDL registry: `solanaut idl add <program-id> <idl.json>`
- [ ] On-chain IDL fetching from Anchor programs
- [ ] Token amount formatting with decimals
- [ ] Block explorer URL generation
- [ ] `solanaut watch` — stream transactions in real-time
- [ ] Transaction simulation mode

## License

MIT
