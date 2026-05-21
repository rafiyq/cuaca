#Requires -Version 5.1
<#
.SYNOPSIS
    Installer for cuaca on Windows.
.DESCRIPTION
    Downloads the latest (or specified) release asset from GitHub,
    verifies SHA256 checksum, extracts cuaca.exe, and installs it to the user's bin directory.
.PARAMETER BinDir
    Directory to install cuaca.exe. Defaults to "$env:USERPROFILE\bin".
.PARAMETER DryRun
    Show what would be done without making changes.
.PARAMETER Force
    Overwrite existing installation even if up-to-date.
.PARAMETER NoVerify
    Skip checksum verification.
.PARAMETER Version
    Specific version tag (without leading 'v'). Default is 'latest'.
.PARAMETER Yes
    Assume yes to prompts.
.PARAMETER Help
    Show this help.
.EXAMPLE
    .\install.ps1 -BinDir "$env:ProgramFiles\cuaca" -Yes
#>

[CmdletBinding()]
param(
    [string]$BinDir = "$env:USERPROFILE\bin",
    [switch]$DryRun,
    [switch]$Force,
    [switch]$NoVerify,
    [string]$Version,
    [switch]$Yes,
    [switch]$Help
)

function Show-Help {
    Get-Help $MyInvocation.MyCommand.Path -Full | Out-Host
    exit 0
}

if ($Help) { Show-Help }

$ErrorActionPreference = 'Stop'
$UserAgent = "cuaca-installer/1.0"
$GitHubRepo = "rafiyq/cuaca"

# Ensure bin directory exists
if (-not (Test-Path $BinDir)) {
    if (-not $DryRun) {
        New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    }
}

# Check existing installation
$ExistingVersion = $null
$CuacaPath = Join-Path $BinDir "cuaca.exe"
if (Test-Path $CuacaPath) {
    try {
        $ExistingVersion = & $CuacaPath --version 2>&1
    } catch { }
    if ($ExistingVersion) {
        Write-Host "Existing installation: $ExistingVersion"
        if (-not $Force -and $ExistingVersion -eq "v$Version") {
            Write-Host "Same version already installed. Use -Force to reinstall."
            exit 0
        }
    }
}

# Prompt
if (-not $Yes -and -not $DryRun) {
    $confirmation = Read-Host "Install cuaca v$Version to $BinDir? [y/N] "
    if ($confirmation -notmatch '^[Yy]') {
        Write-Host "Aborted."
        exit 1
    }
}

# Determine version
if (-not $PSBoundParameters.ContainsKey('Version')) {
    Write-Host "Fetching latest release information..."
    $ApiUrl = "https://api.github.com/repos/$GitHubRepo/releases/latest"
    $ReleaseJson = Invoke-RestMethod -Uri $ApiUrl -UserAgent $UserAgent -ErrorAction Stop
    $Tag = $ReleaseJson.tag_name
    if (-not $Tag) { throw "Could not determine latest tag." }
    $Version = $Tag.TrimStart('v')
} else {
    $Version = $Version.TrimStart('v')
}

Write-Host "Installing cuaca version v$Version for Windows"

$AssetName = "cuaca-v$Version-x86_64-windows.zip"
$AssetUrl = "https://github.com/$GitHubRepo/releases/download/v$Version/$AssetName"
$ChecksumName = "$AssetName.sha256"
$ChecksumUrl = "https://github.com/$GitHubRepo/releases/download/v$Version/$ChecksumName"

$TempDir = Join-Path $env:TEMP ("cuaca_install_" + [Guid]::NewGuid().ToString("N"))
try {
    if (-not $DryRun) {
        New-Item -ItemType Directory -Force -Path $TempDir | Out-Null
    }

    $ArchivePath = Join-Path $TempDir $AssetName
    $ChecksumPath = Join-Path $TempDir $ChecksumName

    function Download-File {
        param($Url, $Out)
        Write-Host "Downloading: $Url"
        if ($DryRun) { return }
        try {
            Invoke-WebRequest -Uri $Url -UserAgent $UserAgent -OutFile $Out -ErrorAction Stop | Out-Null
        } catch {
            throw "Download failed: $Url"
        }
    }

    Download-File -Url $AssetUrl -Out $ArchivePath
    if ($NoVerify) {
        Write-Host "Skipping checksum verification."
    } else {
        try {
            Download-File -Url $ChecksumUrl -Out $ChecksumPath
        } catch {
            Write-Warning "Checksum file not found, skipping verification."
            $NoVerify = $true
        }
    }

    # Verify checksum if available
    if (-not $NoVerify -and (Test-Path $ChecksumPath)) {
        Write-Host "Verifying checksum..."
        $Expected = Get-Content $ChecksumPath -Raw | ForEach-Object { ($_ -split '\s+')[0] }
        $Actual = Get-FileHash -Path $ArchivePath -Algorithm SHA256 | Select-Object -ExpandProperty Hash
        if ($Expected -ne $Actual) {
            throw "Checksum mismatch! Expected $Expected, got $Actual"
        }
        Write-Host "Checksum verified."
    }

    if ($DryRun) {
        Write-Host "[DRY RUN] Would extract '$AssetName' and install cuaca.exe to '$BinDir'"
        exit 0
    }

    # Extract
    Extract-Dir = Join-Path $TempDir "extract"
    New-Item -ItemType Directory -Force -Path $Extract-Dir | Out-Null
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::ExtractToDirectory($ArchivePath, $Extract-Dir)

    $BinarySrc = Join-Path $Extract-Dir "cuaca.exe"
    if (-not (Test-Path $BinarySrc)) {
        # Might be inside a subdirectory named after the asset
        $possible = Get-ChildItem -Path $Extract-Dir -Recurse -File -Filter cuaca.exe | Select-Object -First 1
        if ($possible) { $BinarySrc = $possible.FullName } else { throw "Binary not found in archive." }
    }

    $Dest = Join-Path $BinDir "cuaca.exe"
    Write-Host "Installing to: $Dest"

    # Check write access; if needed, use Start-Process with sudo-like elevation? On Windows, copy may fail.
    # Instead, attempt copy; if fails, prompt to run as admin.
    try {
        Copy-Item -Path $BinarySrc -Destination $Dest -Force
    } catch {
        throw "Failed to copy to $BinDir. Please run PowerShell as Administrator or choose a writable -BinDir."
    }

    # Ensure bin dir is in PATH?
    $envPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
    if ($envPath -notlike "*$BinDir*") {
        Write-Warning "$BinDir is not in your USER PATH. You may want to add it."
    }

    Write-Host "cuaca v$Version installed successfully."
    Write-Host "Run 'cuaca --help' for usage."
} finally {
    if (Test-Path $TempDir) {
        Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
    }
}
