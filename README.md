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
    hex:             ... (264 bytes total)
```

## Features

- **25+ Known Programs** — Jupiter, Orca, Raydium, Pump.fun, Jito, Metaplex, Magic Eden, and more
- **Parsed Instructions** — SPL Token / System Program instructions pre-parsed by RPC
- **Anchor Discriminator Matching** — Matches 8-byte discriminators via sha256("global:<name>")[:8]
- **Borsh Field Decoding** — Decodes common Anchor args (u8-u128, bool, Pubkey, String)
- **JSON Output** — `--json` flag for piping to `jq` or scripts
- **Colored Terminal** — Addresses, amounts, status clearly differentiated

## Installation

```bash
cargo install --git https://github.com/user/solanaut
```

## Usage

```bash
solanaut decode <SIGNATURE>           # Decode a transaction
solanaut decode <SIGNATURE> --json    # JSON output
solanaut decode <SIGNATURE> --rpc <URL>  # Custom RPC
solanaut programs                     # List known programs
```

## Architecture

```
src/
├── main.rs       # CLI entry point (clap)
├── rpc.rs        # Solana RPC client
├── decoder.rs    # Core decoding engine
├── idl.rs        # Anchor IDL types + discriminator
├── program_db.rs # Known program address database
└── display.rs    # Terminal + JSON formatters
```

## Tech Stack

- **Rust** — Zero-cost abstractions
- **solana-client / solana-sdk** — Official Solana crates (v2)
- **clap** — Declarative CLI argument parsing
- **sha2** — Anchor discriminator computation

## License

MIT
