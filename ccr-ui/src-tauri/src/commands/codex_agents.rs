use super::*;

/// 列出 ~/.codex/agents/ 下的所有 agent markdown 文件
#[tauri::command]
pub async fn codex_list_agents() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let agents_dir = codex_agents_dir()?;
        if !agents_dir.exists() {
            return Ok(json!({ "agents": [] }));
        }

        let mut agents: Vec<Value> = Vec::new();
        for entry in fs::read_dir(&agents_dir).map_err(|e| format!("读取 agents 目录失败: {e}"))?
        {
            let entry = entry.map_err(|e| format!("遍历 agents 目录失败: {e}"))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("读取 agent 文件 '{name}' 失败: {e}"))?;
                let (description, body) = extract_frontmatter_description(&content);
                agents.push(json!({
                    "name": name,
                    "description": description,
                    "content": body,
                }));
            }
        }
        Ok(json!({ "agents": agents }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 添加新 agent（写入 ~/.codex/agents/{name}.md）
#[tauri::command]
pub async fn codex_add_agent(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let agents_dir = codex_agents_dir()?;
        fs::create_dir_all(&agents_dir).map_err(|e| format!("创建 agents 目录失败: {e}"))?;

        let file_path = agents_dir.join(format!("{name}.md"));
        if file_path.exists() {
            return Err(format!("Agent '{name}' 已存在"));
        }

        let content = build_agent_markdown(&config);
        fs::write(&file_path, &content).map_err(|e| format!("写入 agent '{name}' 失败: {e}"))?;

        Ok(json!({ "message": format!("Agent '{name}' 已添加") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

/// 更新已有 agent
#[tauri::command]
pub async fn codex_update_agent(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let agents_dir = codex_agents_dir()?;
        let file_path = agents_dir.join(format!("{name}.md"));
        if !file_path.exists() {
            return Err(format!("Agent '{name}' 不存在"));
        }

        let content = build_agent_markdown(&config);
        fs::write(&file_path, &content).map_err(|e| format!("更新 agent '{name}' 失败: {e}"))?;

        Ok(json!({ "message": format!("Agent '{name}' 已更新") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

/// 删除 agent
#[tauri::command]
pub async fn codex_delete_agent(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let agents_dir = codex_agents_dir()?;
        let file_path = agents_dir.join(format!("{name}.md"));
        if !file_path.exists() {
            return Err(format!("Agent '{name}' 不存在"));
        }

        fs::remove_file(&file_path).map_err(|e| format!("删除 agent '{name}' 失败: {e}"))?;

        Ok(format!("Agent '{name}' 已删除"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}
