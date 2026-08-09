//! `.thingd` schema validation and stable hash reporting.

use std::path::Path;

use sha2::Digest;

use crate::core::{AppError, ErrorKind};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchemaReport {
    pub hash: String,
    pub source_path: Option<String>,
    pub source: String,
}

pub fn validate_source(source: &str, source_path: Option<&Path>) -> Result<SchemaReport, AppError> {
    if source.trim().is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "`.thingd` schema is empty",
        ));
    }
    let normalized = source.replace("\r\n", "\n");
    if !normalized
        .lines()
        .any(|line| line.trim_start().starts_with("version "))
    {
        return Err(AppError::new(
            ErrorKind::Validation,
            "`.thingd` schema must declare a version",
        ));
    }
    let digest = sha2::Sha256::digest(normalized.as_bytes());
    Ok(SchemaReport {
        hash: format!("sha256:{}", hex::encode(digest)),
        source_path: source_path.map(|path| path.display().to_string()),
        source: normalized,
    })
}

pub fn validate_file(path: impl AsRef<Path>) -> Result<SchemaReport, AppError> {
    let path = path.as_ref();
    let source = std::fs::read_to_string(path).map_err(|error| {
        AppError::new(
            ErrorKind::Validation,
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    validate_source(&source, Some(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_line_endings_for_stable_hashes() {
        let unix = validate_source("version 1\n", None).unwrap();
        let windows = validate_source("version 1\r\n", None).unwrap();
        assert_eq!(unix.hash, windows.hash);
    }

    #[test]
    fn rejects_schema_without_version() {
        let error = validate_source("collection titles {}", None).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
    }
}
