//! Durable mutation journal data contracts.
//!
//! This module intentionally performs no writes or locking. J0 owns the
//! mutation owner and recovery protocol; F0 supplies one schema so themes,
//! configuration and installation cannot invent incompatible journals.

use serde::{Deserialize, Serialize};

use crate::actions::require_schema;

pub const JOURNAL_SCHEMA: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationOperation {
    Configuration,
    Theme,
    TrayPreferences,
    Update,
    Off,
    Uninstall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Intent,
    Applying,
    Verifying,
    Committed,
    Restoring,
    Restored,
    Conflict,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum JournalTarget {
    OwnedFile { id: String },
    SharedFileKey { file_id: String, key: String },
    RegistryValue { key_id: String, value_name: String },
    Task { id: String },
    Shortcut { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum JournalValue {
    Absent,
    String(String),
    ExpandString(String),
    Dword(u32),
    Qword(u64),
    MultiString(Vec<String>),
    BinaryHash(String),
    FileHash(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JournalStepStatus {
    Pending,
    Applied,
    Verified,
    Restored,
    Conflict,
    Failed,
    NotRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JournalHeader {
    pub schema: u32,
    pub install_id: String,
    pub mutation_id: String,
    pub base_generation: String,
    pub candidate_generation: String,
    pub operation: MutationOperation,
    pub owner_creation_identity: String,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JournalEntry {
    pub target: JournalTarget,
    pub prior: JournalValue,
    pub intended: JournalValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_owned: Option<JournalValue>,
    pub intent: JournalStepStatus,
    pub apply: JournalStepStatus,
    pub verify: JournalStepStatus,
    pub restore: JournalStepStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationJournal {
    pub header: JournalHeader,
    pub entries: Vec<JournalEntry>,
}

impl MutationJournal {
    pub fn validate(&self) -> Result<(), String> {
        require_schema("mutation journal", self.header.schema, JOURNAL_SCHEMA)?;
        for (name, value) in [
            ("install_id", self.header.install_id.as_str()),
            ("mutation_id", self.header.mutation_id.as_str()),
            ("base_generation", self.header.base_generation.as_str()),
            (
                "candidate_generation",
                self.header.candidate_generation.as_str(),
            ),
            (
                "owner_creation_identity",
                self.header.owner_creation_identity.as_str(),
            ),
        ] {
            if value.trim().is_empty() {
                return Err(format!("mutation journal {name} must not be empty"));
            }
        }
        if self.entries.is_empty() {
            return Err("mutation journal needs at least one entry".into());
        }
        Ok(())
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        let value: Self = serde_json::from_str(text)
            .map_err(|error| format!("mutation journal parse error: {error}"))?;
        value.validate()?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn journal() -> MutationJournal {
        MutationJournal {
            header: JournalHeader {
                schema: JOURNAL_SCHEMA,
                install_id: "install-a".into(),
                mutation_id: "mutation-9".into(),
                base_generation: "generation-3".into(),
                candidate_generation: "generation-4".into(),
                operation: MutationOperation::Theme,
                owner_creation_identity: "pid:42@123456".into(),
                status: TransactionStatus::Intent,
            },
            entries: vec![JournalEntry {
                target: JournalTarget::RegistryValue {
                    key_id: "windows.personalize".into(),
                    value_name: "AppsUseLightTheme".into(),
                },
                prior: JournalValue::Absent,
                intended: JournalValue::Dword(1),
                last_owned: None,
                intent: JournalStepStatus::Pending,
                apply: JournalStepStatus::Pending,
                verify: JournalStepStatus::Pending,
                restore: JournalStepStatus::NotRequired,
                error: None,
            }],
        }
    }

    #[test]
    fn journal_round_trip_preserves_absent_and_native_values() {
        let journal = journal();
        journal.validate().unwrap();
        let json = serde_json::to_string_pretty(&journal).unwrap();
        assert!(json.contains("\"type\": \"absent\""));
        assert!(json.contains("\"type\": \"dword\""));
        assert_eq!(MutationJournal::from_json(&json).unwrap(), journal);
    }

    #[test]
    fn journal_rejects_unknown_versions() {
        let mut journal = journal();
        journal.header.schema = 2;
        assert!(journal.validate().unwrap_err().contains("unsupported"));
    }
}
