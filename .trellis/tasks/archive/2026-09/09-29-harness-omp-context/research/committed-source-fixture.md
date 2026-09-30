# T01 已提交源文件夹具

- 提交：30cbeddf730ed01b312c3396ba7b8691853be859
- 提取方式：git -c core.autocrlf=false archive，只取 .gitignore、.omp/extensions/trellis/index.ts、scripts/trellis/omp-context.test.ts。
- 自有临时目录内执行 bun test scripts/trellis/omp-context.test.ts。
- 结果：14 tests / 99 assertions，exit 0。
- 三个输入均逐字节匹配指定提交的 Git blob。
- 初次提取受 core.autocrlf=true 影响，测试开始前的 blob 字节检查失败；改用仅对该命令生效的设置后通过。未更改仓库或全局配置。
- 未注入 ignored 文件、个人目录或已安装 Trellis 状态。
- 验证范围：Bun 契约测试；OMP 原生加载、类型与信任提示仍未验证。

机器记录见 [JSON](committed-source-fixture.json)，原始输出见 [日志](committed-source-fixture.log)。
