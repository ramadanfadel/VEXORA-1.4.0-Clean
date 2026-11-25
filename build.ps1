<#
    Builds VEXORA.

    .\build.ps1              -> release exe (no installer)
    .\build.ps1 -Installer   -> NSIS installer in src-tauri\target\release\bundle\nsis
#>
param(
    [switch]$Installer,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot
try {
    if (-not (Test-Path "node_modules\.bin\tauri.cmd")) {
        Write-Host "Installing Tauri CLI..." -ForegroundColor Cyan
        & npm.cmd install --no-audit --no-fund
    }

    if ($Clean) {
        Write-Host "Cleaning previous build..." -ForegroundColor Cyan
        Remove-Item -Recurse -Force "src-tauri\target\release" -ErrorAction SilentlyContinue
    }

    if ($Installer) {
        Write-Host "Building installer..." -ForegroundColor Cyan
        & ".\node_modules\.bin\tauri.cmd" build
        Write-Host "`nInstaller(s):" -ForegroundColor Green
        Get-ChildItem "src-tauri\target\release\bundle\nsis" -ErrorAction SilentlyContinue |
            Select-Object FullName, Length
    }
    else {
        Write-Host "Building release executable..." -ForegroundColor Cyan
        & ".\node_modules\.bin\tauri.cmd" build --no-bundle
        Write-Host "`nExecutable:" -ForegroundColor Green
        Get-Item "src-tauri\target\release\vexora.exe" -ErrorAction SilentlyContinue |
            Select-Object FullName, Length
    }
}
finally {
    Pop-Location
}
