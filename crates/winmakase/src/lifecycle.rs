//! Versioned lifecycle identity and request types.
//!
//! F0 freezes the data boundary only. L0/L1 own the mutex, lease, request
//! claiming and restoration state machine that use these types.

use serde::{Deserialize, Serialize};

use crate::actions::require_schema;

pub const LIFECYCLE_SCHEMA: u32 = 1;
pub const REQUEST_SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OwnerGeneration(pub String);

impl OwnerGeneration {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err("owner generation must not be empty".into());
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WmGeneration {
    pub process_id: u32,
    pub created_at_100ns: u64,
    pub config_revision: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    Native,
    Starting,
    Ready,
    Degraded,
    Restoring,
    Stopping,
    Stopped,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TakeoverPermission {
    #[default]
    Unknown,
    Revoked,
    Allowed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verb", rename_all = "snake_case", deny_unknown_fields)]
pub enum LifecycleRequest {
    Stop,
    Reload {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_config_revision: Option<String>,
    },
    Status,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestEnvelope {
    pub schema: u32,
    pub request_id: String,
    pub expected_owner_generation: OwnerGeneration,
    pub created_at: String,
    pub deadline: String,
    pub request: LifecycleRequest,
}

impl RequestEnvelope {
    pub fn validate(&self) -> Result<(), String> {
        require_schema("lifecycle request", self.schema, REQUEST_SCHEMA)?;
        for (name, value) in [
            ("request_id", self.request_id.as_str()),
            ("created_at", self.created_at.as_str()),
            ("deadline", self.deadline.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("lifecycle request {name} must not be empty"));
            }
        }
        if self.expected_owner_generation.0.trim().is_empty() {
            return Err("lifecycle request owner generation must not be empty".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_shape_is_finite_and_round_trips() {
        let request = RequestEnvelope {
            schema: REQUEST_SCHEMA,
            request_id: "req-22".into(),
            expected_owner_generation: OwnerGeneration::new("owner-3").unwrap(),
            created_at: "2026-09-16T12:00:00Z".into(),
            deadline: "2026-09-16T12:00:15Z".into(),
            request: LifecycleRequest::Reload {
                expected_config_revision: Some("config-8".into()),
            },
        };
        request.validate().unwrap();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"verb\":\"reload\""));
        assert_eq!(
            serde_json::from_str::<RequestEnvelope>(&json).unwrap(),
            request
        );
    }

    #[test]
    fn request_rejects_an_unknown_schema() {
        let request = RequestEnvelope {
            schema: 99,
            request_id: "req".into(),
            expected_owner_generation: OwnerGeneration::new("owner").unwrap(),
            created_at: "now".into(),
            deadline: "later".into(),
            request: LifecycleRequest::Stop,
        };
        assert!(request.validate().unwrap_err().contains("unsupported"));
    }
}
