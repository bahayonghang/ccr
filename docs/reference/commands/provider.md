# provider - Provider 健康检查

`ccr provider` 用于验证当前配置中的 Provider 是否可达，以及 API Key 是否有效。

如果你想先拿到一份统一诊断总览，请优先使用 [`ccr doctor`](./doctor)；`provider` 适合在你已经定位到“就是 Provider/连通性问题”之后做定向检查。

## 用法

```bash
ccr provider test <name> [--verbose]
ccr provider test --all [--verbose]
ccr provider verify <name>
```

## 子命令

### test

- `ccr provider test <name>`：测试单个配置
- `ccr provider test --all`：批量测试所有配置
- `--verbose`：显示更多模型信息

输出字段通常包括：

- 状态（Healthy / Degraded / Unhealthy / Unknown）
- Base URL
- 延迟
- 错误信息

### verify

```bash
ccr provider verify <name>
```

验证指定配置的 API Key 是否可用。

## 示例

```bash
ccr provider test work --verbose
ccr provider test --all
ccr provider verify work
```

## 适用场景

- 新建 profile 后做连通性验收
- 批量体检所有当前配置
- 排查 model、token、base URL 相关问题

## 与其他诊断命令的边界

- [`doctor`](./doctor)：统一体检入口；默认本地只读，`--online` 时才追加在线探活
- [`validate`](./validate)：静态校验配置和 settings
- [`check`](./check)：只做本地冲突扫描

## 注意

- 这是 CLI 诊断命令，不对应单独的 `provider-health` Web API。
- 如果需要浏览器式总览，请使用 `ccr ui` 中的 Provider Health 页面；其事实源仍然是同一套后端能力。
