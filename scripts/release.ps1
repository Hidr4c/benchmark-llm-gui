<#
.SYNOPSIS
  Industrialized Git Flow release automation script for Benchmark LLM Explorer.
.DESCRIPTION
  Automates the production release workflow:
  - Validates repository hygiene and test suites.
  - Creates and manages dedicated release/vX.Y.Z branches.
  - Bumps Cargo.toml and updates Cargo.lock.
  - Merges into master (squashed: 1 commit per release) and develop.
  - Manages granular release branches (vX.Y.Z) and annotated release tags.
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

    [string]$Title = "",

    [switch]$SkipTests
)

$ErrorActionPreference = "Stop"

# Normalize version (strip leading 'v' if present)
$CleanVersion = $Version.TrimStart('v')
if ($CleanVersion -notmatch '^\d+\.\d+\.\d+$') {
    throw "Invalid semantic version format: '$Version'. Expected format: X.Y.Z (e.g. 1.1.0)"
}

if (-not $Title) {
    $Title = switch ($CleanVersion) {
        "1.2.0" { "Windows GUI Subsystem, Complete EN/FR Parity & Hardware Profiler" }
        default { "Universal LLM Benchmark Explorer & Hardware Profiler" }
    }
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
    
    # 1. Ensure on develop
    git checkout develop
    $hasRemoteDevelop = git ls-remote --heads origin develop
    if ($hasRemoteDevelop) {
        git pull origin develop
    }

    Run-Tests

    # 2. Create release branch (strictly local / private)
    Write-Host "`n>>> [2/4] Creating local release branch $ReleaseBranch..." -ForegroundColor Yellow
    git checkout -b $ReleaseBranch develop

    # 3. Bump version in Cargo.toml
    Write-Host ">>> [3/4] Updating version to $CleanVersion in Cargo.toml..." -ForegroundColor Yellow
    $CargoPath = "Cargo.toml"
    $CargoContent = Get-Content $CargoPath -Raw
    $NewCargo = $CargoContent -replace '(?m)^version\s*=\s*"[^"]+"', "version = `"$CleanVersion`""
    Set-Content -Path $CargoPath -Value $NewCargo -NoNewline

    # Update Cargo.lock
    cargo check --quiet

    # 4. Commit version bump on release branch (kept private / local)
    Write-Host ">>> [4/4] Committing version bump on $ReleaseBranch (local only)..." -ForegroundColor Yellow
    git add Cargo.toml Cargo.lock
    git commit -m "chore(release): bump version to $CleanVersion for $Tag"

    Write-Host "`n[SUCCESS] Local release branch $ReleaseBranch prepared (private)." -ForegroundColor Green
}

function Step-FinishRelease {
    Write-Host "`n>>> Finishing release $Tag..." -ForegroundColor Cyan

    Assert-GitClean

    # 1. Update master to release content (strictly 1 squashed commit per release version)
    Write-Host ">>> Updating master tree with $ReleaseBranch content..." -ForegroundColor Yellow
    git checkout master
    git pull origin master
    git checkout $ReleaseBranch -- .
    git commit -m "feat: release $Tag - $Title"

    # 2. Create annotated tag pointing to master release commit
    Write-Host ">>> Creating annotated release tag $Tag..." -ForegroundColor Yellow
    git tag -a $Tag -m "Release $Tag - $Title" -f

    # 3. Create or update granular release branch vX.Y.Z
    Write-Host ">>> Updating granular release branch $Tag..." -ForegroundColor Yellow
    git branch -f $Tag $ReleaseBranch

    # 4. Merge back into develop (locally)
    Write-Host ">>> Merging $ReleaseBranch back into develop (locally)..." -ForegroundColor Yellow
    git checkout develop
    $hasRemoteDevelop = git ls-remote --heads origin develop
    if ($hasRemoteDevelop) {
        git pull origin develop
    }
    git merge --no-ff $ReleaseBranch -m "merge: sync release $Tag into develop"

    # 5. Push ONLY master and release tag to origin (Public GitHub)
    Write-Host ">>> Pushing master and tag $Tag to origin (release branches remain strictly local/private)..." -ForegroundColor Yellow
    git push origin master
    git push origin "refs/tags/$Tag`:refs/tags/$Tag" --force

    Write-Host "`n==========================================================" -ForegroundColor Green
    Write-Host "  RELEASE $Tag COMPLETED AND DEPLOYED!" -ForegroundColor Green
    Write-Host "  - Master: updated on GitHub (strictly squashed 1 commit per release)" -ForegroundColor Green
    Write-Host "  - Tag: $Tag pushed (GitHub Actions release workflow triggered)" -ForegroundColor Green
    Write-Host "  - Release & develop branches: kept local and private" -ForegroundColor Green
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
