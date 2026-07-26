use ccr_core::core::error::{CcrError, Result};
use reqwest_dav::re_exports::url::Url;
use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};

#[cfg(test)]
const DEFAULT_MAX_COMPONENT_BYTES: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RemoteEntryName(String);

impl RemoteEntryName {
    #[cfg(test)]
    pub fn from_href(href: &str) -> Result<Self> {
        Self::from_href_with_limit(href, DEFAULT_MAX_COMPONENT_BYTES)
    }

    pub fn from_href_with_limit(href: &str, max_component_bytes: usize) -> Result<Self> {
        let path = href_path(href)?;
        let without_leading = path.strip_prefix('/').unwrap_or(&path);
        let trimmed = without_leading.strip_suffix('/').unwrap_or(without_leading);
        if trimmed.split('/').any(str::is_empty) {
            return Err(path_error("empty", "远端 href 包含空路径片段"));
        }
        let encoded = trimmed.rsplit('/').next().unwrap_or_default();
        let decoded = percent_decode_once(encoded)?;
        Self::parse_with_limit(&decoded, max_component_bytes)
    }

    pub fn parse_with_limit(name: &str, max_component_bytes: usize) -> Result<Self> {
        if name.is_empty() {
            return Err(path_error("empty", "远端条目名称为空"));
        }
        if name.len() > max_component_bytes {
            return Err(path_error("component_too_long", "远端条目名称超过长度上限"));
        }
        if name == "." || name == ".." {
            return Err(path_error("dot_component", "远端条目不能是当前或父目录"));
        }
        if name.contains(['/', '\\', ':']) || name.chars().any(|ch| ch == '\0' || ch.is_control()) {
            return Err(path_error(
                "invalid_component",
                "远端条目不是安全的单一路径组件",
            ));
        }

        let mut components = Path::new(name).components();
        match (components.next(), components.next()) {
            (Some(Component::Normal(component)), None) if component == name => {
                Ok(Self(name.to_string()))
            }
            _ => Err(path_error(
                "invalid_component",
                "远端条目必须恰好包含一个普通路径组件",
            )),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn join_contained(&self, local_dir: &Path) -> Result<PathBuf> {
        let destination = local_dir.join(&self.0);
        if !destination.starts_with(local_dir) {
            return Err(path_error("containment", "远端条目超出本地同步目录"));
        }

        if local_dir.exists() {
            let canonical_root = std::fs::canonicalize(local_dir).map_err(|error| {
                path_error("canonical_root", &format!("无法校验本地同步目录: {error}"))
            })?;
            let parent = destination.parent().unwrap_or(local_dir);
            if parent.exists() {
                let canonical_parent = std::fs::canonicalize(parent).map_err(|error| {
                    path_error("canonical_parent", &format!("无法校验目标父目录: {error}"))
                })?;
                if !canonical_parent.starts_with(&canonical_root) {
                    return Err(path_error("containment", "目标父目录超出本地同步目录"));
                }
            }
        }

        Ok(destination)
    }
}

pub fn normalize_href_identity(href: &str) -> Result<String> {
    let path = href_path(href)?;
    let without_leading = path.strip_prefix('/').unwrap_or(&path);
    let trimmed = without_leading.strip_suffix('/').unwrap_or(without_leading);
    if trimmed.is_empty() {
        return Ok("/".to_string());
    }

    let mut normalized = String::new();
    for segment in trimmed.split('/') {
        if segment.is_empty() {
            return Err(path_error("empty", "远端 href 包含空路径片段"));
        }
        let decoded = percent_decode_once(segment)?;
        if decoded == "." || decoded == ".." || decoded.contains(['/', '\\']) {
            return Err(path_error("invalid_href", "远端 href 含不安全路径片段"));
        }
        normalized.push('/');
        normalized.push_str(&decoded);
    }
    if normalized.is_empty() {
        normalized.push('/');
    }
    Ok(normalized)
}

fn href_path(href: &str) -> Result<Cow<'_, str>> {
    if href.contains("://") {
        let url =
            Url::parse(href).map_err(|_| path_error("invalid_href", "远端 href 不是有效 URL"))?;
        return Ok(Cow::Owned(url.path().to_string()));
    }

    let without_fragment = href.split('#').next().unwrap_or_default();
    let without_query = without_fragment.split('?').next().unwrap_or_default();
    Ok(Cow::Borrowed(without_query))
}

fn percent_decode_once(input: &str) -> Result<String> {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(path_error(
                    "invalid_percent_encoding",
                    "远端 href 百分号编码不完整",
                ));
            }
            let high = hex_value(bytes[index + 1]);
            let low = hex_value(bytes[index + 2]);
            let (Some(high), Some(low)) = (high, low) else {
                return Err(path_error(
                    "invalid_percent_encoding",
                    "远端 href 百分号编码无效",
                ));
            };
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded)
        .map_err(|_| path_error("invalid_utf8", "远端 href 解码后不是有效 UTF-8"))
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn path_error(code: &str, message: &str) -> CcrError {
    CcrError::SyncError(format!("sync_path_{code}: {message}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn accepts_exactly_one_decoded_normal_component() {
        assert_eq!(
            RemoteEntryName::from_href("/ccr/a%20b.toml")
                .unwrap()
                .as_str(),
            "a b.toml"
        );
        assert_eq!(
            RemoteEntryName::from_href("https://dav.example/ccr/config.toml?x=1")
                .unwrap()
                .as_str(),
            "config.toml"
        );
        assert_eq!(
            RemoteEntryName::from_href("/ccr/%252e%252e")
                .unwrap()
                .as_str(),
            "%2e%2e"
        );
    }

    #[test]
    fn rejects_hostile_href_corpus() {
        for href in [
            "/ccr/../",
            "/ccr/%2e%2e/",
            "/ccr/..%5cevil",
            "/ccr/C:%5cevil",
            "/ccr/%2f%2fserver%2fshare",
            "/ccr//",
            "/ccr///evil",
            "/",
            "/ccr/%00evil",
            "/ccr/%GG",
        ] {
            assert!(
                RemoteEntryName::from_href(href).is_err(),
                "href should be rejected: {href}"
            );
        }
    }

    #[test]
    fn enforces_component_length_and_containment() {
        assert!(RemoteEntryName::parse_with_limit("12345", 4).is_err());
        let temp = tempfile::tempdir().unwrap();
        let name = RemoteEntryName::parse_with_limit("config.toml", 255).unwrap();
        assert_eq!(
            name.join_contained(temp.path()).unwrap(),
            temp.path().join("config.toml")
        );
    }

    #[test]
    fn normalizes_equivalent_href_identities_for_cycle_detection() {
        assert_eq!(normalize_href_identity("/ccr/%61/").unwrap(), "/ccr/a");
        assert_eq!(
            normalize_href_identity("https://dav.example/ccr/a").unwrap(),
            "/ccr/a"
        );
        assert!(normalize_href_identity("/ccr/%2e%2e/a").is_err());
        assert!(normalize_href_identity("/ccr//a").is_err());
    }
}
