# 版本同步脚本（PowerShell 版本）
# 以根 Cargo.toml 为主，同步到：
# - ccr-ui/backend/Cargo.toml
# - ccr-ui/frontend/package.json
# - ccr-ui/frontend/src-tauri/Cargo.toml
# - ccr-ui/frontend/src-tauri/tauri.conf.json

param(
    [switch]$Check,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

# 获取脚本根目录
$ROOT_DIR = Split-Path -Parent (Split-Path -Parent $PSCommandPath)

# 配置文件路径
$ROOT_CARGO = Join-Path $ROOT_DIR "Cargo.toml"
$BACKEND_CARGO = Join-Path $ROOT_DIR "ccr-ui\backend\Cargo.toml"
$FRONTEND_PKG = Join-Path $ROOT_DIR "ccr-ui\frontend\package.json"
$TAURI_CARGO = Join-Path $ROOT_DIR "ccr-ui\frontend\src-tauri\Cargo.toml"
$TAURI_CONF = Join-Path $ROOT_DIR "ccr-ui\frontend\src-tauri\tauri.conf.json"

# 检查文件是否存在
function Test-RequiredFile {
    param([string]$Path)
    if (-not (Test-Path $Path)) {
        Write-Error "❌ 文件不存在: $Path"
        exit 1
    }
}

Test-RequiredFile $ROOT_CARGO
Test-RequiredFile $BACKEND_CARGO
Test-RequiredFile $FRONTEND_PKG
Test-RequiredFile $TAURI_CARGO
Test-RequiredFile $TAURI_CONF

# 从 Cargo.toml 提取 [package] 区块中的 version
function Get-CargoVersion {
    param([string]$Path)
    
    $content = Get-Content $Path -Raw
    $packageBlock = $content -match '(?s)\[package\](.*?)(\[|$)' | Out-Null
    if ($matches) {
        $block = $matches[1]
        if ($block -match 'version\s*=\s*"([^"]+)"') {
            return $matches[1].Trim()
        }
    }
    Write-Error "❌ 无法从 $Path 提取版本号"
    exit 1
}

# 从 JSON 文件提取 version
function Get-JsonVersion {
    param([string]$Path)
    
    $json = Get-Content $Path -Raw | ConvertFrom-Json
    if ($json.version) {
        return $json.version.Trim()
    }
    Write-Error "❌ 无法从 $Path 提取版本号"
    exit 1
}

# 更新 Cargo.toml 的 [package] 区块中的 version
function Set-CargoVersion {
    param(
        [string]$Path,
        [string]$NewVersion
    )
    
    $content = Get-Content $Path -Raw
    $updated = $content -replace '(\[package\](?:(?!\[).)*?version\s*=\s*)"[^"]+"', "`$1`"$NewVersion`""
    Set-Content -Path $Path -Value $updated -NoNewline
}

# 更新 JSON 文件的 version
function Set-JsonVersion {
    param(
        [string]$Path,
        [string]$NewVersion
    )
    
    $json = Get-Content $Path -Raw | ConvertFrom-Json
    $json.version = $NewVersion
    $json | ConvertTo-Json -Depth 100 | Set-Content -Path $Path -Encoding UTF8
}

# 提取版本号
$ROOT_VER = Get-CargoVersion $ROOT_CARGO
$BACKEND_VER = Get-CargoVersion $BACKEND_CARGO
$FRONTEND_VER = Get-JsonVersion $FRONTEND_PKG
$TAURI_CARGO_VER = Get-CargoVersion $TAURI_CARGO
$TAURI_CONF_VER = Get-JsonVersion $TAURI_CONF

if ($Verbose) {
    Write-Host "🔧 根版本: $ROOT_VER"
    Write-Host "🦀 后端版本: $BACKEND_VER"
    Write-Host "⚛️  前端版本: $FRONTEND_VER"
    Write-Host "🖥️  Tauri Cargo 版本: $TAURI_CARGO_VER"
    Write-Host "🖥️  Tauri Conf 版本: $TAURI_CONF_VER"
}

# 检查模式
if ($Check) {
    if ($ROOT_VER -eq $BACKEND_VER -and 
        $ROOT_VER -eq $FRONTEND_VER -and 
        $ROOT_VER -eq $TAURI_CARGO_VER -and 
        $ROOT_VER -eq $TAURI_CONF_VER) {
        Write-Host "✅ 版本一致性检查通过"
        exit 0
    } else {
        Write-Host "❌ 版本不一致："
        Write-Host "  root Cargo.toml:                        $ROOT_VER"
        Write-Host "  ccr-ui/backend/Cargo.toml:              $BACKEND_VER"
        Write-Host "  ccr-ui/frontend/package.json:           $FRONTEND_VER"
        Write-Host "  ccr-ui/frontend/src-tauri/Cargo.toml:   $TAURI_CARGO_VER"
        Write-Host "  ccr-ui/frontend/src-tauri/tauri.conf.json: $TAURI_CONF_VER"
        exit 1
    }
}

# 检查是否需要同步
if ($ROOT_VER -eq $BACKEND_VER -and 
    $ROOT_VER -eq $FRONTEND_VER -and 
    $ROOT_VER -eq $TAURI_CARGO_VER -and 
    $ROOT_VER -eq $TAURI_CONF_VER) {
    Write-Host "✅ 版本一致，无需同步"
    exit 0
}

Write-Host "♻️  开始同步版本到 UI 文件..."

# 更新后端
if ($BACKEND_VER -ne $ROOT_VER) {
    Write-Host "  - 后端: $BACKEND_VER -> $ROOT_VER"
    Set-CargoVersion $BACKEND_CARGO $ROOT_VER
}

# 更新前端
if ($FRONTEND_VER -ne $ROOT_VER) {
    Write-Host "  - 前端: $FRONTEND_VER -> $ROOT_VER"
    Set-JsonVersion $FRONTEND_PKG $ROOT_VER
}

# 更新 Tauri Cargo.toml
if ($TAURI_CARGO_VER -ne $ROOT_VER) {
    Write-Host "  - Tauri Cargo.toml: $TAURI_CARGO_VER -> $ROOT_VER"
    Set-CargoVersion $TAURI_CARGO $ROOT_VER
}

# 更新 Tauri tauri.conf.json
if ($TAURI_CONF_VER -ne $ROOT_VER) {
    Write-Host "  - Tauri tauri.conf.json: $TAURI_CONF_VER -> $ROOT_VER"
    Set-JsonVersion $TAURI_CONF $ROOT_VER
}

Write-Host "✅ 同步完成"
