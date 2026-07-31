# 设计:auth_mode 一致性与显式 env 所有权

## 所有权模型

`ccr-types::env_keys::CCR_MANAGED_KEYS` 是 ccr 可写、可自动删除的唯一集合,内容必须覆盖 `ConfigSection::to_managed_env_pairs` 所有可能输出的键。现有 `NON_ANTHROPIC_MANAGED_KEYS` 保留为兼容子集,并以测试断言它是总集合的子集。

新增纯数据 API:

- `clear_ccr_managed_vars()`:只删除总集合中的键。
- `has_managed_overrides()`:只检查总集合。
- `managed_env_entries()`:按总集合返回实际存在的键值,供 clear 的预览、计数与执行共用。

`clear_managed_vars()` 改为委托新显式语义,保持已有调用方源码兼容;`clear_anthropic_vars()` 保留其“前缀全清”的历史语义,但不再用于 profile/auth/clear 正常路径。`apply_managed_env()` 先调用显式清理再写新 pairs。

该选择会保留旧版本 ccr 或第三方工具写入、但不在清单中的 `ANTHROPIC_*`。这是避免误删用户配置的必然 tradeoff;后续 doctor 子任务负责告警,不以扩大删除范围兜底。

## auth_mode 单一有效口径

- `resolve_profile_auth_mode` 继续表示字面/存储态,不改函数和测试。
- profile apply、auth switch 清理判断与 runtime summary 全部使用 `effective_auth_mode`。
- `has_anthropic_overrides` 保持现有前缀查询语义;新增调用点使用 `has_managed_overrides`,避免静默改变其他诊断/验证逻辑。

## apply 自愈事务顺序

`ClaudePlatform::apply_profile(name)` 的顺序调整为:

1. 加载 profiles 并克隆目标 profile。
2. 计算 literal/effective;对克隆值执行 normalize 与验证。
3. 若发生纠正,先把纠正值写回 profiles.toml;写失败返回带 profile 名的错误。
4. 重新从纠正后的 profile 构造 section,再读取并修改 settings.json。
5. 保存 settings,更新 current 指针与 registry。

这样可保证 profiles.toml 自愈失败时 runtime settings 尚未修改。自愈成功而后续 settings 写失败时,持久 profile 已变正确,重试是幂等的;这是优于回滚错误存储态的恢复模型。

## clear 展示与执行

`clear` 命令只调用 `managed_env_entries()` 一次得到预览集合,空判断、表格、确认文案、计数与最终删除都以该集合为准。token/key 值继续走掩码;文案改为“CCR 托管的 Claude 环境变量”,不再声称清空所有 `ANTHROPIC_*`。

## 测试策略

- ccr-types:总集合唯一性、NON 子集、显式清理/检测、用户自有键保留、apply 清旧托管键。
- ccr-cli service:错误字面 subscription profile 的清理与 runtime summary。
- ClaudePlatform:自愈落盘、重复 apply 无二次纠正、持久化失败时 settings 字节不变。
- profile_off/clear:预览集合与实际删除集合一致,五个非 Anthropic 托管键覆盖。

## 兼容与回滚

不改 profiles.toml schema、不删除 legacy API、不新增依赖。若显式清单漏键,补清单与映射不变量测试;不得退回前缀全删。
