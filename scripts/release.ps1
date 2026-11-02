<#
.SYNOPSIS
  Industrialized Git Flow release automation script for Benchmark LLM Explorer.
.DESCRIPTION
  Automates the production release workflow:
  - Validates repository hygiene and test suites.
  - Creates and manages dedicated release/vX.Y.Z branches.
  - Bumps Cargo.toml and updates Cargo.lock.
  - Merges into master and develop with annotated git tags.
  - Synchronizes all branches and release tags to origin.
.EXAMPLE
  .\scripts\release.ps1 -Version "1.1.0" -Action full
  .\scripts\release.ps1 -Version "1.1.0" -Action start
  .\scripts\release.ps1 -Version "1.1.0" -Action finish
#>

param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Version,

    [ValidateSet("start", "finish", "full")]
    [string]$Action = "full",

    [switch]$SkipTests
)

$ErrorActionPreference = "Stop"

# Normalize version (strip leading 'v' if present)
$CleanVersion = $Version.TrimStart('v')
if ($CleanVersion -notmatch '^\d+\.\d+\.\d+$') {
    throw "Invalid semantic version format: '$Version'. Expected format: X.Y.Z (e.g. 1.1.0)"
}

$Tag = "v$CleanVersion"
$ReleaseBranch = "release/$Tag"

function Assert-GitClean {
    $status = git status --porcelain
    if ($status) {
        throw "Working tree is dirty. Please commit or stash changes before running release."
    }
}

function Run-Tests {
    if (-not $SkipTests) {
        Write-Host "`n>>> [1/4] Running test verification suite (cargo test)..." -ForegroundColor Yellow
        cargo test --quiet
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo test suite failed. Aborting release."
        }
        Write-Host "    [OK] All tests passed." -ForegroundColor Green
    } else {
        Write-Host "`n>>> Skipping tests as requested (-SkipTests)." -ForegroundColor DarkGray
    }
}

function Step-StartRelease {
    Write-Host "`n>>> Starting release $Tag on branch $ReleaseBranch..." -ForegroundColor Cyan
    
    # 1. Ensure on develop and up to date
    git checkout develop
    git pull origin develop

    Run-Tests

    # 2. Create release branch
    Write-Host "`n>>> [2/4] Creating release branch $ReleaseBranch..." -ForegroundColor Yellow
    git checkout -b $ReleaseBranch develop

    # 3. Bump version in Cargo.toml
    Write-Host ">>> [3/4] Updating version to $CleanVersion in Cargo.toml..." -ForegroundColor Yellow
    $CargoPath = "Cargo.toml"
    $CargoContent = Get-Content $CargoPath -Raw
    $NewCargo = $CargoContent -replace '(?m)^version\s*=\s*"[^"]+"', "version = `"$CleanVersion`""
    Set-Content -Path $CargoPath -Value $NewCargo -NoNewline

    # Update Cargo.lock
    cargo check --quiet

    # 4. Commit version bump on release branch
    Write-Host ">>> [4/4] Committing version bump on $ReleaseBranch..." -ForegroundColor Yellow
    git add Cargo.toml Cargo.lock
    git commit -m "chore(release): bump version to $CleanVersion for $Tag"
    git push origin $ReleaseBranch

    Write-Host "`n[SUCCESS] Release branch $ReleaseBranch created and pushed to origin." -ForegroundColor Green
}

function Step-FinishRelease {
    Write-Host "`n>>> Finishing release $Tag..." -ForegroundColor Cyan

    Assert-GitClean

    # 1. Merge into master
    Write-Host ">>> Merging $ReleaseBranch into master..." -ForegroundColor Yellow
    git checkout master
    git pull origin master
    git merge --no-ff $ReleaseBranch -m "merge: release $Tag into master"

    # 2. Create annotated tag
    Write-Host ">>> Creating annotated release tag $Tag..." -ForegroundColor Yellow
    git tag -a $Tag -m "Release $Tag - Universal LLM Benchmark Explorer & Hardware Profiler"

    # 3. Merge back into develop
    Write-Host ">>> Merging $ReleaseBranch back into develop..." -ForegroundColor Yellow
    git checkout develop
    git pull origin develop
    git merge --no-ff $ReleaseBranch -m "merge: sync release $Tag into develop"

    # 4. Push all changes
    Write-Host ">>> Pushing master, develop, $ReleaseBranch, and tag $Tag to origin..." -ForegroundColor Yellow
    git push origin master
    git push origin develop
    git push origin $ReleaseBranch
    git push origin $Tag

    Write-Host "`n==========================================================" -ForegroundColor Green
    Write-Host "  RELEASE $Tag COMPLETED AND DEPLOYED!" -ForegroundColor Green
    Write-Host "  - Master: updated and merged" -ForegroundColor Green
    Write-Host "  - Develop: updated and synchronized" -ForegroundColor Green
    Write-Host "  - Release branch: $ReleaseBranch preserved" -ForegroundColor Green
    Write-Host "  - Tag: $Tag pushed (GitHub Actions release workflow triggered)" -ForegroundColor Green
    Write-Host "==========================================================" -ForegroundColor Green
}

# Main Execution Dispatcher
Assert-GitClean

switch ($Action) {
    "start" {
        Step-StartRelease
    }
    "finish" {
        Step-FinishRelease
    }
    "full" {
        Step-StartRelease
        Step-FinishRelease
    }
}
