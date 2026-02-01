# RouxFlow Release Script
# Usage: .\scripts\release.ps1 -Version "0.1.2" [-Preview]

param(
    [Parameter(Mandatory=$true)]
    [string]$Version,
    
    [switch]$Preview
)

$ErrorActionPreference = "Stop"

# Build tag name
$Tag = if ($Preview) { "v$Version-preview" } else { "v$Version" }

Write-Host "🚀 Releasing RouxFlow $Tag" -ForegroundColor Cyan

# Files to update
$tomlFiles = @(
    "src-tauri/Cargo.toml",
    "crates/roux-core/Cargo.toml",
    "crates/roux-storage-sqlite/Cargo.toml",
    "crates/roux-storage-cloud/Cargo.toml"
)

# Update version in each Cargo.toml
foreach ($file in $tomlFiles) {
    if (Test-Path $file) {
        Write-Host "  📝 Updating $file" -ForegroundColor Gray
        $content = Get-Content $file -Raw
        $content = $content -replace 'version = "[0-9]+\.[0-9]+\.[0-9]+"', "version = `"$Version`""
        Set-Content $file $content -NoNewline
    }
}

# Update package.json
$packageJson = "apps/frontend/package.json"
if (Test-Path $packageJson) {
    Write-Host "  📝 Updating $packageJson" -ForegroundColor Gray
    $content = Get-Content $packageJson -Raw
    $content = $content -replace '"version": "[0-9]+\.[0-9]+\.[0-9]+"', "`"version`": `"$Version`""
    Set-Content $packageJson $content -NoNewline
}

# Update tauri.conf.json
$tauriConf = "src-tauri/tauri.conf.json"
if (Test-Path $tauriConf) {
    Write-Host "  📝 Updating $tauriConf" -ForegroundColor Gray
    $content = Get-Content $tauriConf -Raw
    $content = $content -replace '"version": "[0-9]+\.[0-9]+\.[0-9]+"', "`"version`": `"$Version`""
    Set-Content $tauriConf $content -NoNewline
}

Write-Host ""
Write-Host "✅ Version updated to $Version" -ForegroundColor Green

# Git operations
Write-Host ""
Write-Host "📦 Committing changes..." -ForegroundColor Cyan
git add -A
git commit -m "chore: bump version to $Version"

Write-Host "⬆️  Pushing commit..." -ForegroundColor Cyan
git push

Write-Host "🏷️  Creating tag $Tag..." -ForegroundColor Cyan
git tag $Tag

Write-Host "⬆️  Pushing tag..." -ForegroundColor Cyan
git push origin $Tag

Write-Host ""
Write-Host "🎉 Release $Tag complete!" -ForegroundColor Green
Write-Host "   GitHub Actions will now build and deploy." -ForegroundColor Gray
