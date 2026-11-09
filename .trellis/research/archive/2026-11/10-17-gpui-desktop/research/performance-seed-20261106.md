# S0/S1/S2 成对合成数据收据

状态：**seed生成与领域投影核对通过；未加载原生应用；不可比较**。
本批没有运行UI、CLI导入、联网、WSL/SSH或修改HOME/USERPROFILE。

## 实现与边界

`crates/ccr-app/src/benchmark_fixture.rs`为`cfg(test)`模块，由lib中的单行测试声明
接入。复用现有ccr-usage test-fixtures dev依赖、SHA256及领域服务，没有新增
生产依赖或在Tauri/native adapter写另一套SQL。

显式ignored入口：

```powershell
$env:CCR_BENCHMARK_FIXTURE_ROOT='D:\Documents\Code\Github\ccr\outputs\desktop\perf\fixtures\<全新目录>'
cargo test -p ccr-app --locked --offline --lib benchmark_fixture::generate_s0_s1_s2_pairs -- --ignored --exact --nocapture --skip export_bindings
```

只接受全新绝对目录；canonical现存父目录必须在`outputs/desktop/perf/fixtures`
内，拒绝链接/reparse和无法读取的祖先元信息。已有目录不会覆盖或自动清理。
目录名不是运行模式开关；future native benchmark身份仍需显式启动参数和captured
source分支，不改已有Fixture布局。

- 两端为`S*/tauri/`与`S*/gpui/`，各自有独立main/archive/catalog/llmusage/home
  文件与root内WebView目录。Tauri marker包含固定version/id。
- Profiles通过ConfigManager一次完整save，再用snapshot读取；生成100个(S1)或
  10,000个(S2)Claude合成profile，10% disabled，current为空。S0保持新fixture的
  missing profile source，不把不存在的catalog伪装成已初始化空catalog。
- 10,000个Codex来源文件按真实provider格式生成，通过捕获的registry与共享refresh
  解析/归档；所有summary分页读回，并读取一个真实provider detail核对正文。
- 100,000个usage事件及240小时分桶通过ccr-usage fixture owner写入，再由共享read
  dashboard读取；owner分页读回全部事件计算hash，实际tokens为1,900,000。
- main/archive schema通过现有迁移初始化，继续保持不同数据库。没有直接插入
  伪造的session summary或略过provider解析以加速。
- `as_of_utc=2026-11-06T12:00:00Z`与`report_timezone=UTC`写入每个manifest，事件
  分布在截止前240小时。未来双方adapter/查询必须显式使用同一定义；当前UI还没有
  完成benchmark clock注入，不能拿wall-clock窗口不一致的数据比较。

## 实际运行

生成到忽略目录：
`outputs/desktop/perf/fixtures/paired-20261106-v1/manifest.json`。
实际命令退出0，**1 passed**，耗时99.17s；这是数据工程耗时，不是应用性能样本。
随后`cargo clippy -p ccr-app --locked --offline --all-targets --all-features -- -D warnings`
退出0。生成后仅把祖先metadata读取错误收紧为拒绝，未改变数据/投影逻辑。

独立check随后发现allowed目录在校验链接祖先之前已`create_dir_all`。已将检查前移，
保留创建后的复核；owned临时junction+不存在后缀的常规回归真实红→绿（1/1），
新的严格clippy退出0。该guard-only修复没有重跑99秒生成器，也未修改已有paired产物；
旧生成收据不冒充已经覆盖这个顺序回归。详见`performance-seed-check-20261106.md`。

| 场景 | 每端Claude profiles | 每端Codex sessions | 每端usage events | 两端logical SHA256 |
| --- | ---: | ---: | ---: | --- |
| S0 | 0（missing source） | 0 | 0 | `871d144280daf2c4e15a95ac4bf1feec5ef7d3a1b289aa9eddbe9fa14905915b` |
| S1 | 100 | 10,000 | 100,000 | `ea49feb9e89f60efccfe476a918797fe6e73e0fa9c33b309155f9fe17120054c` |
| S2 | 10,000 | 10,000 | 100,000 | `ed941d8cffa5c0fde92cc2d9ff2d89a283be7a3de69a8bc136f9eab85af9d63e` |

各场景两端真实投影hash相等；非空场景原opaque source身份hash**不相等**，证明
领域identity仍分别绑定真实物理根。原source/opaque ID未重写。可比hash明确排除
物理根绑定的archive ID，并单独记录身份hash；profiles只对非秘密显示字段做hash，
usage对全部owner-paged事件做hash，session对源内容和公开summary字段做hash。
SQLite本体另有每端artifact SHA256，不要求二进制字节相同。

S2只保存`events_per_second=1000`、`duration_seconds=60`、`state=not_emitted`的
workload descriptor。没有发出流日志，没有预填60,000行后冒称流量通过。

## 仍需实施

- native显式benchmark source统一捕获这些根，Tauri剩余业务图按需要接合；两端
  实际加载、fixture标识和cache/storage原生隔离收据。
- 共同as_of/时区注入、真实S2受控runtime事件源与消费、进程树归属。
- 真正process-create/first-present/action-ready观测、release构建与合同试次。

manifest始终标`performance=not_run`、`comparison=not_comparable`。
不能将seed成功、同逻辑hash或完整headless gate转换成性能PASS。
