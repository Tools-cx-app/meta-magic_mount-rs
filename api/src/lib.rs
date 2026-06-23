// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomMount {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApiConfig {
    pub mountsource: String,
    pub umount: bool,
    pub partitions: Vec<String>,
    pub ignore_list: Vec<String>,
    pub custom_mounts: Vec<CustomMount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub is_mounted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub kernel: Option<String>,
    pub selinux: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub version: String,
    pub device: DeviceInfo,
    pub system: SystemInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpenLinkRequest {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApiError {
    InvalidRequest,
    InvalidConfig,
    Unauthorized,
    NotFound,
    Conflict,
    Internal,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetail {
    pub code: ApiError,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectionInfo {
    pub port: u16,
    pub token: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_uses_camel_case_fields() {
        let value = serde_json::to_value(ApiConfig {
            mountsource: "KSU".into(),
            umount: true,
            partitions: vec!["vendor".into()],
            ignore_list: vec!["/ignored".into()],
            custom_mounts: vec![CustomMount {
                source: "/source".into(),
                target: "/target".into(),
            }],
        })
        .unwrap();

        assert_eq!(value["ignoreList"][0], "/ignored");
        assert_eq!(value["customMounts"][0]["source"], "/source");
        assert!(value.get("ignore_list").is_none());
    }

    #[test]
    fn module_and_status_use_camel_case_fields() {
        let module = serde_json::to_value(Module {
            id: "example".into(),
            name: "Example".into(),
            version: "1".into(),
            author: "Author".into(),
            description: "Description".into(),
            is_mounted: true,
        })
        .unwrap();
        assert_eq!(module["isMounted"], true);

        let status = serde_json::to_value(Status {
            version: "4.0.6".into(),
            device: DeviceInfo { model: None },
            system: SystemInfo {
                kernel: None,
                selinux: None,
            },
        })
        .unwrap();
        assert_eq!(status["version"], "4.0.6");
    }

    #[test]
    fn api_error_serializes_as_camel_case() {
        assert_eq!(
            serde_json::to_string(&ApiError::InvalidRequest).unwrap(),
            "\"invalidRequest\""
        );
    }

    #[test]
    fn api_error_rejects_snake_case() {
        assert!(serde_json::from_str::<ApiError>("\"invalid_request\"").is_err());
    }
}
