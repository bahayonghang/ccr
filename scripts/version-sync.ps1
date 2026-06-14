# 版本同步脚本（PowerShell 版本）
# 以 crates/ccr/Cargo.toml 为主，同步到各目标文件

param(
    [switch]$Check,
    [switch]$Verbose
)

# ═══════════════════════════════════════════════════════════
# 📋 同步目标配置（单一数据源）
# 修改同步逻辑时，优先检查此配置表
# ═══════════════════════════════════════════════════════════
# 格式：@{ Name = "显示名称"; Path = "相对路径"; Type = "cargo|json|vue" }
# 新增目标：在此添加哈希表项，并检查 extract/update 函数是否支持该类型
# 删除目标：从此表中移除即可
$SYNC_TARGETS = @(
    @{ Name = "ccr-types";      Path = "crates\ccr-types\Cargo.toml";             Type = "cargo" }
    @{ Name = "ccr-db";         Path = "crates\ccr-db\Cargo.toml";                Type = "cargo" }
    @{ Name = "frontend";       Path = "ccr-ui\package.json";                     Type = "json"  }
    @{ Name = "tauri-cargo";    Path = "ccr-ui\src-tauri\Cargo.toml";             Type = "cargo" }
    @{ Name = "tauri-conf";     Path = "ccr-ui\src-tauri\tauri.conf.json";        Type = "json"  }
    @{ Name = "ui-component";   Path = "ccr-ui\src\components\MainLayout.vue";    Type = "vue"   }
    @{ Name = "vscode";         Path = "ccr-vscode\package.json";                 Type = "json"  }
)
# ═══════════════════════════════════════════════════════════

# 设置 UTF-8 编码以正确显示中文
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::InputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8
try { chcp 65001 | Out-Null } catch {
    # 忽略代码页切换错误，UTF-8 编码设置已在上文完成
}

$ErrorActionPreference = "Stop"

# 获取脚本根目录
$ROOT_DIR = Split-Path -Parent (Split-Path -Parent $PSCommandPath)

# 配置文件路径
$ROOT_CARGO = Join-Path $ROOT_DIR "Cargo.toml"
$CCR_TYPES_CARGO = Join-Path $ROOT_DIR "crates\ccr-types\Cargo.toml"
$CCR_DB_CARGO = Join-Path $ROOT_DIR "crates\ccr-db\Cargo.toml"
$FRONTEND_PKG = Join-Path $ROOT_DIR "ccr-ui\package.json"
$TAURI_CARGO = Join-Path $ROOT_DIR "ccr-ui\src-tauri\Cargo.toml"
$TAURI_CONF = Join-Path $ROOT_DIR "ccr-ui\src-tauri\tauri.conf.json"
$COMPONENT_MAIN_LAYOUT = Join-Path $ROOT_DIR "ccr-ui\src\components\MainLayout.vue"
$VSCODE_PKG = Join-Path $ROOT_DIR "ccr-vscode\package.json"

# 检查文件是否存在
function Test-RequiredFile {
    param([string]$Path)
    if (-not (Test-Path $Path)) {
        Write-Error "❌ 文件不存在: $Path"
        exit 1
    }
}

Test-RequiredFile $ROOT_CARGO
Test-RequiredFile $CCR_TYPES_CARGO
Test-RequiredFile $CCR_DB_CARGO
Test-RequiredFile $FRONTEND_PKG
Test-RequiredFile $TAURI_CARGO
Test-RequiredFile $TAURI_CONF
Test-RequiredFile $COMPONENT_MAIN_LAYOUT
Test-RequiredFile $VSCODE_PKG

# 从 Cargo.toml 提取 [package] 区块中的 version
function Get-CargoVersion {
    param(
        [string]$Path,
        [string]$WorkspaceVersion = ""
    )

    $content = Get-Content $Path -Raw
    $packageBlock = $content -match '(?ms)^\[(?:workspace\.)?package\]\s*(.*?)(?=^\[|\z)' | Out-Null
    if ($matches) {
        $block = $matches[1]

        if ($block -match '(?m)^\s*version\.workspace\s*=\s*true\s*$') {
            if (-not [string]::IsNullOrWhiteSpace($WorkspaceVersion)) {
                return $WorkspaceVersion.Trim()
            }

            Write-Error "❌ $Path 使用 version.workspace = true，但未提供工作区版本"
            exit 1
        }

        if ($block -match '(?m)^\s*version\s*=\s*"([^"]+)"\s*$') {
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
# 如果目标使用 version.workspace = true，则跳过更新
function Set-CargoVersion {
    param(
        [string]$Path,
        [string]$NewVersion
    )

    $content = Get-Content $Path -Raw
    
    # 检查是否使用 workspace 版本继承
    if ($content -match '(?ms)^\[(?:workspace\.)?package\]\s*(.*?)(?=^\[|\z)') {
        $block = $matches[1]
        if ($block -match '(?m)^\s*version\.workspace\s*=\s*true\s*$') {
            if ($Verbose) {
                Write-Host "  ⏭️  $(Split-Path $Path -Leaf) 使用 workspace 版本继承，跳过"
            }
            return
        }
    }

    # 使用正则替换更新版本号
    $pattern = '(\[package\](?:(?!\[).)*?version\s*=\s*)"[^"]+"'
    $updated = [regex]::Replace($content, $pattern, { param($m) $m.Groups[1].Value + '"' + $NewVersion + '"' }, [System.Text.RegularExpressions.RegexOptions]::Singleline)
    Set-Content -Path $Path -Value $updated -NoNewline
}

# 更新 JSON 文件的 version（优先使用 jq 保留格式）
function Set-JsonVersion {
    param(
        [string]$Path,
        [string]$NewVersion
    )

    # 优先使用 jq 进行原地更新，保留原有格式
    if (Get-Command jq -ErrorAction SilentlyContinue) {
        $tmp = [System.IO.Path]::GetTempFileName()
        try {
            jq --arg ver $NewVersion '.version = $ver' $Path | Out-File -FilePath $tmp -Encoding UTF8 -NoNewline
            Move-Item -Path $tmp -Destination $Path -Force
        }
        catch {
            Remove-Item -Path $tmp -Force -ErrorAction SilentlyContinue
            Write-Error "❌ jq 更新 $Path 失败: $_"
            exit 1
        }
    }
    else {
        # 降级到 ConvertTo-Json，发出警告
        if ($Verbose) {
            Write-Warning "jq 未安装，使用 PowerShell 原生 JSON 处理，格式可能会改变"
        }
        $json = Get-Content $Path -Raw | ConvertFrom-Json
        $json.version = $NewVersion
        $json | ConvertTo-Json -Depth 100 | Set-Content -Path $Path -Encoding UTF8
    }
}

# 从 UI 布局文件中提取 CCR UI 版本
function Get-UiVersion {
    param([string]$Path)

    $content = Get-Content $Path -Raw
    if ($content -match 'CCR UI v([0-9A-Za-z._-]+)') {
        return $matches[1].Trim()
    }
    if ($content -match 'APP_VERSION_LABEL' -or $content -match 'APP_VERSION' -or $content -match 'packageJson\.version') {
        return Get-JsonVersion $FRONTEND_PKG
    }
    Write-Error "❌ 无法从 $Path 提取 CCR UI 版本号"
    exit 1
}

# 更新 UI 布局文件中的 CCR UI 版本
function Set-UiVersion {
    param(
        [string]$Path,
        [string]$NewVersion
    )

    $content = Get-Content $Path -Raw
    if ($content -match 'APP_VERSION_LABEL' -or $content -match 'APP_VERSION' -or $content -match 'packageJson\.version') {
        return
    }
    if ($content -notmatch 'CCR UI v[0-9A-Za-z._-]+') {
        Write-Error "❌ 在 $Path 中找不到 CCR UI 版本标记"
        exit 1
    }
    $updated = $content -replace 'CCR UI v[0-9A-Za-z._-]+', "CCR UI v$NewVersion"
    Set-Content -Path $Path -Value $updated -NoNewline
}

# 提取版本号
$ROOT_VER = Get-CargoVersion $ROOT_CARGO
$CCR_TYPES_VER = Get-CargoVersion $CCR_TYPES_CARGO $ROOT_VER
$CCR_DB_VER = Get-CargoVersion $CCR_DB_CARGO $ROOT_VER
$FRONTEND_VER = Get-JsonVersion $FRONTEND_PKG
$TAURI_CARGO_VER = Get-CargoVersion $TAURI_CARGO $ROOT_VER
$TAURI_CONF_VER = Get-JsonVersion $TAURI_CONF
$UI_COMPONENT_VER = Get-UiVersion $COMPONENT_MAIN_LAYOUT
$VSCODE_VER = Get-JsonVersion $VSCODE_PKG

if ($Verbose) {
    Write-Host "🔧 根版本: $ROOT_VER"
    Write-Host "📦 ccr-types 版本: $CCR_TYPES_VER"
    Write-Host "📦 ccr-db 版本: $CCR_DB_VER"
    Write-Host "⚛️  前端版本: $FRONTEND_VER"
    Write-Host "🖥️  Tauri Cargo 版本: $TAURI_CARGO_VER"
    Write-Host "🖥️  Tauri Conf 版本: $TAURI_CONF_VER"
    Write-Host "🖼️  MainLayout.vue (components) 版本: $UI_COMPONENT_VER"
    Write-Host "🔌 VSCode 扩展版本: $VSCODE_VER"
}

# 检查模式
if ($Check) {
    if ($ROOT_VER -eq $CCR_TYPES_VER -and
        $ROOT_VER -eq $CCR_DB_VER -and
        $ROOT_VER -eq $FRONTEND_VER -and
        $ROOT_VER -eq $TAURI_CARGO_VER -and
        $ROOT_VER -eq $TAURI_CONF_VER -and
        $ROOT_VER -eq $UI_COMPONENT_VER -and
        $ROOT_VER -eq $VSCODE_VER) {
        Write-Host "✅ 版本一致性检查通过"
        exit 0
    } else {
        Write-Host "❌ 版本不一致："
        Write-Host "  root Cargo.toml:                        $ROOT_VER"
        Write-Host "  crates/ccr-types/Cargo.toml:            $CCR_TYPES_VER"
        Write-Host "  crates/ccr-db/Cargo.toml:               $CCR_DB_VER"
        Write-Host "  ccr-ui/package.json:           $FRONTEND_VER"
        Write-Host "  ccr-ui/src-tauri/Cargo.toml:   $TAURI_CARGO_VER"
        Write-Host "  ccr-ui/src-tauri/tauri.conf.json: $TAURI_CONF_VER"
        Write-Host "  ccr-ui/src/components/MainLayout.vue: $UI_COMPONENT_VER"
        Write-Host "  ccr-vscode/package.json:       $VSCODE_VER"
        exit 1
    }
}

if ($ROOT_VER -eq $CCR_TYPES_VER -and
    $ROOT_VER -eq $CCR_DB_VER -and
    $ROOT_VER -eq $FRONTEND_VER -and
    $ROOT_VER -eq $TAURI_CARGO_VER -and
    $ROOT_VER -eq $TAURI_CONF_VER -and
    $ROOT_VER -eq $UI_COMPONENT_VER -and
    $ROOT_VER -eq $VSCODE_VER) {
    Write-Host "✅ 版本一致，无需同步"
    exit 0
}

Write-Host "♻️  开始同步版本到 UI 文件..."

# 更新 ccr-types
if ($CCR_TYPES_VER -ne $ROOT_VER) {
    Write-Host "  - ccr-types: $CCR_TYPES_VER -> $ROOT_VER"
    Set-CargoVersion $CCR_TYPES_CARGO $ROOT_VER
}

# 更新 ccr-db
if ($CCR_DB_VER -ne $ROOT_VER) {
    Write-Host "  - ccr-db: $CCR_DB_VER -> $ROOT_VER"
    Set-CargoVersion $CCR_DB_CARGO $ROOT_VER
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

if ($UI_COMPONENT_VER -ne $ROOT_VER) {
    Write-Host "  - MainLayout (components): $UI_COMPONENT_VER -> $ROOT_VER"
    Set-UiVersion $COMPONENT_MAIN_LAYOUT $ROOT_VER
}

# 更新 VSCode 扩展
if ($VSCODE_VER -ne $ROOT_VER) {
    Write-Host "  - VSCode 扩展: $VSCODE_VER -> $ROOT_VER"
    Set-JsonVersion $VSCODE_PKG $ROOT_VER
}

Write-Host "✅ 同步完成"
