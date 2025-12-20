# provider

Provider 健康检查命令，用于测试 API 端点连通性和验证 API Key。

## 概述

`ccr provider` 命令组提供对配置中 Provider 的健康检查功能：

- **连通性测试**: 检测 API 端点是否可达
- **API Key 验证**: 验证密钥是否有效
- **延迟测量**: 测量 API 响应时间
- **模型可用性**: 检查配置的模型是否可用

## 子命令

### test

测试 Provider 连通性。

```bash
ccr provider test <NAME|--all> [OPTIONS]
```

**参数：**

| 参数 | 说明 |
|------|------|
| `<NAME>` | 要测试的配置名称 |

**选项：**

| 选项 | 说明 |
|------|------|
| `-a, --all` | 测试所有配置 |
| `-v, --verbose` | 显示详细信息 |

**输出信息：**

- Provider 名称
- Base URL
- 健康状态（Healthy/Degraded/Unhealthy/Unknown）
- 延迟（毫秒）
- 错误信息（如果有）

**示例：**

```bash
# 测试单个配置
ccr provider test my-provider

# 测试所有配置
ccr provider test --all

# 显示详细信息
ccr provider test my-provider --verbose
```

### verify

验证 API Key 有效性。

```bash
ccr provider verify <NAME>
```

通过调用 `/v1/models` 端点验证 API Key 是否有效，并列出可用模型。

**输出：**

- Key 有效性状态
- 可用模型列表
- 配置的模型是否在列表中

**示例：**

```bash
# 验证 API Key
ccr provider verify my-provider
```

## 健康状态说明

| 状态 | 说明 |
|------|------|
| `Healthy` | 端点可达，API Key 有效 |
| `Degraded` | 端点可达但响应较慢或部分功能受限 |
| `Unhealthy` | 端点不可达或 API Key 无效 |
| `Unknown` | 无法确定状态 |

## 测试流程

1. **连通性测试**: 请求 `{base_url}/v1/models` 端点
2. **认证测试**: 检查 HTTP 状态码（401/403 = 认证失败）
3. **延迟测试**: 记录请求往返时间
4. **模型验证**: 确认配置的 model 在返回列表中

## 使用场景

### 诊断连接问题

```bash
# 测试特定配置
ccr provider test my-provider --verbose

# 查看详细错误信息
```

### 批量检查所有配置

```bash
# 一次性测试所有配置状态
ccr provider test --all
```

### 验证新配置

```bash
# 添加新配置后验证
ccr add
ccr provider verify new-config
```

### CI/CD 集成

```bash
# 在脚本中检查 Provider 状态
if ccr provider test my-provider; then
    echo "Provider is healthy"
else
    echo "Provider check failed"
    exit 1
fi
```

## Web API

Provider 健康检查也可通过 Web API 访问：

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/provider-health/test` | POST | 测试单个 Provider |
| `/api/provider-health/test-all` | GET | 测试所有 Provider |

## 版本信息

- **引入版本**: v3.12.0
- **依赖特性**: 需要 `web` 特性（网络请求）
