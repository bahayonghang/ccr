# Windows 命令参考

## PowerShell

```powershell
# 方法 1：临时设置环境变量
$env:RUST_LOG="debug"; npm run tauri dev

# 方法 2：分两步
$env:RUST_LOG="debug"
npm run tauri dev
```

## CMD

```cmd
set RUST_LOG=debug && npm run tauri dev
```

## Git Bash

```bash
RUST_LOG=debug npm run tauri dev
```

---

**推荐使用 PowerShell 方法 1**（一行命令）：

```powershell
$env:RUST_LOG="debug"; npm run tauri dev
```
