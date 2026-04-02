use super::*;

/// 列出 config.toml 中的 [mcp_servers]
#[tauri::command]
pub async fn codex_list_mcp_servers() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let path = codex_config_path()?;
        let config = read_codex_config(&path)?;
        let servers: Vec<Value> = config
            .mcp_servers
            .unwrap_or_default()
            .into_iter()
            .map(|(name, server)| {
                json!({
                    "name": name,
                    "command": server.command,
                    "args": server.args,
                    "env": server.env,
                    "cwd": server.cwd,
                    "startup_timeout_ms": server.startup_timeout_ms,
                    "url": server.url,
                    "bearer_token": server.bearer_token,
                })
            })
            .collect();
        Ok(json!({ "servers": servers }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 添加 MCP 服务器到 config.toml
#[tauri::command]
pub async fn codex_add_mcp_server(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let path = codex_config_path()?;
        let mut cfg = read_codex_config(&path)?;

        if let Some(ref servers) = cfg.mcp_servers
            && servers.contains_key(&name)
        {
            return Err(format!("MCP 服务器 '{name}' 已存在"));
        }

        let server = parse_mcp_server(&config)?;
        cfg.mcp_servers
            .get_or_insert_with(HashMap::new)
            .insert(name.clone(), server);

        write_codex_config(&path, &cfg)?;
        Ok(json!({ "message": format!("MCP 服务器 '{name}' 已添加") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

/// 更新已有 MCP 服务器
#[tauri::command]
pub async fn codex_update_mcp_server(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let path = codex_config_path()?;
        let mut cfg = read_codex_config(&path)?;

        let servers = cfg
            .mcp_servers
            .as_mut()
            .ok_or_else(|| format!("MCP 服务器 '{name}' 不存在"))?;

        if !servers.contains_key(&name) {
            return Err(format!("MCP 服务器 '{name}' 不存在"));
        }

        let server = parse_mcp_server(&config)?;
        servers.insert(name.clone(), server);

        write_codex_config(&path, &cfg)?;
        Ok(json!({ "message": format!("MCP 服务器 '{name}' 已更新") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

/// 删除 MCP 服务器
#[tauri::command]
pub async fn codex_delete_mcp_server(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let path = codex_config_path()?;
        let mut cfg = read_codex_config(&path)?;

        let servers = cfg
            .mcp_servers
            .as_mut()
            .ok_or_else(|| format!("MCP 服务器 '{name}' 不存在"))?;

        if servers.remove(&name).is_none() {
            return Err(format!("MCP 服务器 '{name}' 不存在"));
        }

        write_codex_config(&path, &cfg)?;
        Ok(format!("MCP 服务器 '{name}' 已删除"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}
