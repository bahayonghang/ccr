//! 扫描 `~/.claude/plugins/config.json` 读取已启用的插件安装位置。
//!
//! 对应 skill-hub `server/scanner/discovery.ts::discoverPluginSkillDirs`。
//! 关键纪律：**只读** `repositories[].installLocation` 下已启用的插件，
//! 不扫 `marketplaces/` 目录避免把候选插件误报为已装（skill-hub 原版踩过的坑）。

use serde::Deserialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct PluginConfig {
    #[serde(default)]
    repositories: std::collections::BTreeMap<String, PluginRepo>,
}

#[derive(Debug, Deserialize)]
struct PluginRepo {
    #[serde(rename = "installLocation")]
    install_location: Option<String>,
}

/// 读取 `~/.claude/plugins/config.json` 并返回所有已启用插件的 `installLocation` 路径。
/// 配置缺失 / 无效 JSON / 无 repositories 都返回空 vec，不抛错。
pub fn enabled_plugin_install_locations(home: &Path) -> Vec<PathBuf> {
    let config_path = home.join(".claude").join("plugins").join("config.json");
    let raw = match fs::read_to_string(&config_path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let Ok(cfg) = serde_json::from_str::<PluginConfig>(&raw) else {
        return Vec::new();
    };
    cfg.repositories
        .into_values()
        .filter_map(|r| r.install_location)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// 在给定插件目录下递归查找 `skills/` 子目录（最多 4 层）。
pub fn find_plugin_skills_dirs(install_location: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk(install_location, 0, 4, &mut out)?;
    Ok(out)
}

fn walk(dir: &Path, depth: usize, max_depth: usize, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if depth > max_depth {
        return Ok(());
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "node_modules" || name_str.starts_with(".git") {
            continue;
        }
        let sub = entry.path();
        if name_str == "skills" {
            out.push(sub);
            continue;
        }
        walk(&sub, depth + 1, max_depth, out)?;
    }
    Ok(())
}
