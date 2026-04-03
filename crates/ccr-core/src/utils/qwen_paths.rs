use std::ffi::OsString;
use std::path::{Path, PathBuf};

const QWEN_RUNTIME_DIR_ENV: &str = "QWEN_RUNTIME_DIR";

/// 解析 Qwen 运行时根目录。
pub fn resolve_qwen_runtime_base_dir(
    home_dir: Option<PathBuf>,
    env_value: Option<OsString>,
) -> Option<PathBuf> {
    env_value
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| home_dir.map(|home| home.join(".qwen")))
}

/// 获取当前环境下的 Qwen 运行时根目录。
pub fn qwen_runtime_base_dir() -> Option<PathBuf> {
    resolve_qwen_runtime_base_dir(dirs::home_dir(), std::env::var_os(QWEN_RUNTIME_DIR_ENV))
}

/// 获取 Qwen 项目目录。
pub fn qwen_projects_dir() -> Option<PathBuf> {
    qwen_runtime_base_dir().map(|base_dir| base_dir.join("projects"))
}

/// 判断路径是否为官方 Qwen 聊天记录文件。
pub fn is_qwen_chat_file(path: &Path) -> bool {
    if path.extension().is_none_or(|ext| ext != "jsonl") {
        return false;
    }

    let Some(chats_dir) = path.parent() else {
        return false;
    };
    if chats_dir.file_name().and_then(|name| name.to_str()) != Some("chats") {
        return false;
    }

    let Some(project_dir) = chats_dir.parent() else {
        return false;
    };
    let Some(projects_dir) = project_dir.parent() else {
        return false;
    };

    projects_dir.file_name().and_then(|name| name.to_str()) == Some("projects")
}

/// 从 Qwen 聊天记录路径提取已清洗的项目目录名。
pub fn qwen_project_dir_name_from_chat_path(path: &Path) -> Option<String> {
    if !is_qwen_chat_file(path) {
        return None;
    }

    path.parent()?
        .parent()?
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_qwen_runtime_base_dir_prefers_env_override() {
        let home_dir = Some(PathBuf::from("/home/demo"));
        let runtime_dir =
            resolve_qwen_runtime_base_dir(home_dir, Some(OsString::from("/tmp/custom-qwen")))
                .expect("runtime dir should resolve");

        assert_eq!(runtime_dir, PathBuf::from("/tmp/custom-qwen"));
    }

    #[test]
    fn resolve_qwen_runtime_base_dir_falls_back_to_home() {
        let runtime_dir = resolve_qwen_runtime_base_dir(Some(PathBuf::from("/home/demo")), None)
            .expect("runtime dir should resolve");

        assert_eq!(runtime_dir, PathBuf::from("/home/demo/.qwen"));
    }

    #[test]
    fn qwen_chat_file_detection_requires_projects_and_chats_dirs() {
        assert!(is_qwen_chat_file(Path::new(
            "/home/demo/.qwen/projects/workspace___repo/chats/session-1.jsonl"
        )));
        assert!(!is_qwen_chat_file(Path::new(
            "/home/demo/.qwen/sessions/session-1.jsonl"
        )));
        assert!(!is_qwen_chat_file(Path::new(
            "/home/demo/.qwen/projects/workspace___repo/session-1.jsonl"
        )));
    }

    #[test]
    fn extracts_qwen_project_dir_name_from_chat_path() {
        let project_dir = qwen_project_dir_name_from_chat_path(Path::new(
            "/home/demo/.qwen/projects/workspace___repo/chats/session-1.jsonl",
        ))
        .expect("project dir name should resolve");

        assert_eq!(project_dir, "workspace___repo");
    }
}
