param(
  [string]$SourceRepo = "https://github.com/bahayonghang/MapleLXGWBright.git",
  [string]$SourceCommit = "c0f2af4ffad1a1d68b6e873ee416e4cf2a8ce05d",
  [int]$ChunkSize = 2000000
)

$ErrorActionPreference = "Stop"

$projectRoot = (Resolve-Path "$PSScriptRoot/..").Path
$workspaceRoot = (Resolve-Path "$projectRoot/..").Path
$workDir = "$workspaceRoot/.tmp/maplebright-font-build"
$repoDir = "$workDir/repo"
$splitDir = "$repoDir/output/split"
$destBase = "$projectRoot/public/fonts/maplebright"

Write-Host "==> Preparing workspace: $workDir"
New-Item -ItemType Directory -Force -Path $workDir | Out-Null

if (!(Test-Path "$repoDir/.git")) {
  Write-Host "==> Cloning source repo"
  git clone --depth 1 "$SourceRepo" "$repoDir"
}

Set-Location "$repoDir"
Write-Host "==> Syncing source commit: $SourceCommit"
git fetch --depth 1 origin "$SourceCommit"
git checkout --detach "$SourceCommit"

Write-Host "==> Installing Python dependencies (uv)"
uv sync

Write-Host "==> Building MapleBright fonts"
uv run python build.py --styles Regular,Medium,Italic,MediumItalic

$styles = @(
  @{ Name = "Regular"; Weight = "400"; Style = "normal" },
  @{ Name = "Medium"; Weight = "500"; Style = "normal" },
  @{ Name = "Italic"; Weight = "400"; Style = "italic" },
  @{ Name = "MediumItalic"; Weight = "500"; Style = "italic" }
)

Write-Host "==> Splitting fonts to woff2 subsets"
foreach ($style in $styles) {
  $subsetOut = "$splitDir/MapleBright-$($style.Name)-opt"
  if (Test-Path $subsetOut) {
    [System.IO.Directory]::Delete($subsetOut, $true)
  }

  npx --yes cn-font-split run `
    -i "output/fonts/MapleBright-$($style.Name).ttf" `
    -o "$subsetOut" `
    --css.fontFamily MapleBright `
    --css.fontWeight "$($style.Weight)" `
    --css.fontStyle "$($style.Style)" `
    --css.fontDisplay swap `
    --renameOutputFont "[index].[ext]" `
    --testHtml false `
    --reporter false `
    --css.commentBase false `
    --css.commentNameTable false `
    --css.commentUnicodes false `
    -c $ChunkSize
}

Write-Host "==> Syncing assets to: $destBase"
New-Item -ItemType Directory -Force -Path $destBase | Out-Null
foreach ($style in $styles) {
  $targetDir = "$destBase/MapleBright-$($style.Name)"
  if (Test-Path $targetDir) {
    [System.IO.Directory]::Delete($targetDir, $true)
  }
  New-Item -ItemType Directory -Force -Path $targetDir | Out-Null

  $subsetOut = "$splitDir/MapleBright-$($style.Name)-opt"
  Copy-Item "$subsetOut/*.woff2" -Destination "$targetDir" -Force
  Copy-Item "$subsetOut/result.css" -Destination "$targetDir" -Force
}

Write-Host "==> Done. MapleBright assets are updated."
