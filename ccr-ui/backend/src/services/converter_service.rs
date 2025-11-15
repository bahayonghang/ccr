// 配置转换器核心逻辑

use crate::models::converter::*;
use crate::models::platforms::codex::{CodexConfig, CodexMcpServer};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing::info;

pub struct ConfigConverter;

impl ConfigConverter {
    /// 执行配置转换
    pub fn convert(request: ConverterRequest) -> Result<ConverterResponse, String> {
        info!(
            "开始转换配置: {} -> {}",
            format!("{:?}", request.source_format),
            format!("{:?}", request.target_format)
        );

        // Step 1: 解析源配置到中间格式
        let intermediate = Self::parse_source(&request)?;

        // Step 2: 从中间格式生成目标配置
        let (converted_data, format) =
            Self::generate_target(&request.target_format, &intermediate)?;

        // Step 3: 统计转换结果
        let stats = ConversionStats {
            mcp_servers: intermediate.mcp_servers.len(),
            slash_commands: intermediate.commands.len(),
            agents: intermediate.agents.len(),
            profiles: intermediate.profiles.len(),
            base_config: true,
        };

        Ok(ConverterResponse {
            success: true,
            converted_data,
            warnings: vec![],
            format,
            stats,
        })
    }

    /// 解析源配置
    fn parse_source(request: &ConverterRequest) -> Result<IntermediateConfig, String> {
        match request.source_format {
            CliType::ClaudeCode => Self::parse_claude_code(&request.config_data),
            CliType::Codex => Self::parse_codex(&request.config_data),
            _ => Err(format!("暂不支持从 {:?} 格式转换", request.source_format)),
        }
    }

    /// 生成目标配置
    fn generate_target(
        target_format: &CliType,
        intermediate: &IntermediateConfig,
    ) -> Result<(String, String), String> {
        match target_format {
            CliType::ClaudeCode => {
                let json = Self::generate_claude_code(intermediate)?;
                Ok((json, "json".to_string()))
            }
            CliType::Codex => {
                let toml = Self::generate_codex(intermediate)?;
                Ok((toml, "toml".to_string()))
            }
            _ => Err(format!("暂不支持转换到 {:?} 格式", target_format)),
        }
    }

    // ============ Claude Code 解析 ============

    fn parse_claude_code(json_str: &str) -> Result<IntermediateConfig, String> {
        let value: JsonValue = serde_json::from_str(json_str)
            .map_err(|e| format!("解析 Claude Code JSON 失败: {}", e))?;

        let mut config = IntermediateConfig::default();

        // 解析 MCP 服务器
        if let Some(mcp_servers) = value.get("mcpServers").and_then(|v| v.as_object()) {
            for (name, server_value) in mcp_servers {
                if let Some(server_obj) = server_value.as_object() {
                    let server = GenericMcpServer {
                        name: name.clone(),
                        command: server_obj
                            .get("command")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        args: server_obj
                            .get("args")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        env: server_obj
                            .get("env")
                            .and_then(|v| v.as_object())
                            .map(|obj| {
                                obj.iter()
                                    .filter_map(|(k, v)| {
                                        v.as_str().map(|s| (k.clone(), s.to_string()))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                        url: server_obj
                            .get("url")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        bearer_token: None,
                        startup_timeout_ms: None,
                        cwd: None,
                    };
                    config.mcp_servers.push(server);
                }
            }
        }

        info!(
            "从 Claude Code 解析了 {} 个 MCP 服务器",
            config.mcp_servers.len()
        );
        Ok(config)
    }

    // ============ Codex 解析 ============

    fn parse_codex(toml_str: &str) -> Result<IntermediateConfig, String> {
        let codex_config: CodexConfig =
            toml::from_str(toml_str).map_err(|e| format!("解析 Codex TOML 失败: {}", e))?;

        let mut config = IntermediateConfig::default();

        // 解析 MCP 服务器
        if let Some(mcp_servers) = codex_config.mcp_servers {
            for (name, server) in mcp_servers {
                let generic_server = GenericMcpServer {
                    name,
                    command: server.command,
                    args: server.args.unwrap_or_default(),
                    env: server.env.unwrap_or_default(),
                    url: server.url,
                    bearer_token: server.bearer_token,
                    startup_timeout_ms: server.startup_timeout_ms,
                    cwd: server.cwd,
                };
                config.mcp_servers.push(generic_server);
            }
        }

        // 解析 Profiles
        if let Some(profiles) = codex_config.profiles {
            for (name, profile) in profiles {
                let generic_profile = GenericProfile {
                    name,
                    model: profile.model,
                    approval_policy: profile.approval_policy,
                    sandbox_mode: profile.sandbox_mode,
                };
                config.profiles.push(generic_profile);
            }
        }

        info!(
            "从 Codex 解析了 {} 个 MCP 服务器和 {} 个 Profiles",
            config.mcp_servers.len(),
            config.profiles.len()
        );
        Ok(config)
    }

    // ============ Claude Code 生成 ============

    fn generate_claude_code(intermediate: &IntermediateConfig) -> Result<String, String> {
        let mut root = serde_json::Map::new();

        // 生成 mcpServers
        let mut mcp_servers_obj = serde_json::Map::new();
        for server in &intermediate.mcp_servers {
            let mut server_obj = serde_json::Map::new();

            if let Some(ref command) = server.command {
                server_obj.insert("command".to_string(), JsonValue::String(command.clone()));
                server_obj.insert(
                    "args".to_string(),
                    JsonValue::Array(
                        server
                            .args
                            .iter()
                            .map(|s| JsonValue::String(s.clone()))
                            .collect(),
                    ),
                );
            }

            if let Some(ref url) = server.url {
                server_obj.insert("url".to_string(), JsonValue::String(url.clone()));
            }

            if !server.env.is_empty() {
                let env_obj: serde_json::Map<String, JsonValue> = server
                    .env
                    .iter()
                    .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
                    .collect();
                server_obj.insert("env".to_string(), JsonValue::Object(env_obj));
            }

            mcp_servers_obj.insert(server.name.clone(), JsonValue::Object(server_obj));
        }

        root.insert("mcpServers".to_string(), JsonValue::Object(mcp_servers_obj));

        serde_json::to_string_pretty(&root)
            .map_err(|e| format!("生成 Claude Code JSON 失败: {}", e))
    }

    // ============ Codex 生成 ============

    fn generate_codex(intermediate: &IntermediateConfig) -> Result<String, String> {
        let mut codex_config = CodexConfig::default();

        // 生成 MCP 服务器
        let mut mcp_servers = HashMap::new();
        for server in &intermediate.mcp_servers {
            let codex_server = CodexMcpServer {
                command: server.command.clone(),
                args: if server.args.is_empty() {
                    None
                } else {
                    Some(server.args.clone())
                },
                env: if server.env.is_empty() {
                    None
                } else {
                    Some(server.env.clone())
                },
                cwd: server.cwd.clone(),
                startup_timeout_ms: server.startup_timeout_ms.or(Some(20000)),
                url: server.url.clone(),
                bearer_token: server.bearer_token.clone(),
                other: HashMap::new(),
            };
            mcp_servers.insert(server.name.clone(), codex_server);
        }
        codex_config.mcp_servers = Some(mcp_servers);

        toml::to_string_pretty(&codex_config).map_err(|e| format!("生成 Codex TOML 失败: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_to_codex_conversion() {
        let claude_json = r#"{
            "mcpServers": {
                "context7": {
                    "command": "npx",
                    "args": ["-y", "@upstash/context7-mcp"],
                    "env": {
                        "API_KEY": "test-key"
                    }
                }
            }
        }"#;

        let request = ConverterRequest {
            source_format: CliType::ClaudeCode,
            target_format: CliType::Codex,
            config_data: claude_json.to_string(),
            convert_mcp: true,
            convert_commands: false,
            convert_agents: false,
        };

        let result = ConfigConverter::convert(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.format, "toml");
        assert_eq!(response.stats.mcp_servers, 1);
    }

    #[test]
    fn test_codex_to_claude_conversion() {
        let codex_toml = r#"
[mcp_servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp"]
startup_timeout_ms = 20000

[mcp_servers.context7.env]
API_KEY = "test-key"
"#;

        let request = ConverterRequest {
            source_format: CliType::Codex,
            target_format: CliType::ClaudeCode,
            config_data: codex_toml.to_string(),
            convert_mcp: true,
            convert_commands: false,
            convert_agents: false,
        };

        let result = ConfigConverter::convert(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.format, "json");
        assert_eq!(response.stats.mcp_servers, 1);
    }
}
