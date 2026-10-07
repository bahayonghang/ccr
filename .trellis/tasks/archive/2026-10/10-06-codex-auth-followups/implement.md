# 父任务实施计划

1. 保存实施前 Git 基线与现有未提交规划；记录本轮实施授权。
2. 审阅并启动 P2，dispatch Trellis implement，随后独立 Trellis check；记录 AC1–AC7。
3. 根据用户答复收敛 P1 缺失身份规则，补齐设计、实施步骤、上下文；启动、实施、独立检查。
4. 根据用户答复收敛 P3 保留数量与删除前备份；启动、实施、独立检查。
5. 补齐 P4 设计，实施 SID 权限、Unchanged 权限加固、原子 rename 回退、Debug 脱敏与响应体移除；独立检查。
6. P5 交付文件命名评估；没有迁移决定时不改快照布局。
7. P6 按用户选择实施导入身份校验，并交付默认加密导出评估；独立检查。
8. 执行最终 `just version-check`、`just fmt-check`、`just lint-strict`、`just test` 与跨 crate `just ci`。已通过且源码未变化的正式命令不重复运行。
9. 写回相关 spec 和各子任务/父任务验收记录。未经另外授权不提交、归档、推送、创建 PR 或发布。
