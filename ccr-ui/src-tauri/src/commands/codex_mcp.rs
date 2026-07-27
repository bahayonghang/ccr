use super::*;

/// 列出 config.toml 中的 [mcp_servers]
#[tauri::command]
pub async fn codex_list_mcp_servers() -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(|| -> Result<Value, String> {
        let path = codex_config_path()?;
        let config = read_codex_config(&path)?;
        let servers: Vec<Value> = config
            .mcp_servers
            .unwrap_or_default()
            .into_iter()
            .map(|(name, server)| {
                let enabled = server.enabled.unwrap_or(true);
                json!({
                    "name": name,
                    "enabled": enabled,
                    "transport": if server.url.is_some() { "http" } else { "stdio" },
                    "command": server.command,
                    "args": server.args,
                    "env": server.env,
                    "env_vars": server.env_vars,
                    "cwd": server.cwd,
                    "startup_timeout_ms": server.startup_timeout_ms,
                    "startup_timeout_sec": server.startup_timeout_sec,
                    "tool_timeout_sec": server.tool_timeout_sec,
                    "url": server.url,
                    "http_headers": server.http_headers,
                    "env_http_headers": server.env_http_headers,
                    "bearer_token": server.bearer_token,
                    "bearer_token_env_var": server.bearer_token_env_var,
                    "oauth_resource": server.oauth_resource,
                    "scopes": server.scopes,
                    "enabled_tools": server.enabled_tools,
                    "disabled_tools": server.disabled_tools,
                    "required": server.required,
                })
            })
            .collect();
        Ok(json!({ "servers": servers }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

/// 添加 MCP 服务器到 config.toml
#[tauri::command]
pub async fn codex_add_mcp_server(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let config: Value = config.into();
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let path = codex_config_path()?;
        let mut cfg = read_codex_config(&path)?;

        if let Some(ref servers) = cfg.mcp_servers
            && servers.contains_key(&name)
        {
            return Err(format!("MCP 服务器 '{name}' 已存在"));
        }

        let server = parse_mcp_server(&config)?;
        validate_mcp_server(&server)?;
        cfg.mcp_servers
            .get_or_insert_with(HashMap::new)
            .insert(name.clone(), server);

        write_codex_config(&path, &cfg)?;
        Ok(json!({ "message": format!("MCP 服务器 '{name}' 已添加") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    open_json(response)
}

/// 更新已有 MCP 服务器
#[tauri::command]
pub async fn codex_update_mcp_server(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let config: Value = config.into();
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

        let existing = servers
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("MCP 服务器 '{name}' 不存在"))?;

        let mut server = parse_mcp_server(&config)?;
        validate_mcp_server(&server)?;
        merge_codex_mcp_server(&mut server, &existing);
        servers.insert(name.clone(), server);

        write_codex_config(&path, &cfg)?;
        Ok(json!({ "message": format!("MCP 服务器 '{name}' 已更新") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    open_json(response)
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
