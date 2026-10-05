# 修正 Vue 架构残留文案

父任务：`.trellis/tasks/08-25-react-home-style-redesign`（对应父任务 R9 / AC8）

## Goal

前端已完成 Vue → React 迁移，但仓库文案仍把当前架构描述为 Vue，会误导后续开发与 AI 代理。本子任务只修正架构描述漂移，不做文档重写。

## 前置

无，可与其他子任务并行。

## Requirements

- R1：`ccr-ui/package.json` 的包名 `ccr-ui-frontend-vue` 改为与 React 架构一致的名称。改名前确认无脚本、CI、workspace 配置按该名称引用。
- R2：`ccr-ui/CLAUDE.md` 的技术栈表格与架构树中的 Vue 描述（Vue.js 3.5.22 / Vue Router / Pinia / `<script setup lang="ts">` / `views/` `components/` `composables/` `stores/` 的 Vue 语义）改为实际的 React 19 / React Router / TanStack Query / Zustand / Radix UI 事实。
- R3：`code_map.md` 中把前端描述为 Vue 的段落改为 React。
- R4：只改与架构描述漂移相关的语句。不重写章节结构，不补充新内容，不动 Design Context 等与本次漂移无关的段落。
- R5：改动前逐项核对代码事实，不照抄本 PRD 的版本号——以 `ccr-ui/package.json` 的实际依赖版本为准。

## Acceptance Criteria

- [x] AC1（R1）：`rg -n 'ccr-ui-frontend-vue' .` 无命中（`.git` 与 lockfile 历史除外）；`just frontend-check-quick` 通过，构建与脚本不因改名失败。
- [x] AC2（R1）：改名前的引用核查已执行：`rg -n 'ccr-ui-frontend-vue' --glob '!.git' .` 的每个命中都已确认属于「需要同步改」或「历史记录不动」，逐项记录在提交信息或本任务 `research/`。
- [x] AC3（R2,R3）：`rg -ni 'vue' ccr-ui/CLAUDE.md code_map.md` 的剩余命中仅为迁移历史的显式记述，无把当前架构描述为 Vue 的语句。
- [x] AC4（R4）：`git diff --stat` 的改动文件集合仅含 `ccr-ui/package.json`、`ccr-ui/CLAUDE.md`、`code_map.md`（如 lockfile 因改名变更则一并计入）。
- [x] AC5（R5）：文中出现的每个框架版本号与 `ccr-ui/package.json` 的实际依赖一致，逐个核对而非照抄本 PRD。

## 已知事实

- `ccr-ui/package.json` 当前 `"name": "ccr-ui-frontend-vue"`，`"react": "19.2.8"`。
- `ccr-ui/CLAUDE.md` 的技术栈表格仍写 Vue 3.5.22 / Vue Router / Pinia。
- 本任务不改测试，因此不受父任务 XC5 约束（XC5 只针对改动 UI 行为的子任务）。

## Out of Scope

- 目录重命名（`views/`、`composables/`、`stores/` 保持现名）。
- 根 `CLAUDE.md` 中与 Vue 无关的段落。
- 补写缺失的 React 架构文档。
