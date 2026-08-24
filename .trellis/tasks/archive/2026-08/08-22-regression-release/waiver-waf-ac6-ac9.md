# 跳过记录：父 AC9 / 子 AC6 WAF 真实签到

日期：2026-08-25  
范围：父任务 `08-22-react-migration` AC9 合取项中的 **WAF WebView bypass 真实签到**；子任务 `08-22-regression-release` AC6。

## 已测

- WAF event-wait / runtime-coverage smoke 通过。
- Check-in OAuth 向导走到凭据录入步。
- 打包启动、CSP 未放宽、窗口 chrome 六项、杀进程后再启动已测。

## 未做

未完成一次真实签到。凭据未提供。本会话不能代填账号、Cookie 或付费服务。

## 本文件不是关闭授权

本文件只记录跳过原因。它不能把父 AC9 或子 AC6 标为通过，也不能关闭发布门。

要闭合这两条，需要：可用的 WAF/OAuth 签到账号，或对「父 AC9 可在无真实签到的情况下关闭」的明确授权。
