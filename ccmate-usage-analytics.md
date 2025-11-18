# CCMate Claude Code Usage 统计与绘图实现说明

> 本文基于 `djyde/ccmate` 仓库源码分析，总结其如何统计 Claude Code 的 token 使用（usage）、如何读取与过滤数据，以及前端如何进行可视化绘制。可作为在 `ccr-ui` 中设计类似 usage 页面时的参考。

---

## 1. 整体架构概览

CCMate 是一个基于 Tauri + React 的桌面应用，用来管理 Claude Code 配置，并提供一个 **本地 usage 分析面板**。整体数据流大致为：

1. **Claude Code 扩展** 持续将会话与调用信息写入本地日志（`~/.claude/projects/**/*.jsonl`）。
2. **CCMate 后端（Tauri / Rust）** 通过命令 `read_project_usage_files`：
   - 递归扫描上述目录；
   - 解析每条 JSON 行，提取 `uuid`、`timestamp`、`model`、`usage`（token 统计）；
   - 过滤掉无效或 usage 为 0 的记录；
   - 返回结构化的 `ProjectUsageRecord[]` 给前端。
3. **CCMate 前端（React / TypeScript）** 使用 React Query：
   - 通过 `invoke("read_project_usage_files")` 获取 usage 记录列表；
   - 在 `UsagePage` 中汇总出总输入/输出 token 等指标；
   - 将原始记录传入 `TokenUsageChart`，按时间区间和模型做过滤与聚合；
   - 最终通过 `AreaChart` 组件展示多条时间序列曲线（输入 / 输出 / Cache Read Tokens）。

需要注意的是：**CCMate 并不自己计算 usage**，而是**消费 Claude Code 已经记录好的 usage 字段**，本质上是一个“日志聚合 + 可视化”的角色。

---

## 2. 数据来源与存储位置

### 2.1 物理存储位置

- 日志目录：`$HOME/.claude/projects`（即 `~/.claude/projects`）
- 文件模式：`**/*.jsonl`
- 每个 `.jsonl` 文件包含多行 JSON，每一行是一条 usage 相关的事件或会话记录。

### 2.2 单条记录的典型结构

经 `read_project_usage_files` 源码分析，一行 JSON 中可能包含以下字段：

- 顶层字段：
  - `uuid: string`
  - `timestamp: string`（ISO 时间）
  - `model?: string`
  - `usage?: { input_tokens, cache_read_input_tokens, output_tokens }`
- 或嵌套在 `message` 字段下：
  - `message.model?: string`
  - `message.usage?: { ... }`

CCMate 在解析时会同时检查：

- 顶层 `json["model"]` 与 `json["usage"]`；
- `json["message"]["model"]` 与 `json["message"]["usage"]`。

因此，只要 Claude Code 的日志在 **顶层或 message 中** 存储了 `model` 和 `usage`，CCMate 就可以兼容。

### 2.3 usage 字段定义

后端 Rust 中的 usage 数据结构如下（`src-tauri/src/commands.rs`）：

```rust
pub struct UsageData {
    pub input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

pub struct ProjectUsageRecord {
    pub uuid: String,
    pub timestamp: String,
    pub model: Option<String>,
    pub usage: Option<UsageData>,
}
```

前端 TypeScript 对应结构（`src/lib/query.ts`）：

```ts
export interface UsageData {
  input_tokens?: number
  cache_read_input_tokens?: number
  output_tokens?: number
}

export interface ProjectUsageRecord {
  uuid: string
  timestamp: string
  model?: string
  usage?: UsageData
}
```

可以看到，前后端的数据模型一一对应，只是将 Rust 中的 `Option` 映射为 TypeScript 中的可选字段。

---

## 3. 后端：usage 统计的读取与过滤逻辑

### 3.1 与 usage 相关的关键命令

后端 usage 流程的核心在 `src-tauri/src/commands.rs` 中的：

- `read_project_usage_files`：扫描并解析 `~/.claude/projects` 下的 `.jsonl` 文件，输出 `Vec<ProjectUsageRecord>`。

除此之外，还有两个容易混淆但用途不同的模块：

- `hook_server.rs`：
  - 提供本地 HTTP 端口 `/claude_code/hooks`，
  - 主要用来接收 Claude Code 的 Hook 事件并触发系统通知（如 Task 完成、工具使用等），
  - **不负责 usage 统计**。
- `track` 命令 + 前端 `src/lib/tracker.ts`：
  - 负责将事件上报到 PostHog（产品分析），
  - 事件如 `app_launched` 等，
  - **与 token usage 展示无直接关系**。

### 3.2 read_project_usage_files：读取 usage 的完整流程

函数签名（简化）：

```rust
#[tauri::command]
pub async fn read_project_usage_files() -> Result<Vec<ProjectUsageRecord>, String>
```

关键步骤如下：

1. **定位日志根目录**：

   ```rust
   let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
   let projects_dir = home_dir.join(".claude/projects");
   ```

2. **递归查找所有 `.jsonl` 文件**：

   - 使用一个内部递归函数 `find_jsonl_files(dir, files)`：
     - 遍历 `dir` 下的所有子项；
     - 若为文件且扩展名为 `jsonl`，加入列表；
     - 若为目录则递归调用自身。

3. **逐文件、逐行读取内容**：

   ```rust
   for path in jsonl_files {
       let content = std::fs::read_to_string(&path)?;
       for line in content.lines() {
           if line.trim().is_empty() { continue; }
           let json_value: Value = serde_json::from_str(line)?;
           // ... 提取字段
       }
   }
   ```

4. **提取核心字段**：

   - `uuid`：从 `json["uuid"]` 中读取字符串；
   - `timestamp`：从 `json["timestamp"]` 中读取字符串；
   - `model`：优先从顶层 `json["model"]` 读取，其次从 `json["message"]["model"]` 读取；
   - `usage`：优先从顶层 `json["usage"]` 读取，其次从 `json["message"]["usage"]` 读取，并转换为 `UsageData`：

     ```rust
     UsageData {
         input_tokens: usage_obj.get("input_tokens").and_then(|v| v.as_u64()),
         cache_read_input_tokens: usage_obj.get("cache_read_input_tokens").and_then(|v| v.as_u64()),
         output_tokens: usage_obj.get("output_tokens").and_then(|v| v.as_u64()),
     }
     ```

5. **过滤条件**：

   仅在下列条件同时满足时才保留记录：

   - `uuid` 非空；
   - `timestamp` 非空；
   - 存在 `usage`，且：
     - `input_tokens.unwrap_or(0) + output_tokens.unwrap_or(0) > 0`。

   也就是说，**只保留真正产生了 token 消耗的记录**，避免将 system event 或无效调用计入统计。

6. **返回结果**：

   将所有符合条件的记录以 `Vec<ProjectUsageRecord>` 的形式返回给前端。


### 3.3 设计取舍

- **利用 Claude Code 现有日志**：不会在 CC Mate 内重复存储或冗余落盘，而是每次渲染 Usage 页面时现读现算；
- **兼容不同 schema 位置**：同时检查顶层和 `message` 中的 `model` / `usage`，提高对日志格式变动的鲁棒性；
- **过滤无用记录**：统一在后端过滤 `token == 0` 的记录，减少前端压力，也避免图表被噪音污染。

这些模式对于 ccr 未来的 usage 设计也具有参考价值：尽量将 **清洗 / 过滤** 放在单一后端入口完成，前端只消费干净数据。

---

## 4. 前端：usage 数据模型与查询

前端的 usage 数据访问集中在 `src/lib/query.ts` 中，使用 `@tanstack/react-query` 与 Tauri 的 `invoke` 结合：

```ts
import { useQuery } from "@tanstack/react-query"
import { invoke } from "@tauri-apps/api/core"

export interface UsageData {
  input_tokens?: number
  cache_read_input_tokens?: number
  output_tokens?: number
}

export interface ProjectUsageRecord {
  uuid: string
  timestamp: string
  model?: string
  usage?: UsageData
}

export const useProjectUsageFiles = () => {
  return useQuery({
    queryKey: ["project-usage-files"],
    queryFn: () => invoke<ProjectUsageRecord[]>("read_project_usage_files"),
  })
}
```

说明：

- `useProjectUsageFiles` 是前端唯一的 usage 数据入口；
- `queryKey` 为 `"project-usage-files"`，便于缓存与刷新；
- 依赖 Tauri 的 `invoke` 调用同名命令 `read_project_usage_files`；
- 调用方只需要处理 loading / error / data 三种状态。

对于 ccr-ui 来说，可以直接借鉴这一模式：

- 定义统一的 `useXxxStats` Hook，内部封装对后端 `/api/stats/...` 的访问；
- 页面组件只关心 hook 返回的 `data` 等，逻辑清晰、利于测试。

---

## 5. Usage 页面整体结构

Usage 页面组件位于 `src/pages/UsagePage.tsx`，职责是：

- 调用 `useProjectUsageFiles` 拉取 usage 记录；
- 显示加载 / 错误 / 空状态；
- 将数据传给 `ActivityGrid`（活动分布）和 `TokenUsageChart`（时间序列图）；
- 计算并展示输入 / 输出 / Cache Read Tokens 三种 token 的汇总值。

关键片段（简化）：

```tsx
export function UsagePage() {
  const { t } = useTranslation()
  const {
    data: usageData,
    isLoading,
    error,
    refetch,
    isRefetching,
  } = useProjectUsageFiles()

  const [filteredUsageData, setFilteredUsageData] = useState<ProjectUsageRecord[]>([])

  useEffect(() => {
    if (usageData) {
      setFilteredUsageData(usageData)
    }
  }, [usageData])

  // ... 渲染头部、刷新按钮、加载/错误态等

  // 有数据时：
  return (
    <>
      <ActivityGrid data={usageData} />

      {/* 三个汇总卡片：分别统计 input/output/cache_read_input_tokens 之和 */}
      {/* 每个卡片都基于 filteredUsageData 做 reduce，确保与图表筛选联动 */}

      <TokenUsageChart
        data={usageData}
        onFilteredDataChange={setFilteredUsageData}
      />
    </>
  )
}
```

要点：

- 页面内部维护了一个 `filteredUsageData`：
  - 初始为完整的 `usageData`；
  - 当图表内部根据时间/模型过滤后，会通过 `onFilteredDataChange` 将最新过滤结果回传；
  - 顶部 summary 卡片按照 `filteredUsageData` 进行 reduce，可实现 **“卡片与图表同步筛选”** 的体验。

这一模式对于构建 ccr 的 usage 页面也非常适合：

- 图表控制过滤条件，并返回“当前视图”的数据；
- 上方数值卡片始终与图表视图一致，避免用户困惑。

---

## 6. TokenUsageChart：时间范围与模型维度的可视化

`TokenUsageChart` 是 Usage 页面的核心图表组件（`src/components/TokenUsageChart.tsx`），主要职责：

1. 基于原始 `ProjectUsageRecord[]`：
   - 进行 **模型维度过滤**；
   - 进行 **时间范围过滤**；
2. 将过滤后的记录按时间区间聚合，生成适合绘制面积图的数据；
3. 支持 legend 交互：控制显示 `Input Tokens` / `Output Tokens` / `Cache Read Tokens` 三条线；
4. 使用 `AreaChart` + Tremor 风格的颜色和渐变进行展示。

### 6.1 组件状态与属性

```ts
interface TokenUsageChartProps {
  data: ProjectUsageRecord[]
  onFilteredDataChange?: (filtered: ProjectUsageRecord[]) => void
}

type TimeRange = "5h" | "today" | "7d" | "week" | "month" | "all"

const [selectedModel, setSelectedModel] = useState<string>("all")
const [timeRange, setTimeRange] = useState<TimeRange>("5h")
const [activeCategories, setActiveCategories] = useState<string[]>([
  "Input Tokens",
  "Output Tokens",
])
```

- `selectedModel`：当前选中的模型（或 `"all"`）；
- `timeRange`：当前选中的时间窗口：
  - `5h`：最近 5 小时；
  - `today`：从当天 0 点起；
  - `7d`：最近 7 天；
  - `week`：当前周（从周日开始）；
  - `month`：当前月；
  - `all`：全时间范围（从最早记录开始）；
- `activeCategories`：当前显示的 token 类型（Input / Output / Cache），可通过 legend 开关。

### 6.2 过滤逻辑（模型 + 时间范围）

过滤逻辑通过 `useMemo` 实现：

```ts
const filteredData = useMemo(() => {
  let filtered = data

  // 模型过滤
  if (selectedModel !== "all") {
    filtered = filtered.filter((r) => r.model === selectedModel)
  }

  // 时间范围过滤
  const now = dayjs()
  let startTime: dayjs.Dayjs

  switch (timeRange) {
    case "5h":   startTime = now.subtract(5, "hour"); break
    case "today": startTime = now.startOf("day"); break
    case "7d":    startTime = now.subtract(6, "day"); break
    case "week":  startTime = now.day(0); break // 周日
    case "month": startTime = now.startOf("month"); break
    case "all":
      if (filtered.length > 0) {
        const earliestTime = dayjs(Math.min(
          ...filtered.map((r) => dayjs(r.timestamp).valueOf()),
        ))
        startTime = earliestTime
      } else {
        startTime = dayjs(0)
      }
      break
    default:
      startTime = now.subtract(5, "hour")
  }

  filtered = filtered.filter((r) =>
    dayjs(r.timestamp).isAfter(startTime.subtract(1, "millisecond")),
  )

  return filtered
}, [data, selectedModel, timeRange])
```

随后通过 `useEffect` 将 `filteredData` 通过 `onFilteredDataChange` 回传给父组件：

```ts
useEffect(() => {
  if (onFilteredDataChange) {
    onFilteredDataChange(filteredData)
  }
}, [filteredData, onFilteredDataChange])
```

### 6.3 时间区间聚合：构建图表数据

组件内部定义了一个 `groupDataByInterval(records)` 函数，用于根据不同时间范围将多条原始记录聚合为离散的时间点：

- 输出结构类似：

  ```ts
  type IntervalAcc = { input: number; output: number; cache: number }
  const intervals: { [timeKey: string]: IntervalAcc } = {}
  ```

- 不同时间范围对应不同的分组粒度：
  - `all`：按“周”为单位分组；
  - 其它范围：通常按天或小时分组（实现上基于 `dayjs` 生成 key）。

以 `all` 范围为例的关键逻辑（简化）：

```ts
if (timeRange === "all") {
  const earliestTime = ... // 最早记录的时间
  let currentWeekStart = earliestTime.day(0)
  const nowWeekStart = now.day(0)

  // 预生成从最早周到当前周的所有周起始 key
  while (currentWeekStart.isBefore(nowWeekStart) || currentWeekStart.isSame(nowWeekStart)) {
    intervals[currentWeekStart.valueOf()] = { input: 0, output: 0, cache: 0 }
    currentWeekStart = currentWeekStart.add(1, "week")
  }

  // 将记录聚合到对应周
  records.forEach((record) => {
    const recordTime = dayjs(record.timestamp)
    const weekStart = recordTime.day(0)
    const weekKey = weekStart.valueOf()

    if (intervals[weekKey]) {
      intervals[weekKey].input += record.usage?.input_tokens || 0
      intervals[weekKey].output += record.usage?.output_tokens || 0
      intervals[weekKey].cache += record.usage?.cache_read_input_tokens || 0
    }
  })
}
```

最终将 `intervals` 转换成 `chartData` 数组，每个元素形如：

```ts
{
  time: string,             // x 轴标签
  "Input Tokens": number,  // y1
  "Output Tokens": number, // y2
  "Cache Read Tokens": number, // y3
}
```

### 6.4 图表渲染与颜色配置

渲染使用的是 `AreaChart` 组件（`@/components/ui/area-chart`），并结合 `chartUtils.ts` 中的颜色工具：

```tsx
<AreaChart
  data={chartData}
  index="time"
  categories={activeCategories}
  colors={activeCategories.map((cat) => {
    if (cat === "Input Tokens") return "blue"
    if (cat === "Output Tokens") return "emerald"
    if (cat === "Cache Read Tokens") return "amber"
    return "blue"
  })}
  valueFormatter={formatLargeNumber}
  fill="gradient"
  className="h-full"
  showLegend={false}
/>
```

颜色字符串会由 `src/lib/chartUtils.ts` 映射到具体的 Tailwind 类名，例如：

```ts
export const chartColors = {
  blue: {
    bg: "bg-blue-500",
    stroke: "stroke-blue-500",
    fill: "fill-blue-500",
    text: "text-blue-500",
  },
  emerald: { ... },
  amber: { ... },
  // ...
}
```

并且提供：

- `constructCategoryColors(categories, colors)`：为多个类别循环分配颜色；
- `getColorClassName(color, type)`：根据颜色 key 和用途（背景 / 线条 / 填充 / 文本）选出具体类名。

---

## 7. 额外：PostHog 事件埋点（与 usage 展示解耦）

项目中还存在一个 `src/lib/tracker.ts` + 后端 `track` 命令的组合，用于将产品层面的事件（例如 App 启动）上报到 PostHog：

- 前端：

  ```ts
  export enum TrackEvent {
    AppLaunched = "app_launched",
  }

  export const track = async (event: string, properties: Record<string, any> = {}) => {
    if (!import.meta.env.PROD) return
    await invoke<void>("track", { event, properties })
  }
  ```

- 后端：

  ```rust
  #[tauri::command]
  pub async fn track(event: String, properties: serde_json::Value, app: tauri::AppHandle) -> Result<(), String> {
      // 组装 distinct_id、app_version、$os、$os_version 等
      // POST 到 https://us.i.posthog.com/capture/
  }
  ```

这一链路提供了 **远程产品分析** 能力，但与本地 token usage 可视化是完全解耦的；

- token usage 可视化只依赖本地 `~/.claude/projects` 日志；
- PostHog 埋点则面向开发者自己做产品决策。

在 ccr 设计中也可以采用类似分层：

- 一条链路专门负责本地统计与可视化（面向终端用户）；
- 另一条链路可选地汇总匿名统计（面向产品/开发）。

---

## 8. 对 ccr-UI 设计 usage 页面的启发

结合上面的分析，如果要在 `ccr-ui` 中设计“过去一段时间的 usage 统计页面”，可以借鉴 CCMate 的几个关键模式：

1. **数据来源模式**：
   - CCMate 依赖 Claude Code 的本地日志；
   - ccr 可以依赖自身 CLI（如 `ccr stats cost` / `ccr stats usage`）导出的 JSON，或直接读取内部账单 / 事件存储；
   - 统一在服务端（Axum 后端）做数据清洗与聚合，前端消费干净接口。

2. **前端数据访问抽象**：
   - 使用集中定义的 API 客户端与类型（类似 `src/lib/query.ts` + `useProjectUsageFiles`）；
   - 每个页面通过 hook 获取数据，逻辑清晰、易测。

3. **时间范围与维度筛选**：
   - 在前端提供：时间范围（today/7d/month/all）+ 维度（模型 / 项目 / 提供商）筛选；
   - 将筛选逻辑集中在图表组件内部，并通过回调向外暴露“当前过滤后数据”，供 summary 卡片使用。

4. **聚合粒度**：
   - 对于长期数据（如 all），按周聚合可减少点数、提升可读性；
   - 对于短期数据（如 24 小时），可以按小时或更细粒度聚合。

5. **可视化一致性**：
   - 使用统一的颜色映射工具（类似 `chartUtils.ts`），保证所有图表颜色语义一致（输入=蓝、输出=绿、cache=橙等）。

本说明文档可作为在 `ccr-ui` 中实现 usage 统计与绘图能力时的参考基础，后续可以在 OpenSpec 中基于此抽象出更通用的“usage-stats” 能力规范。
