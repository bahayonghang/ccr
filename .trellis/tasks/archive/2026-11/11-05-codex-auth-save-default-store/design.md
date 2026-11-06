# Design

纠正共享 CredentialStoreKind 的缺省值为官方 file；显式不支持/未知值仍保守拒绝。认证服务区分缺少配置与读取/解析失败，不允许 as_str() 将错误类型折叠成缺省 file。get_auth_state 不改变外部 DTO 形状，失败产生明确 Invalid/Unsupported 状态与脱敏原因；写入口必须保留同等保护。

CLI save 消费认证状态并通过现有 CcrError/顶层错误渲染返回非零，不添加错误枚举或进程退出调用。按项目已有模式保留重复账号覆盖提示，错误只输出一次。保存仍交给已有 save_current_content/registry/secret writer，不重写凭据复制流程。

检查共享 from_config_value 调用方和非字符串值的 as_str() 入口，限制默认值改变造成的误放行。仅修复此契约所需路径；如需大范围公共 API 改造先反馈主会话。

安全边界：文件存在不证明显式 auto 的有效来源，仍拒绝；只用合成凭据验收，不访问系统钥匙串或输出 token。官方 CLI 的完整 layered config 解析不在本次范围。

回滚只针对本任务 diff，绝不重置其他工作树内容。
