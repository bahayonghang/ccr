//! 技能管理命令 — Skills CRUD + 仓库管理。

use serde_json::Value;

// ── 辅助函数 ──

fn make_manager() -> Result<ccr::managers::skills_manager::SkillsManager, String> {
    ccr::managers::skills_manager::SkillsManager::new(ccr::Platform::Claude)
        .map_err(|e| format!("Failed to create skills manager: {e}"))
}

/// 从任意目录扫描技能文件夹（每个子目录含 SKILL.md 即为一个技能）。
fn scan_skills_dir(base: &std::path::Path) -> Vec<ccr::models::skill::Skill> {
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
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let instruction = std::fs::read_to_string(&skill_file).unwrap_or_default();
        let (metadata, description) =
            ccr::models::skill::Skill::parse_with_fallback(&instruction);
        skills.push(ccr::models::skill::Skill {
            name,
            description,
            path: path.to_string_lossy().to_string(),
            instruction,
            metadata,
            is_remote: false,
            repository: None,
        });
    }
    skills
}

// ── 技能 CRUD ──

#[tauri::command]
pub async fn list_skills() -> Result<Value, String> {
    let skills = tokio::task::spawn_blocking(|| {
        let manager = make_manager()?;

        // 1) 用户全局技能 (~/.claude/skills/)
        let mut all_skills = manager.list_skills().map_err(|e| format!("Failed to list skills: {e}"))?;

        // 2) 项目本地技能 (./.claude/skills/)
        let project_skills: Vec<ccr::models::skill::Skill> = (|| {
            let cwd = std::env::current_dir().ok()?;
            let skills_dir = cwd.join(".claude").join("skills");
            Some(scan_skills_dir(&skills_dir))
        })()
        .unwrap_or_default();

        // 3) 插件目录技能 (~/.claude/plugins/*/skills/)
        let plugin_skills: Vec<ccr::models::skill::Skill> = (|| -> Option<Vec<_>> {
            let home = dirs::home_dir()?;
            let plugins_dir = home.join(".claude").join("plugins");
            if !plugins_dir.exists() {
                return None;
            }
            let mut skills = Vec::new();
            for entry in std::fs::read_dir(&plugins_dir).ok()?.flatten() {
                let plugin_skills_dir = entry.path().join("skills");
                if plugin_skills_dir.is_dir() {
                    skills.extend(scan_skills_dir(&plugin_skills_dir));
                }
            }
            Some(skills)
        })()
        .unwrap_or_default();

        // Merge: user skills first, then project, then plugin (dedup by name)
        let mut seen = std::collections::HashSet::new();
        for s in &all_skills {
            seen.insert(s.name.clone());
        }
        for s in project_skills {
            if seen.insert(s.name.clone()) {
                all_skills.push(s);
            }
        }
        for s in plugin_skills {
            if seen.insert(s.name.clone()) {
                all_skills.push(s);
            }
        }

        Ok::<_, String>(all_skills)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    serde_json::to_value(skills).map_err(|e| format!("Serialization error: {e}"))
}

#[tauri::command]
pub async fn add_skill(name: String, instruction: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let manager = make_manager()?;
        manager
            .install_skill(&name, &instruction)
            .map_err(|e| format!("Failed to add skill: {e}"))?;
        Ok::<_, String>(serde_json::json!({ "success": true, "name": name }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn delete_skill(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let manager = make_manager()?;
        manager
            .uninstall_skill(&name)
            .map_err(|e| format!("Failed to delete skill: {e}"))?;
        Ok::<_, String>(serde_json::json!({ "success": true, "name": name }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

// ── 单个技能查询与更新 ──

#[tauri::command]
pub async fn get_skill(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let manager = make_manager()?;
        let all_skills = manager
            .list_skills()
            .map_err(|e| format!("Failed to list skills: {e}"))?;

        let skill = all_skills
            .into_iter()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;

        serde_json::to_value(skill).map_err(|e| format!("Serialization error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn update_skill(name: String, instruction: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let manager = make_manager()?;

        // 先卸载旧版本再重新安装（与原始后端一致）
        // 忽略卸载错误（技能可能不存在）
        let _ = manager.uninstall_skill(&name);

        manager
            .install_skill(&name, &instruction)
            .map_err(|e| format!("Failed to update skill: {e}"))?;

        Ok::<_, String>(serde_json::json!({ "success": true, "name": name }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

// ── 仓库管理 ──

#[tauri::command]
pub async fn list_skill_repositories() -> Result<Value, String> {
    let repos = tokio::task::spawn_blocking(|| {
        let manager = make_manager()?;
        manager
            .list_repositories()
            .map_err(|e| format!("Failed to list repositories: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    serde_json::to_value(repos).map_err(|e| format!("Serialization error: {e}"))
}

#[tauri::command]
pub async fn add_skill_repository(repo: Value) -> Result<Value, String> {
    let repo_struct: ccr::models::skill::SkillRepository =
        serde_json::from_value(repo).map_err(|e| format!("Invalid repository data: {e}"))?;

    let name = repo_struct.name.clone();
    tokio::task::spawn_blocking(move || {
        let manager = make_manager()?;
        manager
            .add_repository(repo_struct)
            .map_err(|e| format!("Failed to add repository: {e}"))?;
        Ok::<_, String>(serde_json::json!({ "success": true, "name": name }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn remove_skill_repository(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let manager = make_manager()?;
        manager
            .remove_repository(&name)
            .map_err(|e| format!("Failed to remove repository: {e}"))?;
        Ok::<_, String>(serde_json::json!({ "success": true, "name": name }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

// ── 远程仓库扫描 ──

#[tauri::command]
pub async fn scan_skill_repository(url: String) -> Result<Value, String> {
    // Parse GitHub URL: https://github.com/owner/repo
    let url_parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();

    if url_parts.len() < 5 || url_parts.get(2) != Some(&"github.com") {
        return Ok(serde_json::json!({
            "url": url,
            "skills": [],
            "message": "Only GitHub repositories (https://github.com/owner/repo) are supported for scanning"
        }));
    }

    let owner = url_parts[3].to_string();
    let repo_name = url_parts[4].to_string();
    let branch = "main".to_string();

    let api_url = format!(
        "https://api.github.com/repos/{}/{}/contents",
        owner, repo_name
    );

    let client = reqwest::Client::builder()
        .user_agent("ccr-ui/1.0")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let resp = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch repository contents: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "GitHub API returned {}: {}",
            resp.status().as_u16(),
            resp.status().canonical_reason().unwrap_or("Unknown")
        ));
    }

    let contents: Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {e}"))?;

    let mut skills = Vec::new();

    if let Some(array) = contents.as_array() {
        for item in array {
            if item["type"].as_str() == Some("dir") {
                if let Some(name) = item["name"].as_str() {
                    let raw_url = format!(
                        "https://raw.githubusercontent.com/{}/{}/{}/{}/SKILL.md",
                        owner, repo_name, branch, name
                    );
                    skills.push(serde_json::json!({
                        "name": name,
                        "description": format!("Remote skill from {}/{}", owner, repo_name),
                        "path": raw_url,
                        "is_remote": true,
                        "repository": format!("{}/{}", owner, repo_name),
                    }));
                }
            }
        }
    }

    Ok(serde_json::json!({
        "url": url,
        "owner": owner,
        "repo": repo_name,
        "branch": branch,
        "skills": skills,
        "total": skills.len()
    }))
}
