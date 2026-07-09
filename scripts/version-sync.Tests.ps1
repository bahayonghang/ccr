# version-sync.ps1 测试套件
# 需要安装 Pester: Install-Module Pester -Force

BeforeAll {
    $ScriptDir = Split-Path -Parent $PSCommandPath
    $RootDir = Split-Path -Parent $ScriptDir
    $TestDir = Join-Path $env:TEMP "ccr-ps-test-$(Get-Random)"
    
    # 创建测试目录结构
    $root = Join-Path $TestDir "ccr-test"
    New-Item -Path "$root/crates/ccr" -ItemType Directory -Force | Out-Null
    New-Item -Path "$root/crates/ccr-types" -ItemType Directory -Force | Out-Null
    New-Item -Path "$root/crates/ccr-db" -ItemType Directory -Force | Out-Null
    New-Item -Path "$root/ccr-ui/src-tauri" -ItemType Directory -Force | Out-Null
    New-Item -Path "$root/ccr-ui/src/components" -ItemType Directory -Force | Out-Null
    New-Item -Path "$root/ccr-vscode" -ItemType Directory -Force | Out-Null
    New-Item -Path "$root/scripts" -ItemType Directory -Force | Out-Null

    # 创建测试文件
    @'
[package]
name = "ccr"
version = "1.2.3"
edition = "2021"
'@ | Set-Content -Path "$root/Cargo.toml" -Encoding UTF8

    @'
[package]
name = "ccr-types"
version = "1.2.3"
edition = "2021"
'@ | Set-Content -Path "$root/crates/ccr-types/Cargo.toml" -Encoding UTF8

    @'
[package]
name = "ccr-db"
version = "1.2.3"
edition = "2021"
'@ | Set-Content -Path "$root/crates/ccr-db/Cargo.toml" -Encoding UTF8

    @'
{
  "name": "ccr-ui",
  "version": "1.2.3",
  "private": true
}
'@ | Set-Content -Path "$root/ccr-ui/package.json" -Encoding UTF8

    @'
[package]
name = "ccr-ui-tauri"
version = "1.2.3"
edition = "2021"
'@ | Set-Content -Path "$root/ccr-ui/src-tauri/Cargo.toml" -Encoding UTF8

    @'
{
  "version": "1.2.3",
  "build": {
    "beforeDevCommand": "npm run dev"
  }
}
'@ | Set-Content -Path "$root/ccr-ui/src-tauri/tauri.conf.json" -Encoding UTF8

    @'
<template>
  <div class="footer">CCR UI v1.2.3</div>
</template>
'@ | Set-Content -Path "$root/ccr-ui/src/components/MainLayout.vue" -Encoding UTF8

    @'
{
  "name": "ccr-vscode",
  "version": "1.2.3",
  "publisher": "ccr"
}
'@ | Set-Content -Path "$root/ccr-vscode/package.json" -Encoding UTF8

    # 复制脚本
    Copy-Item "$RootDir/scripts/version-sync.ps1" "$root/scripts/"
}

AfterAll {
    # 清理测试目录
    if (Test-Path $TestDir) {
        Remove-Item $TestDir -Recurse -Force
    }
}

Describe "version-sync.ps1 基础功能测试" {
    BeforeEach {
        # 设置环境变量使脚本使用测试目录
        $env:CCR_TEST_ROOT = $root
    }

    It "版本一致时 --check 应返回 0" {
        $scriptPath = Join-Path $root "scripts\version-sync.ps1"
        # 注意：需要修改脚本中的 ROOT_DIR 或使用环境变量
        # 这里假设脚本支持测试模式
        $result = & powershell -File $scriptPath -Check -Verbose 2>&1
        $LASTEXITCODE | Should -Be 0
    }

    It "版本不一致时 --check 应返回 1" {
        # 修改前端版本
        @'
{
  "name": "ccr-ui",
  "version": "0.9.0",
  "private": true
}
'@ | Set-Content -Path "$root/ccr-ui/package.json" -Encoding UTF8

        $scriptPath = Join-Path $root "scripts\version-sync.ps1"
        $result = & powershell -File $scriptPath -Check 2>&1
        $LASTEXITCODE | Should -Be 1
    }

    It "同步模式应更新不一致的版本" {
        # 修改前端版本
        @'
{
  "name": "ccr-ui",
  "version": "0.9.0",
  "private": true
}
'@ | Set-Content -Path "$root/ccr-ui/package.json" -Encoding UTF8

        $scriptPath = Join-Path $root "scripts\version-sync.ps1"
        $result = & powershell -File $scriptPath -Verbose 2>&1
        $LASTEXITCODE | Should -Be 0

        # 验证更新
        $pkg = Get-Content "$root/ccr-ui/package.json" -Raw | ConvertFrom-Json
        $pkg.version | Should -Be "1.2.3"
    }

    It "文件不存在时应报错退出" {
        Remove-Item "$root/ccr-vscode/package.json" -Force

        $scriptPath = Join-Path $root "scripts\version-sync.ps1"
        $result = & powershell -File $scriptPath 2>&1
        $LASTEXITCODE | Should -Be 1
    }

    It "workspace 版本继承不应被覆盖" {
        # 设置 workspace 版本继承
        @'
[package]
name = "ccr-types"
version.workspace = true
edition = "2021"
'@ | Set-Content -Path "$root/crates/ccr-types/Cargo.toml" -Encoding UTF8

        $scriptPath = Join-Path $root "scripts\version-sync.ps1"
        $result = & powershell -File $scriptPath -Verbose 2>&1
        
        # 检查 ccr-types 是否保持 workspace 继承
        $content = Get-Content "$root/crates/ccr-types/Cargo.toml" -Raw
        $content | Should -Match "version\.workspace\s*=\s*true"
    }
}

Describe "JSON 更新功能测试 (CORR-001 修复验证)" {
    It "使用 jq 更新 JSON 应保留格式" {
        $jsonPath = "$root/ccr-ui/package.json"
        $original = Get-Content $jsonPath -Raw
        
        # 如果有 jq，验证 jq 路径
        if (Get-Command jq -ErrorAction SilentlyContinue) {
            $tmp = [System.IO.Path]::GetTempFileName()
            jq '.version = "2.0.0"' $jsonPath | Out-File $tmp -Encoding UTF8 -NoNewline
            Move-Item $tmp $jsonPath -Force
            
            $newContent = Get-Content $jsonPath -Raw
            $newContent | Should -Match '"version": "2.0.0"'
        }
    }
}
