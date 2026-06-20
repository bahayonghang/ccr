# Implement — 官方 GLM profile

## 前置

- 确认 `06-20-fable-model-support` 已合入（fable 后端字段 + catalog override 类型已含 `defaultFableModel` 与 `*_modelName`）。

## 步骤 1：仓库内置 GLM 预设（R2）

- [ ] `crates/ccr-checkin/data/providers-catalog.json` 新增 provider 条目（GLM / bigmodel）：
      id、name、description、domain=`open.bigmodel.cn`、websiteUrl、icon、bizCategory、tags；
      `platforms.claude` override：baseUrl、provider=`glm`、providerType=`third_party_model`、
      defaultOpusModel/SonnetModel/HaikuModel/**FableModel**=`glm-5.2[1m]`、四层 `*ModelName`=`GLM-5.2`。
- [ ] 核对 schemaVersion 与 Rust `PROVIDERS_CATALOG_SCHEMA_VERSION` 一致。
- [ ] 验证：`cargo test -p ccr-checkin -- --test-threads=1`（builtin_providers 解析）+ `just frontend-check-quick`

## 步骤 2：写入运行时官方 GLM profile（R1）

- [ ] 用 ccr 正常路径创建 profile（优先 CLI；或经 ccr-ui 套用步骤 1 的预设后填占位 token）。
- [ ] 落盘字段核对：base_url / 四层模型 / fable / 四显示名 / provider=glm / provider_type=third_party_model / auth_mode=api_key / auth_token=占位符。
- [ ] 决策 R1.3：非模型 env 默认不写入；若用户需要，单独说明经 settings 直写。
- [ ] 验证：apply 后读 `~/.claude/settings.json` 比对截图（除 token）。

## 验证命令

```
just lint-strict
just test
just frontend-check-quick
```

## Review Gate

- catalog/Rust 改动触发 `rust-security-reviewer` 必要性评估（无 credential 入库则较轻）。
- 确认占位 api key 非真实可用值。

## Rollback Point

- 步骤 1（catalog）与步骤 2（运行时 profile）相互独立，可分别回滚。
- 回滚运行时 profile：从 `~/.ccr/platforms/claude/profiles.toml` 移除该节并切回原 profile。
