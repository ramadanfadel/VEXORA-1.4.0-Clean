<#
    Generates verifiable release artifacts:
      SHA256SUMS.txt  — hashes of every installer
      sbom-cargo.json — full Rust dependency graph
      sbom-npm.json   — locked npm dependency tree
    Run after .\build.ps1 -Installer.
#>
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    $out = Join-Path $root "release-artifacts"
    New-Item -ItemType Directory -Path $out -Force | Out-Null

    $nsis = Join-Path $root "src-tauri\target\release\bundle\nsis"
    $lines = @()
    if (Test-Path $nsis) {
        Get-ChildItem $nsis -Filter "*_x64-setup.exe" | ForEach-Object {
            $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
            $lines += "$hash  $($_.Name)"
        }
    }
    $lines += "# Generated $(Get-Date -Format o)"
    Set-Content -Path (Join-Path $out "SHA256SUMS.txt") -Value $lines -Encoding UTF8

    Push-Location (Join-Path $root "src-tauri")
    cargo metadata --format-version 1 > (Join-Path $out "sbom-cargo.json")
    Pop-Location

    Copy-Item (Join-Path $root "package-lock.json") (Join-Path $out "sbom-npm.json") -Force
    Write-Host "Release artifacts written to $out" -ForegroundColor Green
}
finally { Pop-Location }
