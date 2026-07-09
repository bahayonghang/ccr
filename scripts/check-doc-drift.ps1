param(
    [switch]$Verbose
)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

$RootDir = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
$ReadmePath = Join-Path $RootDir 'ccr-ui\README.md'
$PackagePath = Join-Path $RootDir 'ccr-ui\package.json'
$BunLockPath = Join-Path $RootDir 'ccr-ui\bun.lock'
$NpmLockPath = Join-Path $RootDir 'ccr-ui\package-lock.json'
$TauriCargoPath = Join-Path $RootDir 'ccr-ui\src-tauri\Cargo.toml'

function Fail([string]$Message) {
    Write-Error "❌ $Message"
    exit 1
}

foreach ($path in @($ReadmePath, $PackagePath, $BunLockPath, $TauriCargoPath)) {
    if (-not (Test-Path $path)) {
        Fail "文件不存在: $path"
    }
}

if (Test-Path $NpmLockPath) {
    Fail 'ccr-ui/package-lock.json 存在；ccr-ui 只维护 Bun/bun.lock'
}

$pkg = Get-Content $PackagePath -Raw | ConvertFrom-Json
$frontendVersion = [string]$pkg.version
$packageManager = [string]$pkg.packageManager
if ([string]::IsNullOrWhiteSpace($frontendVersion)) {
    Fail 'ccr-ui/package.json 缺少 version 字段'
}
if ($packageManager -notmatch '^bun@[0-9]') {
    Fail "ccr-ui/package.json#packageManager 必须声明 bun@x.y.z，当前: $packageManager"
}

$tauriCargo = Get-Content $TauriCargoPath -Raw
if ($tauriCargo -notmatch '(?m)^rust-version\s*=\s*"([^"]+)"') {
    Fail 'ccr-ui/src-tauri/Cargo.toml 缺少 rust-version'
}
$rustVersion = $matches[1]
if ($tauriCargo -notmatch '(?m)^edition\s*=\s*"([^"]+)"') {
    Fail 'ccr-ui/src-tauri/Cargo.toml 缺少 edition'
}
$edition = $matches[1]

$readme = Get-Content $ReadmePath -Raw
$required = @(
    "version-$frontendVersion",
    "Bun is the only maintained frontend package manager",
    'bun.lock is the dependency source of truth',
    "Bun | ``$packageManager``",
    "Rust | ``>= $rustVersion``",
    "Rust edition | Edition $edition",
    'Tauri invoke APIs',
    'Web runtime',
    'bun run lint:fix'
)
foreach ($needle in $required) {
    if (-not $readme.Contains($needle)) {
        Fail "ccr-ui/README.md 缺少当前事实: $needle"
    }
}

$stalePatterns = @(
    'version-2.5.0',
    'TypeScript-5.7',
    'Rust >= 1.70',
    'Edition 2021',
    'Tokio 1.48',
    'Axios',
    'HTTP API',
    '13 个命令',
    'Web 模式: 浏览器访问，通过 HTTP API',
    '自动检测环境，透明切换后端'
)
foreach ($pattern in $stalePatterns) {
    if ($readme.Contains($pattern)) {
        Fail "ccr-ui/README.md 仍包含过期描述: $pattern"
    }
}

if ($Verbose) {
    Write-Host "📄 ccr-ui/README.md version: $frontendVersion"
    Write-Host "📦 package manager: $packageManager"
    Write-Host "🦀 rust-version: $rustVersion, edition: $edition"
    Write-Host '🔒 JS lock strategy: bun.lock only'
}
Write-Host '✅ 文档/锁文件 drift 检查通过'
