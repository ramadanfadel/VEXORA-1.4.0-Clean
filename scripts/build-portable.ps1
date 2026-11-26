<#
    Builds a portable VEXORA folder next to the source tree.
    Requires a completed release build (.\build.ps1).
    The resulting folder is self-contained; dropping portable.json beside the
    exe switches data_dir() to VEXORA-Data (no %APPDATA% writes).
#>
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$release = Join-Path $root "src-tauri\target\release"
$exe = Join-Path $release "vexora.exe"
if (-not (Test-Path $exe)) { throw "Run .\build.ps1 first" }
$out = Join-Path $root "VEXORA-Portable"
Remove-Item $out -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $out -Force | Out-Null

Copy-Item $exe $out
Get-ChildItem $release -Filter "*.dll" -File | Copy-Item -Destination $out -Force
Get-ChildItem $release -Filter "*.cat" -File | Copy-Item -Destination $out -Force
if (Test-Path (Join-Path $release "resources")) {
  Copy-Item (Join-Path $release "resources") $out -Recurse -Force
}
Set-Content -Path (Join-Path $out "portable.json") -Value '{ "portable": true }' -Encoding UTF8
Write-Host "Portable build: $out" -ForegroundColor Green
