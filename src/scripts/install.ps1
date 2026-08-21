# Open Document Spec (ods) installer for Windows (PowerShell 5.1+)
#
# Supported platforms (auto-detected):
#   Windows x64   — windows-x86_64
#   Windows ARM64 — windows-arm64
#
# Usage:
#   irm https://raw.githubusercontent.com/open-doc-spec/ods/main/src/scripts/install.ps1 | iex
#
# Options via environment variables:
#   ODS_VERSION — pin a release tag, e.g. "v0.1.0"  (default: latest; legacy ODC_VERSION still read)
#   ODS_PREFIX  — install dir (default: %LOCALAPPDATA%\Programs\ods; legacy ODC_PREFIX still read)
#   ODS_NO_VERIFY — set to "1" to skip SHA256 checksum verification
#   GH_TOKEN / GITHUB_TOKEN — optional token (e.g. for higher API rate limits)
#
[CmdletBinding()]
param(
    [switch]$Force
)
$ErrorActionPreference = "Stop"

$isForce = $Force -or ($env:ODS_FORCE -eq "1")

$Repo = "open-doc-spec/ods"
$Api  = "https://api.github.com/repos/$Repo"

function Write-Step { Write-Host "==> $($args -join ' ')" }
function Write-Warn { Write-Warning $($args -join ' ') }

function Get-GitHubToken {
    if ($env:GH_TOKEN) { return $env:GH_TOKEN }
    if ($env:GITHUB_TOKEN) { return $env:GITHUB_TOKEN }
    return $null
}

function Get-AuthHeaders {
    param([string]$Accept = "application/vnd.github+json")
    $token = Get-GitHubToken
    $h = @{
        "Accept"     = $Accept
        "User-Agent" = "ods-install"
    }
    if ($token) {
        $h["Authorization"] = "Bearer $token"
    }
    return $h
}

function Normalize-OdsVersion {
    param([string]$Version)
    if (-not $Version) { return "0.0.0" }
    return ($Version.Trim() -replace '^[vV]', '')
}

function Compare-OdsVersion {
    param(
        [Parameter(Mandatory)] [string] $Left,
        [Parameter(Mandatory)] [string] $Right
    )
    $l = (Normalize-OdsVersion $Left).Split('.')
    $r = (Normalize-OdsVersion $Right).Split('.')
    for ($i = 0; $i -lt 3; $i++) {
        $li = if ($i -lt $l.Length) { [int]($l[$i] -replace '[^0-9].*$', '') } else { 0 }
        $ri = if ($i -lt $r.Length) { [int]($r[$i] -replace '[^0-9].*$', '') } else { 0 }
        if ($li -gt $ri) { return 1 }
        if ($li -lt $ri) { return -1 }
    }
    return 0
}

function Get-InstalledOdsVersion {
    $cmd = Get-Command ods -ErrorAction SilentlyContinue
    if ($cmd) {
        $out = & $cmd.Source --version 2>$null
        if ($out -match '(?:ods|ods)\s+([^\s]+)') { return $Matches[1] }
    }
    $prefix = $env:ODS_PREFIX
    if (-not $prefix) { $prefix = Join-Path $env:LOCALAPPDATA "Programs\ods" }
    $candidate = Join-Path $prefix "ods.exe"
    if (Test-Path $candidate) {
        $out = & $candidate --version 2>$null
        if ($out -match '(?:ods|ods)\s+([^\s]+)') { return $Matches[1] }
    }
    return $null
}

function Get-Release {
    param([string]$Tag)
    if ($Tag) {
        return Invoke-RestMethod "$Api/releases/tags/$Tag" -Headers (Get-AuthHeaders)
    }
    return Invoke-RestMethod "$Api/releases/latest" -Headers (Get-AuthHeaders)
}

function Download-ReleaseAsset {
    param(
        [Parameter(Mandatory)] [string] $Tag,
        [Parameter(Mandatory)] [string] $Name,
        [Parameter(Mandatory)] [string] $OutFile,
        $Release
    )
    $directUrl = "https://github.com/$Repo/releases/download/$Tag/$Name"
    try {
        Invoke-WebRequest -Uri $directUrl -OutFile $OutFile -UserAgent "ods-install" -UseBasicParsing
        return
    } catch {
        # Fallback to API asset download
    }

    if (-not $Release) {
        $Release = Get-Release -Tag $Tag
    }
    $asset = $Release.assets | Where-Object { $_.name -eq $Name } | Select-Object -First 1
    if (-not $asset) {
        throw "Asset '$Name' not found on release $($Release.tag_name)"
    }
    Invoke-WebRequest -Uri "$Api/releases/assets/$($asset.id)" `
        -OutFile $OutFile `
        -Headers (Get-AuthHeaders -Accept "application/octet-stream") `
        -UseBasicParsing
}

# ── Architecture detection → short asset id ───────────────────────────────────
$ProcArch = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture
$Asset = switch ($ProcArch.ToString()) {
    "X64"   { "windows-x86_64" }
    "Arm64" { "windows-arm64" }
    default { throw "Unsupported architecture: $ProcArch. Only x64 and ARM64 are supported." }
}

# ── Version resolution ────────────────────────────────────────────────────────
$Version = $env:ODS_VERSION
if (-not $Version) { $Version = $env:ODC_VERSION } # legacy fallback
try {
    if ($Version) {
        Write-Step "Fetching release $Version..."
        $Release = Get-Release -Tag $Version
    } else {
        Write-Step "Resolving latest ODS release..."
        $Release = Get-Release
        $Version = $Release.tag_name
    }
} catch {
    throw "Could not reach GitHub API. Error: $_"
}
if (-not $Version) { throw "Could not resolve latest release tag." }
Write-Step "Installing ODS $Version for $Asset"

$InstalledVersion = Get-InstalledOdsVersion
if (-not $isForce -and $InstalledVersion -and ((Compare-OdsVersion $InstalledVersion $Version) -ge 0)) {
    Write-Step "ods $InstalledVersion is up to date (latest $(Normalize-OdsVersion $Version))"
    Write-Step "Use -Force (or set `$env:ODS_FORCE = '1') to force re-installation."
    $cmd = Get-Command ods -ErrorAction SilentlyContinue
    if ($cmd) {
        & $cmd.Source --version
    } else {
        $prefix = $env:ODS_PREFIX
        if (-not $prefix) { $prefix = Join-Path $env:LOCALAPPDATA "Programs\ods" }
        & (Join-Path $prefix "ods.exe") --version
    }
    return
}

# ── Filenames ─────────────────────────────────────────────────────────────────
$Filename = "ods-$Version-$Asset.zip"

# ── Temp workspace ────────────────────────────────────────────────────────────
$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "ods-install-$([System.Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null
try {

# ── Download ──────────────────────────────────────────────────────────────────
Write-Step "Downloading $Filename..."
try {
    Download-ReleaseAsset -Tag $Version -Release $Release -Name $Filename -OutFile "$TmpDir\$Filename"
} catch {
    throw "Download failed for archive on $Version`nhttps://github.com/$Repo/releases`nError: $_"
}

# ── Checksum verification ─────────────────────────────────────────────────────
if ($env:ODS_NO_VERIFY -ne "1") {
    Write-Step "Verifying checksum..."
    try {
        Download-ReleaseAsset -Tag $Version -Release $Release -Name "SHA256SUMS" -OutFile "$TmpDir\SHA256SUMS"
    } catch {
        throw "Could not download SHA256SUMS for $Version"
    }
    $SumsContent = Get-Content "$TmpDir\SHA256SUMS"
    $ExpectedLine = $SumsContent | Where-Object { $_ -match " $([regex]::Escape($Filename))$" } | Select-Object -First 1
    if (-not $ExpectedLine) {
        throw "No checksum entry found for '$Filename' in SHA256SUMS."
    }
    $Expected = ($ExpectedLine -split '\s+')[0].ToLowerInvariant()
    $Actual   = (Get-FileHash "$TmpDir\$Filename" -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($Expected -ne $Actual) {
        throw "Checksum mismatch!`n  Expected: $Expected`n  Got:      $Actual`nThe downloaded file may be corrupt or tampered with."
    }
    Write-Host "    Checksum OK"
}

# ── Extract ───────────────────────────────────────────────────────────────────
Write-Step "Extracting..."
Expand-Archive -Path "$TmpDir\$Filename" -DestinationPath $TmpDir -Force
$Extracted = "$TmpDir\ods-$Version-$Asset"
$BinSrc = $null
if (Test-Path "$Extracted\ods.exe") { $BinSrc = "$Extracted\ods.exe" }
else {
    $found = Get-ChildItem -Path $TmpDir -Recurse -Include "ods.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) { $BinSrc = $found.FullName }
}
if (-not $BinSrc) { throw "ods.exe not found in archive" }

# ── Install ───────────────────────────────────────────────────────────────────
$Prefix = $env:ODS_PREFIX
if (-not $Prefix) { $Prefix = $env:ODC_PREFIX } # legacy fallback
if (-not $Prefix) { $Prefix = Join-Path $env:LOCALAPPDATA "Programs\ods" }
New-Item -ItemType Directory -Force -Path $Prefix | Out-Null
Copy-Item $BinSrc (Join-Path $Prefix "ods.exe") -Force

Write-Host ""
Write-Host "==> Installed successfully:"
Write-Host "    $Prefix\ods.exe"

# ── PATH update ───────────────────────────────────────────────────────────────
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$Prefix*") {
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$Prefix", "User")
    Write-Host ""
    Write-Host "  NOTE: '$Prefix' has been added to your user PATH."
    Write-Host "  Please restart your terminal (or run: `$env:PATH += ';$Prefix'`)"
} else {
    Write-Host "  '$Prefix' is already in your PATH."
}

} finally {
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}

# ── Next steps ────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  Verify installation (in a new terminal):"
Write-Host "    ods --version"
Write-Host ""
Write-Host "  Get started:"
Write-Host "    ods init .              # make project ODS-compliant (creates root ods.toml)"
Write-Host "    ods setup               # set up machine background service & check workspace health"
Write-Host "    ods lint"
Write-Host "    ods export              # optional graph.md for AI"
Write-Host ""
Write-Host "  Keep tools current:"
Write-Host "    ods update              # update binary & restart background service"
Write-Host "    (auto-check ~daily; disable with ODS_AUTO_UPDATE=0)"
Write-Host ""
Write-Host "  Guide: https://github.com/$Repo/blob/main/README.md"
Write-Host "  Changelog: https://github.com/$Repo/blob/main/CHANGELOG.md"
