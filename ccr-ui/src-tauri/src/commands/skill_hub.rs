//! SkillHub 市场命令 — 技能市场浏览、搜索、安装与多平台管理。

use std::{collections::HashMap, sync::Arc};

use serde_json::Value;
use tauri::State;
use tokio::{sync::Semaphore, task::JoinSet};

use crate::process::tokio_command;
use crate::state::AppState;

/// 6 大平台的 skills 目录配置：(id, display_name, relative_path)
const PLATFORM_SKILLS_DIRS: &[(&str, &str, &str)] = &[
    ("claude-code", "Claude Code", ".claude/skills"),
    ("codex", "Codex", ".codex/skills"),
    ("gemini", "Gemini CLI", ".gemini/skills"),
    ("qwen", "Qwen", ".qwen/skills"),
    ("qoder", "Qoder", ".qoder/skills"),
    ("droid", "Droid", ".gemini/antigravity/skills"),
];

// ── 内部辅助 ──

/// 扫描指定目录下的 skills（每个含 SKILL.md 的子目录为一个技能）。
/// 返回 JSON 数组，每项包含 name/description/skill_dir/platform/platform_name/category/tags 等。
fn scan_platform_skills(
    base: &std::path::Path,
    platform_id: &str,
    platform_name: &str,
) -> Vec<Value> {
    if !base.is_dir() {
        return Vec::new();
    }
    let entries = match std::fs::read_dir(base) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut skills = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_file = path.join("SKILL.md");
        if !skill_file.exists() {
            continue;
        }
        let name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        if name.is_empty() {
            continue;
        }

        let instruction = std::fs::read_to_string(&skill_file).unwrap_or_default();
        let (metadata, description) = ccr::models::skill::Skill::parse_with_fallback(&instruction);

        // 可选：读取 .skill-meta.json 元数据
        let meta_path = path.join(".skill-meta.json");
        let meta: serde_json::Value = if meta_path.exists() {
            std::fs::read_to_string(&meta_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            serde_json::Value::Null
        };

        let skill_dir = path.to_string_lossy().to_string();

        skills.push(serde_json::json!({
            "name": name,
            "description": description,
            "skill_dir": skill_dir,
            "platform": platform_id,
            "platform_name": platform_name,
            "category": metadata.category,
            "tags": metadata.tags,
            "version": metadata.version,
            "author": metadata.author,
            // .skill-meta.json 中的安装元数据
            "source": meta.get("source").and_then(|v| v.as_str()),
            "source_url": meta.get("source_url").and_then(|v| v.as_str()),
            "install_date": meta.get("install_date").and_then(|v| v.as_i64()),
            "commit_hash": meta.get("commit_hash").and_then(|v| v.as_str()),
        }));
    }
    skills
}

/// 统计目录下含 SKILL.md 的子目录数量。
fn count_skills_in_dir(dir: &std::path::Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").exists())
                .count()
        })
        .unwrap_or(0)
}

/// 查找平台配置。
fn find_platform(platform_id: &str) -> Option<(&'static str, &'static str, &'static str)> {
    PLATFORM_SKILLS_DIRS
        .iter()
        .find(|(id, _, _)| *id == platform_id)
        .copied()
}

/// 写入 .skill-meta.json 安装元数据。
fn write_skill_meta(
    skill_dir: &std::path::Path,
    source: &str,
    source_url: Option<&str>,
) -> Result<(), String> {
    let meta = serde_json::json!({
        "source": source,
        "source_url": source_url,
        "install_date": chrono::Utc::now().timestamp_millis(),
    });
    let meta_path = skill_dir.join(".skill-meta.json");
    std::fs::write(
        &meta_path,
        serde_json::to_string_pretty(&meta).unwrap_or_default(),
    )
    .map_err(|e| format!("写入 .skill-meta.json 失败: {e}"))
}

// ── Step 1: 平台摘要（重写） ──

#[tauri::command]
pub async fn skill_hub_agents() -> Result<Value, String> {
    let platforms = tokio::task::spawn_blocking(|| {
        let home = dirs::home_dir().ok_or_else(|| "无法获取主目录".to_string())?;
        let mut platform_list: Vec<Value> = Vec::new();

        for &(id, display_name, rel_path) in PLATFORM_SKILLS_DIRS {
            let dir = home.join(rel_path);
            let detected = dir.exists();
            let installed_count = if detected {
                count_skills_in_dir(&dir)
            } else {
                0
            };

            platform_list.push(serde_json::json!({
                "id": id,
                "display_name": display_name,
                "global_skills_dir": dir.to_string_lossy(),
                "detected": detected,
                "installed_count": installed_count,
            }));
        }

        Ok::<_, String>(platform_list)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(serde_json::json!({ "platforms": platforms }))
}

// ── Step 2: 平台 skills 列表（重写） ──

#[tauri::command]
pub async fn skill_hub_agent_skills(agent_name: String) -> Result<Value, String> {
    // agent_name 实际上是 platform id（如 "claude-code"）
    let platform_id = agent_name;

    let skills = tokio::task::spawn_blocking(move || {
        let (id, name, rel_path) =
            find_platform(&platform_id).ok_or_else(|| format!("未知平台: {platform_id}"))?;
        let home = dirs::home_dir().ok_or_else(|| "无法获取主目录".to_string())?;
        let dir = home.join(rel_path);

        Ok::<_, String>(scan_platform_skills(&dir, id, name))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(serde_json::json!({ "skills": skills }))
}

// ── Step 3: 统一查询（新增） ──

#[tauri::command]
pub async fn skill_hub_unified(platform: Option<String>) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        let home = dirs::home_dir().ok_or_else(|| "无法获取主目录".to_string())?;
        let mut all_skills: Vec<Value> = Vec::new();
        let mut platform_list: Vec<Value> = Vec::new();

        let targets: Vec<(&str, &str, &str)> = if let Some(ref pid) = platform {
            find_platform(pid).map(|p| vec![p]).unwrap_or_default()
        } else {
            PLATFORM_SKILLS_DIRS.to_vec()
        };

        for (id, display_name, rel_path) in &targets {
            let dir = home.join(rel_path);
            let detected = dir.exists();
            let skills = scan_platform_skills(&dir, id, display_name);
            let installed_count = skills.len();

            platform_list.push(serde_json::json!({
                "id": id,
                "display_name": display_name,
                "global_skills_dir": dir.to_string_lossy(),
                "detected": detected,
                "installed_count": installed_count,
            }));
            all_skills.extend(skills);
        }

        let total = all_skills.len();
        Ok::<_, String>(serde_json::json!({
            "skills": all_skills,
            "platforms": platform_list,
            "total": total,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

// ── Step 4a: 读取 Skill 内容（新增） ──

#[tauri::command]
pub async fn skill_hub_skill_content(skill_dir: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let dir = std::path::Path::new(&skill_dir);
        let skill_file = dir.join("SKILL.md");
        if !skill_file.exists() {
            return Err(format!("SKILL.md 不存在: {}", skill_file.display()));
        }

        let raw =
            std::fs::read_to_string(&skill_file).map_err(|e| format!("读取 SKILL.md 失败: {e}"))?;
        let (metadata, description) = ccr::models::skill::Skill::parse_with_fallback(&raw);

        // 提取 frontmatter 后的 body 内容
        let content = {
            let trimmed = raw.trim();
            if let Some(after_prefix) = trimmed.strip_prefix("---") {
                if let Some(end_idx) = after_prefix.find("---") {
                    after_prefix[end_idx + 3..].trim().to_string()
                } else {
                    raw.clone()
                }
            } else {
                raw.clone()
            }
        };

        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        Ok(serde_json::json!({
            "name": name,
            "description": description,
            "category": metadata.category,
            "tags": metadata.tags,
            "content": content,
            "raw": raw,
            "skill_dir": skill_dir,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

// ── Step 4b: 保存 Skill 内容（新增） ──

#[tauri::command]
pub async fn skill_hub_save_skill_content(
    skill_dir: String,
    content: String,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let dir = std::path::Path::new(&skill_dir);
        if !dir.exists() {
            return Err(format!("技能目录不存在: {skill_dir}"));
        }

        let skill_file = dir.join("SKILL.md");

        // 原子写入：先写临时文件再 rename
        let temp_dir = dir;
        let temp_file = tempfile::NamedTempFile::new_in(temp_dir)
            .map_err(|e| format!("创建临时文件失败: {e}"))?;
        std::fs::write(temp_file.path(), &content).map_err(|e| format!("写入临时文件失败: {e}"))?;
        // persist 会自动 rename（Windows 上如果目标存在则先删除）
        temp_file
            .persist(&skill_file)
            .map_err(|e| format!("原子重命名失败: {e}"))?;

        Ok(serde_json::json!({ "success": true }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

// ── Step 6a: 检查 npx 可用性（新增） ──

#[tauri::command]
pub async fn skill_hub_check_npx() -> Result<Value, String> {
    let result = tokio_command(if cfg!(windows) { "cmd" } else { "sh" })
        .args(if cfg!(windows) {
            vec!["/C", "npx", "--version"]
        } else {
            vec!["-c", "npx --version"]
        })
        .output()
        .await
        .map_err(|e| format!("执行 npx --version 失败: {e}"))?;

    if result.status.success() {
        let version = String::from_utf8_lossy(&result.stdout).trim().to_string();
        // 获取 npx 路径
        let path_result = tokio_command(if cfg!(windows) { "cmd" } else { "sh" })
            .args(if cfg!(windows) {
                vec!["/C", "where", "npx"]
            } else {
                vec!["-c", "which npx"]
            })
            .output()
            .await;

        let path = path_result.ok().filter(|r| r.status.success()).map(|r| {
            String::from_utf8_lossy(&r.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .to_string()
        });

        Ok(serde_json::json!({
            "available": true,
            "version": version,
            "path": path,
        }))
    } else {
        Ok(serde_json::json!({ "available": false }))
    }
}

// ── Step 6b: 文件夹选择对话框（新增） ──

#[tauri::command]
pub async fn skill_hub_browse_folder(app: tauri::AppHandle) -> Result<Value, String> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = tokio::sync::oneshot::channel::<Option<String>>();
    app.dialog().file().pick_folder(move |folder_path| {
        let path_str = folder_path.map(|p| p.to_string());
        let _ = tx.send(path_str);
    });

    let result: Option<String> = rx.await.map_err(|e| format!("Dialog error: {e}"))?;
    Ok(serde_json::json!({ "path": result }))
}

// ── Step 7a: GitHub 导入（新增） ──

#[tauri::command]
pub async fn skill_hub_import_github(
    state: State<'_, AppState>,
    url: String,
    agents: Vec<String>,
    force: Option<bool>,
) -> Result<Value, String> {
    let force = force.unwrap_or(false);
    let client = state.http_client.clone();

    // 将 GitHub URL 转换为 raw URL
    let raw_url = if url.contains("github.com") && !url.contains("raw.githubusercontent.com") {
        // https://github.com/owner/repo/tree/branch/skill-name -> raw URL
        let converted = url
            .replace("github.com", "raw.githubusercontent.com")
            .replace("/tree/", "/")
            .replace("/blob/", "/");
        if converted.ends_with("/SKILL.md") {
            converted
        } else {
            format!("{}/SKILL.md", converted.trim_end_matches('/'))
        }
    } else if url.ends_with("/SKILL.md") {
        url.clone()
    } else {
        format!("{}/SKILL.md", url.trim_end_matches('/'))
    };

    // 下载 SKILL.md 内容
    let content = client
        .get(&raw_url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("下载技能失败: {e}"))?
        .text()
        .await
        .map_err(|e| format!("读取响应内容失败: {e}"))?;

    // 从 URL 推导技能名
    let skill_name = url
        .trim_end_matches('/')
        .trim_end_matches("/SKILL.md")
        .split('/')
        .next_back()
        .unwrap_or("imported-skill")
        .to_string();

    // 对每个 agent（平台）安装
    let home = dirs::home_dir().ok_or_else(|| "无法获取主目录".to_string())?;
    let semaphore = Arc::new(Semaphore::new(4));
    let mut jobs = JoinSet::new();

    for agent in agents {
        let permit_pool = Arc::clone(&semaphore);
        let home = home.clone();
        let skill_name = skill_name.clone();
        let content = content.clone();
        let source_url = url.clone();
        jobs.spawn(async move {
            let permit = match permit_pool.acquire_owned().await {
                Ok(permit) => permit,
                Err(e) => {
                    return serde_json::json!({
                        "agent": agent,
                        "ok": false,
                        "message": format!("并发控制失败: {e}"),
                    });
                }
            };

            let agent_for_join_error = agent.clone();
            let result = tokio::task::spawn_blocking(move || {
                let (_, _, rel_path) = match find_platform(&agent) {
                    Some(p) => p,
                    None => {
                        return serde_json::json!({
                            "agent": agent,
                            "ok": false,
                            "message": format!("未知平台: {agent}"),
                        });
                    }
                };

                let skill_dir = home.join(rel_path).join(&skill_name);
                if skill_dir.exists() && !force {
                    return serde_json::json!({
                        "agent": agent,
                        "ok": false,
                        "message": format!("技能已存在: {}（使用 force 覆盖）", skill_dir.display()),
                    });
                }

                if let Err(e) = std::fs::create_dir_all(&skill_dir) {
                    return serde_json::json!({
                        "agent": agent,
                        "ok": false,
                        "message": format!("创建目录失败: {e}"),
                    });
                }
                if let Err(e) = std::fs::write(skill_dir.join("SKILL.md"), &content) {
                    return serde_json::json!({
                        "agent": agent,
                        "ok": false,
                        "message": format!("写入 SKILL.md 失败: {e}"),
                    });
                }

                let _ = write_skill_meta(&skill_dir, "github", Some(&source_url));

                serde_json::json!({
                    "agent": agent,
                    "ok": true,
                    "message": format!("已安装到 {}", skill_dir.display()),
                })
            })
            .await;

            drop(permit);

            match result {
                Ok(value) => value,
                Err(e) => serde_json::json!({
                    "agent": agent_for_join_error,
                    "ok": false,
                    "message": format!("安装任务失败: {e}"),
                }),
            }
        });
    }

    let mut results: Vec<Value> = Vec::new();
    while let Some(job) = jobs.join_next().await {
        match job {
            Ok(value) => results.push(value),
            Err(e) => results.push(serde_json::json!({
                "agent": "unknown",
                "ok": false,
                "message": format!("安装任务 join 失败: {e}"),
            })),
        }
    }

    Ok(serde_json::json!({ "results": results }))
}

// ── Step 7b: 本地目录导入（新增） ──

#[tauri::command]
pub async fn skill_hub_import_local(
    source_path: String,
    agents: Vec<String>,
    skill_name: Option<String>,
) -> Result<Value, String> {
    let results = tokio::task::spawn_blocking(move || {
        let src = std::path::Path::new(&source_path);
        let src_skill = src.join("SKILL.md");
        if !src_skill.exists() {
            return Err(format!("源目录中不存在 SKILL.md: {source_path}"));
        }

        let name = skill_name.unwrap_or_else(|| {
            src.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "local-skill".to_string())
        });

        let home = dirs::home_dir().ok_or_else(|| "无法获取主目录".to_string())?;
        let mut results: Vec<Value> = Vec::new();

        for agent in &agents {
            let (_, _, rel_path) = match find_platform(agent) {
                Some(p) => p,
                None => {
                    results.push(serde_json::json!({
                        "agent": agent, "ok": false,
                        "message": format!("未知平台: {agent}"),
                    }));
                    continue;
                }
            };

            let dest_dir = home.join(rel_path).join(&name);
            if let Err(e) = std::fs::create_dir_all(&dest_dir) {
                results.push(serde_json::json!({
                    "agent": agent, "ok": false, "message": format!("创建目录失败: {e}"),
                }));
                continue;
            }

            // 复制所有文件（浅层）
            let copy_result = (|| -> Result<(), String> {
                for entry in std::fs::read_dir(src).map_err(|e| e.to_string())?.flatten() {
                    let ep = entry.path();
                    if ep.is_file() {
                        let dest_file = dest_dir.join(ep.file_name().unwrap_or_default());
                        std::fs::copy(&ep, &dest_file).map_err(|e| format!("复制文件失败: {e}"))?;
                    }
                }
                Ok(())
            })();

            match copy_result {
                Ok(()) => {
                    let _ = write_skill_meta(&dest_dir, "local", Some(&source_path));
                    results.push(serde_json::json!({
                        "agent": agent, "ok": true,
                        "message": format!("已安装到 {}", dest_dir.display()),
                    }));
                }
                Err(e) => {
                    results.push(serde_json::json!({
                        "agent": agent, "ok": false, "message": e,
                    }));
                }
            }
        }

        Ok::<_, String>(results)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(serde_json::json!({ "results": results }))
}

// ── Step 7c: npx 安装（新增） ──

#[tauri::command]
pub async fn skill_hub_import_npx(
    package_name: String,
    agents: Vec<String>,
    global: Option<bool>,
) -> Result<Value, String> {
    let _global = global.unwrap_or(false);

    // 先检查 npx 可用性
    let check = tokio_command(if cfg!(windows) { "cmd" } else { "sh" })
        .args(if cfg!(windows) {
            vec!["/C", "npx", "--version"]
        } else {
            vec!["-c", "npx --version"]
        })
        .output()
        .await;

    if check.is_err() || !check.as_ref().is_ok_and(|r| r.status.success()) {
        return Ok(serde_json::json!({
            "success": false,
            "method": "npx",
            "stderr": "npx 不可用",
            "results": [],
        }));
    }

    // 运行 npx skills install
    let mut cmd_args = vec!["skills", "install", &package_name];
    // 附加平台参数
    let agents_str = agents.join(",");
    if !agents.is_empty() {
        cmd_args.push("--agents");
        cmd_args.push(&agents_str);
    }

    let result = tokio_command("npx")
        .args(&cmd_args)
        .output()
        .await
        .map_err(|e| format!("执行 npx 失败: {e}"))?;

    let stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let stderr = String::from_utf8_lossy(&result.stderr).to_string();

    let agent_results: Vec<Value> = agents
        .iter()
        .map(|a| {
            serde_json::json!({
                "agent": a,
                "ok": result.status.success(),
                "message": if result.status.success() { "npx 安装成功" } else { "npx 安装失败" },
            })
        })
        .collect();

    Ok(serde_json::json!({
        "success": result.status.success(),
        "method": "npx",
        "stdout": stdout,
        "stderr": stderr,
        "results": agent_results,
    }))
}

// ── Step 7d: 批量安装（新增） ──

#[tauri::command]
pub async fn skill_hub_batch_install(
    state: State<'_, AppState>,
    packages: Vec<String>,
    agents: Vec<String>,
    force: Option<bool>,
) -> Result<Value, String> {
    let force = force.unwrap_or(false);
    let client = state.http_client.clone();
    let mut all_results: Vec<Value> = Vec::new();
    let mut install_details: Vec<Value> = Vec::new();
    let mut success_count = 0usize;
    let mut fail_count = 0usize;
    let mut package_stats: HashMap<String, (usize, usize)> = HashMap::new();
    let semaphore = Arc::new(Semaphore::new(4));
    let mut jobs = JoinSet::new();

    let home = dirs::home_dir().ok_or_else(|| "无法获取主目录".to_string())?;
    for pkg in &packages {
        package_stats.insert(pkg.clone(), (0, 0));
    }

    for pkg in &packages {
        // 将 GitHub URL 转换为 raw URL
        let raw_url = if pkg.contains("github.com") && !pkg.contains("raw.githubusercontent.com") {
            let converted = pkg
                .replace("github.com", "raw.githubusercontent.com")
                .replace("/tree/", "/")
                .replace("/blob/", "/");
            if converted.ends_with("/SKILL.md") {
                converted
            } else {
                format!("{}/SKILL.md", converted.trim_end_matches('/'))
            }
        } else if pkg.ends_with("/SKILL.md") {
            pkg.clone()
        } else {
            format!("{}/SKILL.md", pkg.trim_end_matches('/'))
        };

        // 下载 SKILL.md
        let content = match client
            .get(&raw_url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
        {
            Ok(resp) => match resp.text().await {
                Ok(text) => text,
                Err(e) => {
                    if let Some(stats) = package_stats.get_mut(pkg) {
                        stats.1 += agents.len().max(1);
                    }
                    install_details.push(serde_json::json!({
                        "package": pkg,
                        "agent": "download",
                        "ok": false,
                        "message": format!("读取响应失败: {e}"),
                    }));
                    continue;
                }
            },
            Err(e) => {
                if let Some(stats) = package_stats.get_mut(pkg) {
                    stats.1 += agents.len().max(1);
                }
                install_details.push(serde_json::json!({
                    "package": pkg,
                    "agent": "download",
                    "ok": false,
                    "message": format!("下载失败: {e}"),
                }));
                continue;
            }
        };

        // 推导技能名
        let skill_name = pkg
            .trim_end_matches('/')
            .trim_end_matches("/SKILL.md")
            .split('/')
            .next_back()
            .unwrap_or("imported-skill")
            .to_string();

        for agent in &agents {
            let permit_pool = Arc::clone(&semaphore);
            let home = home.clone();
            let agent = agent.clone();
            let package = pkg.clone();
            let source_url = pkg.clone();
            let skill_name = skill_name.clone();
            let content = content.clone();

            jobs.spawn(async move {
                let permit = match permit_pool.acquire_owned().await {
                    Ok(permit) => permit,
                    Err(e) => {
                        return serde_json::json!({
                            "package": package,
                            "agent": agent,
                            "ok": false,
                            "message": format!("并发控制失败: {e}"),
                        });
                    }
                };

                let package_for_join_error = package.clone();
                let agent_for_join_error = agent.clone();
                let result = tokio::task::spawn_blocking(move || {
                    let (_, _, rel_path) = match find_platform(&agent) {
                        Some(p) => p,
                        None => {
                            return serde_json::json!({
                                "package": package,
                                "agent": agent,
                                "ok": false,
                                "message": format!("未知平台: {agent}"),
                            });
                        }
                    };

                    let skill_dir = home.join(rel_path).join(&skill_name);
                    if skill_dir.exists() && !force {
                        return serde_json::json!({
                            "package": package,
                            "agent": agent,
                            "ok": false,
                            "message": format!("技能已存在: {}（使用 force 覆盖）", skill_dir.display()),
                        });
                    }
                    if let Err(e) = std::fs::create_dir_all(&skill_dir) {
                        return serde_json::json!({
                            "package": package,
                            "agent": agent,
                            "ok": false,
                            "message": format!("创建目录失败: {e}"),
                        });
                    }
                    if let Err(e) = std::fs::write(skill_dir.join("SKILL.md"), &content) {
                        return serde_json::json!({
                            "package": package,
                            "agent": agent,
                            "ok": false,
                            "message": format!("写入 SKILL.md 失败: {e}"),
                        });
                    }

                    let _ = write_skill_meta(&skill_dir, "github", Some(&source_url));

                    serde_json::json!({
                        "package": package,
                        "agent": agent,
                        "ok": true,
                        "message": format!("已安装到 {}", skill_dir.display()),
                    })
                })
                .await;

                drop(permit);

                match result {
                    Ok(value) => value,
                    Err(e) => serde_json::json!({
                        "package": package_for_join_error,
                        "agent": agent_for_join_error,
                        "ok": false,
                        "message": format!("安装任务失败: {e}"),
                    }),
                }
            });
        }
    }

    while let Some(job) = jobs.join_next().await {
        let detail = match job {
            Ok(value) => value,
            Err(e) => serde_json::json!({
                "package": "unknown",
                "agent": "unknown",
                "ok": false,
                "message": format!("安装任务 join 失败: {e}"),
            }),
        };

        let package = detail
            .get("package")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let ok = detail.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);

        let stats = package_stats.entry(package).or_insert((0, 0));
        if ok {
            stats.0 += 1;
        } else {
            stats.1 += 1;
        }

        install_details.push(detail);
    }

    for pkg in &packages {
        let (success_agents, failed_agents) = package_stats.get(pkg).copied().unwrap_or((0, 1));
        let pkg_ok = success_agents > 0 && failed_agents == 0;

        if pkg_ok {
            success_count += 1;
        } else {
            fail_count += 1;
        }

        let message = if agents.is_empty() {
            "未指定安装平台"
        } else if pkg_ok {
            "安装成功"
        } else {
            "部分平台安装失败"
        };

        all_results.push(serde_json::json!({
            "package": pkg,
            "ok": pkg_ok,
            "message": message,
            "success_agents": success_agents,
            "failed_agents": failed_agents,
        }));
    }

    Ok(serde_json::json!({
        "total": packages.len(),
        "success_count": success_count,
        "fail_count": fail_count,
        "results": all_results,
        "details": install_details,
    }))
}

// ── 保留的原有命令 ──

#[tauri::command]
pub async fn skill_hub_trending(state: State<'_, AppState>) -> Result<Value, String> {
    let client = state.http_client.clone();
    let result = client
        .get("https://skills.sh/api/trending")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    match result {
        Ok(resp) if resp.status().is_success() => {
            let body: Value = resp.json().await.unwrap_or(serde_json::json!({
                "skills": [],
                "updated_at": chrono::Utc::now().to_rfc3339()
            }));
            Ok(body)
        }
        _ => Ok(serde_json::json!({
            "skills": [],
            "updated_at": chrono::Utc::now().to_rfc3339()
        })),
    }
}

#[tauri::command]
pub async fn skill_hub_search(query: String, category: Option<String>) -> Result<Value, String> {
    let results = tokio::task::spawn_blocking(move || {
        let home = dirs::home_dir().ok_or_else(|| "无法获取主目录".to_string())?;
        let q_lower = query.to_lowercase();
        let cat_lower = category.as_deref().map(str::to_lowercase);

        let mut matches: Vec<Value> = Vec::new();

        // 搜索所有平台的 skills 目录
        for &(id, display_name, rel_path) in PLATFORM_SKILLS_DIRS {
            let dir = home.join(rel_path);
            if !dir.is_dir() {
                continue;
            }
            for skill in scan_platform_skills(&dir, id, display_name) {
                let name = skill
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_lowercase();
                let desc = skill
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_lowercase();

                if !name.contains(&q_lower) && !desc.contains(&q_lower) {
                    continue;
                }

                if let Some(ref cat) = cat_lower {
                    let skill_cat = skill
                        .get("category")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_lowercase();
                    if !skill_cat.contains(cat.as_str()) {
                        continue;
                    }
                }

                matches.push(skill);
            }
        }

        Ok::<_, String>(matches)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    let total = results.len();
    Ok(serde_json::json!({
        "results": results,
        "total": total
    }))
}

#[tauri::command]
pub async fn skill_hub_install(
    state: State<'_, AppState>,
    skill_url: String,
    target_dir: Option<String>,
) -> Result<Value, String> {
    let client = state.http_client.clone();

    let raw_url =
        if skill_url.contains("github.com") && !skill_url.contains("raw.githubusercontent.com") {
            skill_url
                .replace("github.com", "raw.githubusercontent.com")
                .replace("/blob/", "/")
        } else {
            skill_url.clone()
        };

    let content = client
        .get(&raw_url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("下载技能失败: {e}"))?
        .text()
        .await
        .map_err(|e| format!("读取响应内容失败: {e}"))?;

    let install_dir = if let Some(dir) = target_dir {
        std::path::PathBuf::from(dir)
    } else {
        let home = dirs::home_dir().ok_or_else(|| "无法获取主目录".to_string())?;
        home.join(".claude").join("skills")
    };

    if !install_dir.exists() {
        std::fs::create_dir_all(&install_dir).map_err(|e| format!("创建目录失败: {e}"))?;
    }

    let file_name = skill_url
        .split('/')
        .next_back()
        .unwrap_or("skill.md")
        .to_string();
    let file_name = if file_name.is_empty() {
        "skill.md".to_string()
    } else {
        file_name
    };
    let file_name_for_write = file_name.clone();

    let installed_path = tokio::task::spawn_blocking(move || {
        if !install_dir.exists() {
            std::fs::create_dir_all(&install_dir).map_err(|e| format!("创建目录失败: {e}"))?;
        }

        let installed_path = install_dir.join(&file_name_for_write);
        std::fs::write(&installed_path, &content).map_err(|e| format!("写入文件失败: {e}"))?;

        Ok::<_, String>(installed_path)
    })
    .await
    .map_err(|e| format!("安装任务失败: {e}"))??;

    Ok(serde_json::json!({
        "success": true,
        "installed_path": installed_path.to_string_lossy(),
        "file_name": file_name,
    }))
}

#[tauri::command]
pub async fn skill_hub_remove(skill_path: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let path = std::path::Path::new(&skill_path);
        if !path.exists() {
            return Err(format!("路径不存在: {}", skill_path));
        }
        // 如果是目录则递归删除，如果是文件则删除文件
        if path.is_dir() {
            std::fs::remove_dir_all(path).map_err(|e| format!("删除目录失败: {e}"))?;
        } else {
            std::fs::remove_file(path).map_err(|e| format!("删除文件失败: {e}"))?;
        }
        Ok(serde_json::json!({ "success": true }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
