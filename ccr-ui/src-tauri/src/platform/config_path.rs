//! Config path normalization and validation helpers.
//!
//! We only allow relative paths *within* a platform config directory (e.g. `~/.claude/`).
//! This prevents path traversal such as `../.ssh/id_rsa`.

use super::EnvError;

const DEFAULT_CONFIG_FILE_NAME: &str = "settings.json";
const MAX_CONFIG_RELATIVE_PATH_LEN: usize = 1024;

fn windows_drive_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    if bytes.len() < 2 {
        return false;
    }
    let first = bytes[0] as char;
    bytes[1] == b':' && first.is_ascii_alphabetic()
}

/// Normalizes a user-provided config relative path and rejects unsafe inputs.
///
/// Rules:
/// - Empty input defaults to `settings.json`.
/// - Backslashes are normalized to `/`.
/// - Reject absolute paths (POSIX `/...`, UNC `//...`, Windows `C:\...` / `C:/...`).
/// - Reject any parent traversal segment (`..`).
/// - Reject control characters and overly long strings.
pub fn normalize_config_relative_path(path: &str) -> Result<String, EnvError> {
    let trimmed = path.trim();
    let candidate = if trimmed.is_empty() {
        DEFAULT_CONFIG_FILE_NAME
    } else {
        trimmed
    };

    if candidate.len() > MAX_CONFIG_RELATIVE_PATH_LEN {
        return Err(EnvError::Other(format!(
            "Invalid config path: exceeds {MAX_CONFIG_RELATIVE_PATH_LEN} chars"
        )));
    }

    if candidate.chars().any(|c| c.is_control()) {
        return Err(EnvError::Other(
            "Invalid config path: contains control characters".to_string(),
        ));
    }

    // Normalize separators first so subsequent checks can be consistent.
    let candidate = candidate.replace('\\', "/");

    // Absolute / UNC.
    if candidate.starts_with('/') || candidate.starts_with("//") {
        return Err(EnvError::Other(
            "Invalid config path: absolute paths are not allowed".to_string(),
        ));
    }
    // Windows drive prefix (after separator normalization `C:\x` -> `C:/x`).
    if windows_drive_prefix(&candidate) {
        return Err(EnvError::Other(
            "Invalid config path: Windows drive paths are not allowed".to_string(),
        ));
    }

    // Normalize `.` and repeated `/`, and reject any `..` traversal.
    let mut parts = Vec::new();
    for segment in candidate.split('/') {
        let seg = segment.trim();
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            return Err(EnvError::Other(
                "Invalid config path: parent traversal is not allowed".to_string(),
            ));
        }
        parts.push(seg);
    }

    if parts.is_empty() {
        return Ok(DEFAULT_CONFIG_FILE_NAME.to_string());
    }

    Ok(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::normalize_config_relative_path;

    #[test]
    fn empty_defaults_to_settings_json() {
        assert_eq!(
            normalize_config_relative_path("").unwrap(),
            "settings.json"
        );
        assert_eq!(
            normalize_config_relative_path("   ").unwrap(),
            "settings.json"
        );
        assert_eq!(
            normalize_config_relative_path(".").unwrap(),
            "settings.json"
        );
        assert_eq!(
            normalize_config_relative_path("./").unwrap(),
            "settings.json"
        );
    }

    #[test]
    fn normalizes_separators_and_dot_segments() {
        assert_eq!(
            normalize_config_relative_path(r#"a\b\c.json"#).unwrap(),
            "a/b/c.json"
        );
        assert_eq!(
            normalize_config_relative_path("a/./b//c.json").unwrap(),
            "a/b/c.json"
        );
    }

    #[test]
    fn rejects_parent_traversal() {
        assert!(normalize_config_relative_path("../x").is_err());
        assert!(normalize_config_relative_path("a/../x").is_err());
        assert!(normalize_config_relative_path(r#"a\..\x"#).is_err());
    }

    #[test]
    fn rejects_absolute_or_unc_paths() {
        assert!(normalize_config_relative_path("/etc/passwd").is_err());
        assert!(normalize_config_relative_path("//server/share/file").is_err());
        assert!(normalize_config_relative_path(r#"C:\Windows\win.ini"#).is_err());
        assert!(normalize_config_relative_path("C:/Windows/win.ini").is_err());
    }

    #[test]
    fn rejects_control_characters() {
        assert!(normalize_config_relative_path("a\nb").is_err());
        assert!(normalize_config_relative_path("a\rb").is_err());
        assert!(normalize_config_relative_path("a\0b").is_err());
    }
}

