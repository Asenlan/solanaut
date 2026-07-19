//! Core decoding engine — takes raw transaction data and produces
//! human-readable instruction descriptions.

use crate::program_db;
use crate::idl::{self, Idl};
use base64::Engine;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiCompiledInstruction, UiInstruction,
    UiMessage, UiParsedInstruction, UiPartiallyDecodedInstruction, UiTransactionStatusMeta,
};

/// A decoded instruction ready for display.
#[derive(Debug)]
pub struct DecodedInstruction {
    /// Instruction index in the transaction (0-based).
    pub index: usize,
    /// Human name of the program being called.
    pub program: String,
    /// Human-readable instruction name (e.g. "Transfer", "Swap").
    pub instruction_name: String,
    /// Account keys with labels where we can infer them.
    pub accounts: Vec<DecodedAccount>,
    /// The instruction data — either parsed or as raw bytes.
    pub data: DecodedData,
}

#[derive(Debug)]
pub struct DecodedAccount {
    pub pubkey: String,
    pub label: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Debug)]
pub enum DecodedData {
    /// Data was parsed by the RPC (e.g. SPL Token Transfer).
    Parsed {
        kind: String,
        info: Vec<(String, String)>,
    },
    /// Raw instruction data — base64-encoded bytes plus hex dump.
    Raw {
        base64: String,
        hex: String,
        discriminator: Option<String>,
    },
    /// Successfully decoded via IDL.
    IdlDecoded {
        instruction_name: String,
        args: Vec<(String, String)>,
    },
}

/// Top-level transaction decode result.
#[derive(Debug)]
pub struct DecodedTransaction {
    pub signature: String,
    pub slot: u64,
    pub fee: u64,
    pub status: String,
    pub signer: String,
    pub instructions: Vec<DecodedInstruction>,
    pub logs: Vec<String>,
}

/// Main entry point — decode a full transaction.
pub fn decode_transaction(tx: &EncodedConfirmedTransactionWithStatusMeta, idl_map: &std::collections::HashMap<Pubkey, Idl>) -> DecodedTransaction {
    let meta = tx.transaction.meta.as_ref();
    let message = &tx.transaction.transaction.message;
    let account_keys = message.account_keys();

    let fee = meta
        .as_ref()
        .and_then(|m| m.fee)
        .unwrap_or(0);

    let status = meta
        .as_ref()
        .and_then(|m| m.status.as_ref())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let signer = account_keys
        .first()
        .map(|k| k.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let slot = tx.slot;

    let instructions = decode_instructions(message, meta, account_keys, idl_map);

    let logs = meta
        .as_ref()
        .and_then(|m| m.log_messages.as_ref())
        .cloned()
        .unwrap_or_default();

    DecodedTransaction {
        signature: tx.transaction.transaction.signatures[0].to_string(),
        slot,
        fee,
        status,
        signer,
        instructions,
        logs,
    }
}

fn decode_instructions(
    message: &UiMessage,
    meta: Option<&UiTransactionStatusMeta>,
    account_keys: &[solana_sdk::pubkey::Pubkey],
    idl_map: &std::collections::HashMap<Pubkey, Idl>,
) -> Vec<DecodedInstruction> {
    let ui_instructions: Vec<&UiInstruction> = message
        .instructions()
        .iter()
        .collect();

    ui_instructions
        .iter()
        .enumerate()
        .map(|(idx, ui_ix)| {
            decode_single_instruction(idx, ui_ix, meta, account_keys, idl_map)
        })
        .collect()
}

fn decode_single_instruction(
    index: usize,
    ui_ix: &UiInstruction,
    meta: Option<&UiTransactionStatusMeta>,
    account_keys: &[solana_sdk::pubkey::Pubkey],
    idl_map: &std::collections::HashMap<Pubkey, Idl>,
) -> DecodedInstruction {
    match ui_ix {
        UiInstruction::Parsed(parsed) => {
            decode_parsed_instruction(index, parsed)
        }
        UiInstruction::PartiallyDecoded(partial) => {
            decode_partial_instruction(index, partial, meta, account_keys, idl_map)
        }
        UiInstruction::Compiled(compiled) => {
            decode_compiled_instruction(index, compiled, account_keys, idl_map)
        }
    }
}

fn decode_parsed_instruction(
    index: usize,
    parsed: &UiParsedInstruction,
) -> DecodedInstruction {
    let program = program_db::program_name(&parsed.program_id.parse().unwrap_or_else(|_| Pubkey::default()));
    let mut info = Vec::new();

    // Flatten parsed info into key-value pairs for display
    let parsed_value = &parsed.parsed;
    let kind = parsed_value
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    if let Some(info_obj) = parsed_value.get("info") {
        if let Some(obj) = info_obj.as_object() {
            for (key, val) in obj {
                let display_val = match val {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => val.to_string(),
                };
                info.push((key.clone(), display_val));
            }
        }
    }

    DecodedInstruction {
        index,
        program,
        instruction_name: kind.clone(),
        accounts: vec![],
        data: DecodedData::Parsed { kind, info },
    }
}

fn decode_partial_instruction(
    index: usize,
    partial: &UiPartiallyDecodedInstruction,
    meta: Option<&UiTransactionStatusMeta>,
    account_keys: &[solana_sdk::pubkey::Pubkey],
    idl_map: &std::collections::HashMap<Pubkey, Idl>,
) -> DecodedInstruction {
    let program_id = partial
        .program_id
        .parse::<Pubkey>()
        .unwrap_or(Pubkey::default());
    let program = program_db::program_name(&program_id);

    let data = &partial.data;
    let raw_bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .unwrap_or_default();

    let (discriminator, instruction_name) = if raw_bytes.len() >= 8 {
        let disc = &raw_bytes[..8];
        let disc_hex = hex_str(disc);

        // Try to match against an IDL
        if let Some(idl) = idl_map.get(&program_id) {
            if let Some(ix) = idl.find_instruction(&raw_bytes) {
                (Some(disc_hex), ix.name.clone())
            } else {
                (Some(disc_hex), "Unknown".to_string())
            }
        } else {
            (Some(disc_hex), "Unknown".to_string())
        }
    } else {
        (None, "Unknown".to_string())
    };

    // Try IDL decoding of args
    let decoded_data = if let Some(idl) = idl_map.get(&program_id) {
        if let Some(ix) = idl.find_instruction(&raw_bytes) {
            let args = decode_idl_args(ix, &raw_bytes[8..]);
            DecodedData::IdlDecoded {
                instruction_name: ix.name.clone(),
                args,
            }
        } else {
            DecodedData::Raw {
                base64: data.clone(),
                hex: hex_str(&raw_bytes),
                discriminator,
            }
        }
    } else {
        DecodedData::Raw {
            base64: data.clone(),
            hex: hex_str(&raw_bytes),
            discriminator,
        }
    };

    // Build account list with labels
    let accounts: Vec<DecodedAccount> = partial
        .accounts
        .iter()
        .enumerate()
        .map(|(i, acc_str)| {
            let pk = acc_str.parse::<Pubkey>().unwrap_or(Pubkey::default());
            let label = account_label(i, &pk, account_keys, meta);
            let is_signer = meta
                .and_then(|m| m.pre_token_balances.as_ref())
                .map(|_| false)
                .unwrap_or(false);
            DecodedAccount {
                pubkey: pk.to_string(),
                label,
                is_signer,
                is_writable: meta
                    .and_then(|m| m.post_token_balances.as_ref())
                    .map(|_| false)
                    .unwrap_or(false),
            }
        })
        .collect();

    DecodedInstruction {
        index,
        program,
        instruction_name,
        accounts,
        data: decoded_data,
    }
}

fn decode_compiled_instruction(
    index: usize,
    compiled: &UiCompiledInstruction,
    account_keys: &[solana_sdk::pubkey::Pubkey],
    idl_map: &std::collections::HashMap<Pubkey, Idl>,
) -> DecodedInstruction {
    let program_id = account_keys
        .get(compiled.program_id_index as usize)
        .copied()
        .unwrap_or(Pubkey::default());
    let program = program_db::program_name(&program_id);

    let raw_bytes = base64::engine::general_purpose::STANDARD
        .decode(&compiled.data)
        .unwrap_or_default();

    let discriminator = if raw_bytes.len() >= 8 {
        Some(hex_str(&raw_bytes[..8]))
    } else {
        None
    };

    let (instruction_name, decoded_data) = if let Some(idl) = idl_map.get(&program_id) {
        if let Some(ix) = idl.find_instruction(&raw_bytes) {
            let args = decode_idl_args(ix, &raw_bytes[8..]);
            (ix.name.clone(), DecodedData::IdlDecoded {
                instruction_name: ix.name.clone(),
                args,
            })
        } else {
            ("Unknown".to_string(), DecodedData::Raw {
                base64: compiled.data.clone(),
                hex: hex_str(&raw_bytes),
                discriminator,
            })
        }
    } else {
        ("Unknown".to_string(), DecodedData::Raw {
            base64: compiled.data.clone(),
            hex: hex_str(&raw_bytes),
            discriminator,
        })
    };

    let accounts: Vec<DecodedAccount> = compiled
        .account_indexes
        .iter()
        .enumerate()
        .map(|(i, &acc_idx)| {
            let pk = account_keys
                .get(acc_idx as usize)
                .copied()
                .unwrap_or(Pubkey::default());
            let label = account_label(i, &pk, account_keys, None);
            DecodedAccount {
                pubkey: pk.to_string(),
                label,
                is_signer: false,
                is_writable: false,
            }
        })
        .collect();

    DecodedInstruction {
        index,
        program,
        instruction_name,
        accounts,
        data: decoded_data,
    }
}

/// Best-effort label for an account based on its position and role.
fn account_label(
    index: usize,
    pk: &Pubkey,
    all_keys: &[Pubkey],
    _meta: Option<&UiTransactionStatusMeta>,
) -> String {
    // Position-based heuristics
    match index {
        0 => return format!("[signer] {}", program_db::program_name(pk)),
        1 if all_keys.len() > 2 => {
            return format!("[writable] {}", program_db::program_name(pk));
        }
        _ => {}
    }

    // Check if this is a known program
    let map = program_db::known_programs();
    if map.contains_key(pk) {
        return program_db::program_name(pk);
    }

    // Default: short address
    pk.to_string()
}

/// Convert bytes to a hex string like "a1 b2 c3 d4 e5 f6 78 90".
fn hex_str(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Attempt to decode instruction args using IDL type information.
///
/// ponytail: simplified Borsh-like deserialization — handles common Anchor
/// types (u8-u64, i8-i64, bool, Pubkey, String). Full Borsh would need a
/// recursive deserializer; this covers 90% of real-world instructions.
fn decode_idl_args(ix: &idl::IdlInstruction, data: &[u8]) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut offset = 0usize;

    for arg in &ix.args {
        if offset >= data.len() {
            results.push((arg.name.clone(), "<truncated>".into()));
            break;
        }

        let (val, advanced) = decode_idl_value(&arg.field_type, data, offset);
        results.push((arg.name.clone(), val));
        offset = advanced;
    }

    results
}

fn decode_idl_value(ty: &idl::IdlType, data: &[u8], offset: usize) -> (String, usize) {
    use idl::IdlType;

    match ty {
        IdlType::Simple(name) => match name.as_str() {
            "u8" => {
                if offset < data.len() {
                    (data[offset].to_string(), offset + 1)
                } else {
                    ("?".into(), offset)
                }
            }
            "u16" => read_u16(data, offset),
            "u32" => read_u32(data, offset),
            "u64" => read_u64(data, offset),
            "u128" => read_u128(data, offset),
            "i8" => {
                if offset < data.len() {
                    ((data[offset] as i8).to_string(), offset + 1)
                } else {
                    ("?".into(), offset)
                }
            }
            "i16" => read_i16(data, offset),
            "i32" => read_i32(data, offset),
            "i64" => read_i64(data, offset),
            "bool" => {
                if offset < data.len() {
                    (if data[offset] != 0 { "true" } else { "false" }.into(), offset + 1)
                } else {
                    ("?".into(), offset)
                }
            }
            "pubkey" | "publicKey" => {
                if offset + 32 <= data.len() {
                    let pk = Pubkey::new_from_array(
                        data[offset..offset + 32].try_into().unwrap(),
                    );
                    (pk.to_string(), offset + 32)
                } else {
                    ("<truncated>".into(), data.len())
                }
            }
            "string" => {
                // Anchor strings: 4-byte length prefix + UTF-8 bytes
                if offset + 4 <= data.len() {
                    let len = u32::from_le_bytes(
                        data[offset..offset + 4].try_into().unwrap(),
                    ) as usize;
                    let start = offset + 4;
                    if start + len <= data.len() {
                        let s = String::from_utf8_lossy(&data[start..start + len]);
                        (s.to_string(), start + len)
                    } else {
                        ("<truncated>".into(), data.len())
                    }
                } else {
                    ("?".into(), offset)
                }
            }
            _ => {
                // Unknown type — show remaining bytes as hex
                let remaining = &data[offset..];
                (format!("<{} bytes: {}>", remaining.len(), hex_str(remaining)), data.len())
            }
        },

        IdlType::Vec(inner) => {
            // Anchor vec: 4-byte length + elements
            if offset + 4 <= data.len() {
                let _len = u32::from_le_bytes(
                    data[offset..offset + 4].try_into().unwrap(),
                ) as usize;
                // ponytail: skip vec contents for display — show count only
                (format!("[{} items]", _len), offset + 4)
            } else {
                ("?".into(), offset)
            }
        }

        IdlType::Option(inner) => {
            // Anchor option: 1 byte tag (0 = None, 1 = Some) + value
            if offset < data.len() {
                if data[offset] == 0 {
                    ("None".into(), offset + 1)
                } else {
                    let (val, adv) = decode_idl_value(inner, data, offset + 1);
                    (format!("Some({})", val), adv)
                }
            } else {
                ("?".into(), offset)
            }
        }

        IdlType::Defined { defined } => {
            // User-defined type — dump remaining bytes as hex
            let remaining = data.len().saturating_sub(offset);
            (format!("<{}: {} bytes>", defined, remaining), data.len())
        }

        IdlType::Array { array: _, size } => {
            let remaining = data.len().saturating_sub(offset);
            let bytes_to_skip = remaining.min(*size);
            (format!("[{} bytes]", bytes_to_skip), offset + bytes_to_skip)
        }
    }
}

// --- Numeric readers (little-endian, Borsh encoding) ---

fn read_u16(data: &[u8], offset: usize) -> (String, usize) {
    if offset + 2 <= data.len() {
        let val = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
        (val.to_string(), offset + 2)
    } else {
        ("?".into(), offset)
    }
}

fn read_u32(data: &[u8], offset: usize) -> (String, usize) {
    if offset + 4 <= data.len() {
        let val = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        (val.to_string(), offset + 4)
    } else {
        ("?".into(), offset)
    }
}

fn read_u64(data: &[u8], offset: usize) -> (String, usize) {
    if offset + 8 <= data.len() {
        let val = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        (val.to_string(), offset + 8)
    } else {
        ("?".into(), offset)
    }
}

fn read_u128(data: &[u8], offset: usize) -> (String, usize) {
    if offset + 16 <= data.len() {
        let val = u128::from_le_bytes(data[offset..offset + 16].try_into().unwrap());
        (val.to_string(), offset + 16)
    } else {
        ("?".into(), offset)
    }
}

fn read_i16(data: &[u8], offset: usize) -> (String, usize) {
    if offset + 2 <= data.len() {
        let val = i16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
        (val.to_string(), offset + 2)
    } else {
        ("?".into(), offset)
    }
}

fn read_i32(data: &[u8], offset: usize) -> (String, usize) {
    if offset + 4 <= data.len() {
        let val = i32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        (val.to_string(), offset + 4)
    } else {
        ("?".into(), offset)
    }
}

fn read_i64(data: &[u8], offset: usize) -> (String, usize) {
    if offset + 8 <= data.len() {
        let val = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        (val.to_string(), offset + 8)
    } else {
        ("?".into(), offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_str() {
        assert_eq!(hex_str(&[0xde, 0xad, 0xbe, 0xef]), "de ad be ef");
    }

    #[test]
    fn test_read_u64_basic() {
        let data = vec![0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let (val, offset) = read_u64(&data, 0);
        assert_eq!(val, "100");
        assert_eq!(offset, 8);
    }
}
