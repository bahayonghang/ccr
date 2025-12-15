# ui - Launch CCR UI

Start the full CCR UI (frontend: Vue3+Vite+Pinia+Tauri; backend: Axum). The command auto-detects or downloads the UI bundle.

**Version**: v1.4.0+

## Usage

```bash
ccr ui [-p <frontend-port>] [--backend-port <port>]
```

- `-p, --port`: Frontend port, default `3000`
- `--backend-port`: Backend port, default `38081`

## Startup Order

1) `./ccr-ui/` or `../ccr-ui/` (dev source)  
2) `~/.ccr/ccr-ui/` (user cache)  
3) Prompted GitHub download (first-time use)  

## Examples

```bash
ccr ui                             # auto-detect/download, ports 3000/38081
ccr ui -p 3100 --backend-port 39000
```

> For a lightweight API/compatibility server, use `ccr web`. `ccr ui` provides the full graphical experience with command execution, sync, stats, etc.
