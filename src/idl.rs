//! Anchor IDL type definitions and instruction matching.
//!
//! An IDL (Interface Definition Language) file describes a Solana program's
//! instructions, accounts, and types. We use it to decode raw instruction data
//! into named fields with human-readable values.

use serde::{Deserialize, Serialize};

/// A complete Anchor IDL file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idl {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub metadata: Option<IdlMetadata>,
    #[serde(default)]
    pub instructions: Vec<IdlInstruction>,
    #[serde(default)]
    pub accounts: Vec<IdlAccount>,
    #[serde(default)]
    pub types: Vec<IdlTypeDef>,
    #[serde(default)]
    pub events: Vec<IdlEvent>,
    #[serde(default)]
    pub errors: Vec<IdlError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlMetadata {
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlInstruction {
    pub name: String,
    #[serde(default)]
    pub docs: Vec<String>,
    #[serde(default)]
    pub accounts: Vec<IdlAccountItem>,
    pub args: Vec<IdlField>,
    /// The 8-byte discriminator is derived from `sha256("global:<name>")[:8]`.
    /// We compute it from the name rather than storing it in the IDL.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdlAccountItem {
    /// A single account with constraints
    Account(IdlAccountDef),
    /// A nested group of accounts
    Group {
        name: String,
        accounts: Vec<IdlAccountItem>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlAccountDef {
    pub name: String,
    #[serde(default)]
    pub is_mut: bool,
    #[serde(default)]
    pub is_signer: bool,
    #[serde(default)]
    pub docs: Vec<String>,
    #[serde(default)]
    pub pda: Option<serde_json::Value>,
}

/// A type definition block in the IDL (structs, enums).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlTypeDef {
    pub name: String,
    #[serde(rename = "type")]
    pub type_kind: IdlTypeKind,
    #[serde(default)]
    pub fields: Vec<IdlField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: IdlType,
    #[serde(default)]
    pub docs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdlType {
    Simple(String),
    Vec(Box<IdlType>),
    Option(Box<IdlType>),
    Defined {
        defined: String,
    },
    Array {
        array: Box<IdlType>,
        size: usize,
    },
}

impl IdlType {
    pub fn name(&self) -> String {
        match self {
            IdlType::Simple(s) => s.clone(),
            IdlType::Vec(inner) => format!("Vec<{}>", inner.name()),
            IdlType::Option(inner) => format!("Option<{}>", inner.name()),
            IdlType::Defined { defined } => defined.clone(),
            IdlType::Array { array, size } => format!("[{}; {}]", array.name(), size),
        }
    }
}

/// Determines the displayed name for each type so we call u64 "u64"
/// rather than "unsigned 64-bit integer".
impl std::fmt::Display for IdlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdlType::Simple(s) => write!(f, "{}", s),
            IdlType::Vec(inner) => write!(f, "Vec<{}>", inner),
            IdlType::Option(inner) => write!(f, "Option<{}>", inner),
            IdlType::Defined { defined } => write!(f, "{}", defined),
            IdlType::Array { array, size } => write!(f, "[{}; {}]", array, size),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdlTypeKind {
    Struct,
    Enum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlAccount {
    pub name: String,
    #[serde(rename = "type")]
    pub type_kind: IdlTypeKind,
    #[serde(default)]
    pub fields: Vec<IdlField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlEvent {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<IdlField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlError {
    pub code: u32,
    pub name: String,
    #[serde(default)]
    pub msg: Option<String>,
}

// --- Discriminator computation ---

/// Compute the 8-byte discriminator for an Anchor instruction.
///
/// Anchor uses `sha256("global:<instruction_name>")[:8]` as the
/// instruction discriminator.
pub fn instruction_discriminator(name: &str) -> [u8; 8] {
    use sha2::{Digest, Sha256};
    let preimage = format!("global:{}", name);
    let hash = Sha256::digest(preimage.as_bytes());
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash[..8]);
    disc
}

// ponytail: sha2 is only used in this one function for discriminator
// computation. If adding sha2 feels heavy, we could precompute known
// discriminators and match by name. But sha2 is the correct approach
// for matching against IDLs from unknown programs.

impl Idl {
    /// Load an IDL from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Find the instruction whose discriminator matches the given data prefix.
    pub fn find_instruction(&self, data: &[u8]) -> Option<&IdlInstruction> {
        if data.len() < 8 {
            return None;
        }
        let disc = &data[..8];
        self.instructions.iter().find(|ix| {
            instruction_discriminator(&ix.name) == disc
        })
    }

    /// Find the instruction by name.
    pub fn get_instruction(&self, name: &str) -> Option<&IdlInstruction> {
        self.instructions.iter().find(|ix| ix.name == name)
    }

    /// Resolve a known program error code to its name.
    pub fn resolve_error(&self, error_code: u32) -> Option<&IdlError> {
        self.errors.iter().find(|e| e.code == error_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discriminator_for_create_vesting() {
        // Golden discriminator for create_vesting
        let disc = instruction_discriminator("create_vesting");
        assert_eq!(disc.len(), 8);
        // Discriminator should be deterministic
        let disc2 = instruction_discriminator("create_vesting");
        assert_eq!(disc, disc2);
        // Different names produce different discriminators
        let disc_claim = instruction_discriminator("claim_tokens");
        assert_ne!(disc, disc_claim);
    }

    #[test]
    fn test_find_instruction_by_discriminator() {
        let idl = Idl {
            name: "test".into(),
            version: "0.1.0".into(),
            metadata: None,
            instructions: vec![
                IdlInstruction {
                    name: "create".into(),
                    docs: vec![],
                    accounts: vec![],
                    args: vec![],
                },
                IdlInstruction {
                    name: "claim".into(),
                    docs: vec![],
                    accounts: vec![],
                    args: vec![],
                },
            ],
            accounts: vec![],
            types: vec![],
            events: vec![],
            errors: vec![],
        };

        let create_disc = instruction_discriminator("create");
        assert!(idl.find_instruction(&create_disc).is_some());
        assert_eq!(
            idl.find_instruction(&create_disc).unwrap().name,
            "create"
        );
    }

    #[test]
    fn test_short_data_no_panic() {
        let idl = Idl {
            name: "test".into(),
            version: "0.1.0".into(),
            metadata: None,
            instructions: vec![],
            accounts: vec![],
            types: vec![],
            events: vec![],
            errors: vec![],
        };
        // Data shorter than 8 bytes should return None, not panic
        assert!(idl.find_instruction(&[1, 2, 3]).is_none());
    }
}
