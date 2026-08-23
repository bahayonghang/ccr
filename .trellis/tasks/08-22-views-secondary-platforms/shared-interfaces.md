# 留守 generic 接口公示

## SystemPromptsView

```ts
interface SystemPromptsViewProps {
  config: SystemPromptsConfig
  t?: TranslateFunction
}

interface SystemPromptsConfig {
  platform: 'claude' | 'codex' | 'gemini' | 'opencode'
  module: string
  features: {
    hierarchyNote?: boolean
    geminiNote?: boolean
    showRules?: boolean
    limitHint?: boolean
  }
}
```

状态由视图持有（list / selected / dirty / conflict）。无 slot。

## AgentDetailView

无 props。路由参数 `name`。编辑弹层内部 `useForm`。
