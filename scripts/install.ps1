# Installer for furl (https://github.com/ghimiresdp/furl-cli).
#
#   powershell -ExecutionPolicy ByPass -c "irm https://raw.githubusercontent.com/ghimiresdp/furl-cli/main/scripts/install.ps1 | iex"
#
# Env vars:
#   $env:FURL_VERSION      Install a specific tag (e.g. "v0.9.1") instead of latest.
#   $env:FURL_INSTALL_DIR  Where to put the binary (default: "$env:LOCALAPPDATA\furl\bin").

$ErrorActionPreference = "Stop"

$Repo = "ghimiresdp/furl-cli"
$InstallDir = if ($env:FURL_INSTALL_DIR) { $env:FURL_INSTALL_DIR } else { "$env:LOCALAPPDATA\furl\bin" }

function Write-Info($msg) { Write-Host "info: $msg" -ForegroundColor Cyan }
function Write-Warn($msg) { Write-Host "warn: $msg" -ForegroundColor Yellow }
function Die($msg) { Write-Host "error: $msg" -ForegroundColor Red; exit 1 }

$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { Die "unsupported architecture (32-bit Windows is not supported)" }
$Platform = "windows-$Arch"

# Finds the newest release whose tag looks like a furl-cli binary release
# (a bare "vX.Y.Z" tag, as opposed to this repo's separate, binary-less
# "furl-cli@X.Y.Z" / "furl-core@X.Y.Z" version tags).
function Get-LatestTag {
    $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases"
    $match = $releases | Where-Object { $_.tag_name -match '^v[0-9]' } | Select-Object -First 1
    if (-not $match) { Die "couldn't find a release to install; see https://github.com/$Repo/releases" }
    return $match.tag_name
}

$Tag = if ($env:FURL_VERSION) { $env:FURL_VERSION } else {
    Write-Info "looking up the latest release..."
    Get-LatestTag
}

$Asset = "furl-$Tag-$Platform.zip"
$Url = "https://github.com/$Repo/releases/download/$Tag/$Asset"

$TmpDir = Join-Path $env:TEMP ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $TmpDir | Out-Null

try {
    Write-Info "downloading furl $Tag for $Platform..."
    $ArchivePath = Join-Path $TmpDir $Asset
    try {
        Invoke-WebRequest -Uri $Url -OutFile $ArchivePath
    } catch {
        Die "download failed: $Url (does a release for $Platform exist for $Tag?)"
    }

    Expand-Archive -Path $ArchivePath -DestinationPath $TmpDir -Force

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item -Path (Join-Path $TmpDir "furl.exe") -Destination (Join-Path $InstallDir "furl.exe") -Force

    Write-Info "installed furl to $InstallDir\furl.exe"

    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (";$UserPath;" -notlike "*;$InstallDir;*") {
        Write-Warn "$InstallDir is not on your PATH"
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
        Write-Info "added $InstallDir to your user PATH (restart your terminal to pick it up)"
    }

    Write-Info "run 'furl --help' to get started"
} finally {
    Remove-Item -Path $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
}
