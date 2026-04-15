// 🧰 Codex 公共工具函数
//
// 消除 ccr-codex 内部的重复代码:
// - Base64URL 解码 (原 3 处重复)
// - 路径解析 (原 3 处重复)

use ccr_core::core::error::{CcrError, Result};
use std::path::{Path, PathBuf};

/// Codex 相关路径集合
///
/// 统一解析 CCR_DATA_DIR / CCR_ROOT / CCR_CODEX_DIR 环境变量，
/// 消除各 service 构造函数中的重复逻辑。
pub struct CodexPaths {
    /// CCR 平台数据目录 (~/.ccr/platforms/codex/)
    pub ccr_codex_dir: PathBuf,
    /// Codex CLI 配置目录 (~/.codex/)
    pub codex_dir: PathBuf,
}

impl CodexPaths {
    /// 从环境变量和默认路径解析 Codex 路径
    pub fn resolve() -> Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;

        let ccr_root = if let Ok(custom) = std::env::var("CCR_DATA_DIR") {
            PathBuf::from(custom)
        } else if let Ok(custom) = std::env::var("CCR_ROOT") {
            PathBuf::from(custom)
        } else {
            home.join(".ccr")
        };
        let ccr_codex_dir = ccr_root.join("platforms/codex");

        let codex_dir = if let Ok(custom) = std::env::var("CCR_CODEX_DIR") {
            PathBuf::from(custom)
        } else {
            home.join(".codex")
        };

        Ok(Self {
            ccr_codex_dir,
            codex_dir,
        })
    }

    /// 从显式路径构造（用于测试注入）
    pub fn from_dirs(ccr_codex_dir: PathBuf, codex_dir: PathBuf) -> Self {
        Self {
            ccr_codex_dir,
            codex_dir,
        }
    }
}

/// OpenCode 相关路径集合
///
/// 统一解析 CCR_DATA_DIR / CCR_ROOT / CCR_OPENCODE_DIR 环境变量，
/// 用于 OpenCode Auth 的注册表与真实 auth.json 路径定位。
pub struct OpenCodePaths {
    /// CCR 平台数据目录 (~/.ccr/platforms/opencode/)
    pub ccr_opencode_dir: PathBuf,
    /// OpenCode 数据目录（官方默认：`$HOME/.local/share/opencode/`）
    pub opencode_dir: PathBuf,
}

impl OpenCodePaths {
    /// 从环境变量和默认路径解析 OpenCode 路径
    pub fn resolve() -> Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;

        let ccr_root = if let Ok(custom) = std::env::var("CCR_DATA_DIR") {
            PathBuf::from(custom)
        } else if let Ok(custom) = std::env::var("CCR_ROOT") {
            PathBuf::from(custom)
        } else {
            home.join(".ccr")
        };
        let ccr_opencode_dir = ccr_root.join("platforms/opencode");

        let opencode_dir = if let Ok(custom) = std::env::var("CCR_OPENCODE_DIR") {
            PathBuf::from(custom)
        } else {
            Self::resolve_default_opencode_dir(&home)
        };

        Ok(Self {
            ccr_opencode_dir,
            opencode_dir,
        })
    }

    /// 官方 OpenCode storage 目录。
    ///
    /// 上游文档当前明确说明：
    /// - macOS / Linux: `~/.local/share/opencode/`
    /// - Windows: `%USERPROFILE%\\.local\\share\\opencode\\`
    ///
    /// 因此这里统一以 `$HOME/.local/share/opencode` 为默认值，
    /// 并仅将系统 data-local 路径作为兼容旧实现的回退。
    fn official_storage_dir(home: &Path) -> PathBuf {
        home.join(".local").join("share").join("opencode")
    }

    fn resolve_default_opencode_dir(home: &Path) -> PathBuf {
        Self::select_default_opencode_dir(
            Self::official_storage_dir(home),
            dirs::data_local_dir().map(|dir| dir.join("opencode")),
        )
    }

    fn select_default_opencode_dir(official: PathBuf, legacy: Option<PathBuf>) -> PathBuf {
        if official.exists() {
            return official;
        }

        if let Some(legacy) = legacy
            && legacy.exists()
        {
            return legacy;
        }

        official
    }

    /// 从显式路径构造（用于测试注入）
    pub fn from_dirs(ccr_opencode_dir: PathBuf, opencode_dir: PathBuf) -> Self {
        Self {
            ccr_opencode_dir,
            opencode_dir,
        }
    }
}

/// Base64 URL-safe 解码
///
/// 处理带/不带 padding 的 base64url 输入，自动补齐后解码。
/// 如果 URL_SAFE_NO_PAD 失败，回退到标准 base64 解码器。
pub fn decode_base64url(input: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    // 添加 padding
    let padded = match input.len() % 4 {
        2 => format!("{}==", input),
        3 => format!("{}=", input),
        _ => input.to_string(),
    };

    URL_SAFE_NO_PAD.decode(&padded).ok().or_else(|| {
        use base64::engine::general_purpose::STANDARD;
        STANDARD.decode(&padded).ok()
    })
}

/// 设置文件为仅当前用户可读写（凭据文件保护）
///
/// - Unix: chmod 0o600
/// - Windows: 通过 icacls 移除继承权限，仅保留当前用户完全控制
pub fn ensure_private_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        let _ = std::fs::set_permissions(path, perms);
    }

    #[cfg(windows)]
    {
        let Some(path_str) = path.to_str() else {
            return;
        };
        let username = std::env::var("USERNAME").unwrap_or_default();
        if username.is_empty() {
            return;
        }
        // 移除继承权限，仅授予当前用户完全控制
        let _ = std::process::Command::new("icacls")
            .arg(path_str)
            .args(["/inheritance:r"])
            .args(["/grant:r", &format!("{username}:(F)")])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_base64url_standard() {
        // "test" 的 base64url 编码
        let decoded = decode_base64url("dGVzdA").unwrap();
        assert_eq!(decoded, b"test");
    }

    #[test]
    fn test_decode_base64url_with_padding() {
        let decoded = decode_base64url("dGVzdA==").unwrap();
        assert_eq!(decoded, b"test");
    }

    #[test]
    fn test_decode_base64url_json_payload() {
        // {"email":"test@example.com","sub":"1234567890"}
        let encoded = "eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJzdWIiOiIxMjM0NTY3ODkwIn0";
        let decoded = decode_base64url(encoded).unwrap();
        let text = String::from_utf8(decoded).unwrap();
        assert!(text.contains("test@example.com"));
    }

    #[test]
    fn test_codex_paths_from_dirs() {
        let paths = CodexPaths::from_dirs(
            PathBuf::from("/tmp/ccr/platforms/codex"),
            PathBuf::from("/tmp/codex"),
        );
        assert_eq!(
            paths.ccr_codex_dir,
            PathBuf::from("/tmp/ccr/platforms/codex")
        );
        assert_eq!(paths.codex_dir, PathBuf::from("/tmp/codex"));
    }

    #[test]
    fn test_opencode_paths_from_dirs() {
        let paths = OpenCodePaths::from_dirs(
            PathBuf::from("/tmp/ccr/platforms/opencode"),
            PathBuf::from("/tmp/opencode"),
        );
        assert_eq!(
            paths.ccr_opencode_dir,
            PathBuf::from("/tmp/ccr/platforms/opencode")
        );
        assert_eq!(paths.opencode_dir, PathBuf::from("/tmp/opencode"));
    }

    #[test]
    fn test_opencode_official_storage_dir() {
        let dir = OpenCodePaths::official_storage_dir(Path::new("/home/test"));
        assert_eq!(dir, PathBuf::from("/home/test/.local/share/opencode"));
    }

    #[test]
    fn test_select_default_opencode_dir_prefers_official_path() {
        let temp = tempfile::tempdir().unwrap();
        let official = temp.path().join("home/.local/share/opencode");
        let legacy = temp.path().join("legacy/opencode");
        std::fs::create_dir_all(&official).unwrap();
        std::fs::create_dir_all(&legacy).unwrap();

        let selected = OpenCodePaths::select_default_opencode_dir(official.clone(), Some(legacy));
        assert_eq!(selected, official);
    }

    #[test]
    fn test_select_default_opencode_dir_falls_back_to_legacy_path() {
        let temp = tempfile::tempdir().unwrap();
        let official = temp.path().join("home/.local/share/opencode");
        let legacy = temp.path().join("legacy/opencode");
        std::fs::create_dir_all(&legacy).unwrap();

        let selected = OpenCodePaths::select_default_opencode_dir(official, Some(legacy.clone()));
        assert_eq!(selected, legacy);
    }
}
