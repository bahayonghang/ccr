# doctor - 统一体检

`ccr doctor` 提供一个类似 `omx doctor` 的统一诊断入口，用来聚合 CCR 的本地环境、平台注册表、当前 profile、运行态认证，以及可选的在线 Provider 探活。

## 用法

```bash
ccr doctor
ccr doctor --json
ccr doctor --verbose
ccr doctor --online
ccr doctor --all-platforms
ccr doctor --platform codex
```

## 默认行为

- 默认只做**本地优先、只读、无副作用**检查。
- 默认范围是：**全局 CCR 状态 + 当前平台深度检查**。
- 不会自动修复、初始化或改写任何配置文件。
- 只有 `--online` 时才会执行在线 Provider 探活。

## 选项

- `--json`：输出固定 JSON 结构，便于脚本和 CI 消费
- `--verbose`：额外显示路径、细节和修复建议
- `--online`：启用当前目标平台 current profile 的 Provider 在线检查
- `--all-platforms`：检查所有已配置平台
- `--platform <name>`：只检查指定平台

> `--platform` 与 `--all-platforms` 互斥。

## 检查范围

### 全局检查

- CCR root 与 registry 路径是否存在且可读
- 当前平台是否可解析且为已实现平台
- 已配置平台列表是否可解析
- 本地冲突扫描（复用 `ccr check conflicts` 的事实源）

### 平台深度检查

- `profiles.toml` 是否存在且可读
- current profile 是否能解析到真实 profile
- 当前 profile 是否通过本地字段校验
- 平台 settings/config 文件路径是否可解析
- 平台 settings/config 文件是否存在、可读、可校验
- 平台运行态认证是否健康

### 在线检查

当启用 `--online` 时：

- 只对当前目标平台的 **current profile** 做在线探活
- 不会扫描所有 profile
- 对无法在线探测的 profile 会标记为 `SKIP`

## 输出格式

默认输出为逐项体检结果，状态包括：

- `[OK]`
- `[WARN]`
- `[FAIL]`
- `[SKIP]`

结尾会汇总：

```text
Results: X passed, Y warnings, Z failed, K skipped
```

## 退出码

- 有 `failed` 项时返回非 0
- 只有 `warning` 时仍返回 0

## JSON 输出

`--json` 时输出结构固定为：

- `scope`
- `online`
- `summary`
- `checks`

每个 `check` 至少包含：

- `id`
- `status`
- `summary`

并按需附带：

- `path`
- `detail`
- `recommendation`

## 与其他诊断命令的关系

- `ccr doctor`：统一总览入口，先跑它最适合
- [`ccr validate`](./validate)：专注配置与 settings 的静态校验
- [`ccr provider`](./provider)：专注 Provider 健康检查与连通性
- [`ccr check conflicts`](./check)：专注本地冲突扫描

## 示例

```bash
# 默认：全局 + 当前平台，本地只读检查
ccr doctor

# 对 Codex 做完整本地体检
ccr doctor --platform codex --verbose

# 检查所有已配置平台，并启用在线探活
ccr doctor --all-platforms --online

# 在 CI 中消费 JSON
ccr doctor --json
```
