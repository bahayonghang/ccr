# 配置表单字段清单（AC4）

校验经 zod + react-hook-form。草稿键：新增 `__new__`，编辑为配置 id。

## Add / Edit 配置

| 名称 | 类型 | 默认值 | 校验 | 读写 |
| --- | --- | --- | --- | --- |
| name | string | `''` | 新增必填 trim min 1；编辑只读 id | 读写 |
| description | string | `''` | 无 | 读写 |
| base_url | string | `''` | 新增必填 trim min 1 | 读写 |
| auth_token | string | `''` | 新增必填 trim min 1；界面 password 掩码 | 读写，不入日志 |
| model | string | `''` | 无 | 读写 |
| small_fast_model | string | `''` | 无 | 读写 |
| provider_type | string | `''` | `''` / `official_relay` / `third_party_model` | 读写 |
| provider | string | `''` | 无 | 读写 |
| account | string | `''` | 无 | 读写 |
| tagsInput | string | `''` | 逗号分隔，提交时拆成 `tags: string[]` | 读写 |

模板应用只填非密钥字段（`base_url` / model / provider / description），不改 `auth_token`。

## AppSettingsView

| 名称 | 类型 | 默认值 | 校验 | 读写 |
| --- | --- | --- | --- | --- |
| theme | `'light' \| 'dark' \| 'system'` | 存储值 / system | zod enum | 即时写入 |
| flavor | `'neutral' \| 'clay'` | 存储值 / neutral | zod enum | 即时写入 |
| locale | `'zh-CN' \| 'en-US'` | 存储值 | zod enum | 即时写入 |
| uiFont | string | `''`（系统默认） | 净化后写入 | 即时写入 |
| codeFont | string | `''` | 净化后写入 | 即时写入 |
| confirmBeforeExit | boolean | 运行时偏好 / true | boolean | 即时写入 shell API |
| closeToTray | boolean | false | boolean | 即时写入 shell API |
| openPanelOnTrayClick | boolean | true | boolean | 即时写入 shell API |
| sidebarWidth | number | 240 | 200–480 step 8 | 即时写入 |
| perfTelemetryEnabled | boolean | localStorage | boolean | 即时写入 |

## ConverterView

| 名称 | 类型 | 默认值 | 校验 | 读写 |
| --- | --- | --- | --- | --- |
| sourceFormat | CliType | `claude-code` | 与 target 不得相同 | 读写 |
| targetFormat | CliType | `codex` | 与 source 不得相同 | 读写 |
| convertMcp | boolean | true | 无 | 读写 |
| convertCommands | boolean | true | 无 | 读写 |
| convertAgents | boolean | true | 无 | 读写 |
| configData | string | `''` | 转换前非空 | 读写 |

## Provider 自定义模板

| 名称 | 类型 | 默认值 | 校验 | 读写 |
| --- | --- | --- | --- | --- |
| name | string | `''` | trim min 1 | 读写 |
| id | string | slug | 无 | 读写 |
| category | enum | `third_party` | 五档分类 | 读写 |
| websiteUrl / apiKeyUrl | string | `''` | 无密钥 | 读写 |
| platforms.* | boolean | 当前平台 | 至少一平台 | 读写 |
| override JSON | string | `'{}'` | 必须是对象；敏感键由 utils 剥离 | 读写 |
| baseUrls / models / aliases / tags | 换行列表 | `''` | parseListInput | 读写 |
