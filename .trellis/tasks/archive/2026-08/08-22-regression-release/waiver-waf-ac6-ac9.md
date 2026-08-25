# 用户授权：父 AC9 可在无真实签到的情况下关闭

日期：2026-08-25  
授权人：任务提出者（会话原文）

原文：

> 不用验证签到，这个功能没用了，可以删掉，授权：父 AC9 可以在没有真实签到的情况下关闭。

## 授权范围

- 父任务 `08-22-react-migration` **AC9**：WAF WebView bypass 真实签到不再作为合取必要条件。打包启动、CSP、窗口 chrome、杀进程后再启动已测，本条可关闭。
- 子任务 `08-22-regression-release` **AC6**：不做真实签到验证。
- 本迁移任务**不删除** Check-in 产品代码、路由或 `crates/ccr-checkin`。原目标冻结 75 条路由、禁止 `crates/` 功能改动。删除签到功能另开任务。

## 已测（非签到）

- WAF event-wait / runtime-coverage smoke
- OAuth 向导止于凭据步
- 打包启动截图、CSP 未放宽、chrome 六项、杀进程后再启动

## 未做

未完成一次真实签到。按上述授权不再验证。
