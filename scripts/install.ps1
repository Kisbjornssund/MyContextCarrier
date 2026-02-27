# ContextGenOS installer for Windows
# Usage: irm https://contextgenos.dev/install.ps1 | iex

$ErrorActionPreference = 'Stop'

$Repo       = "Kisbjornssund/ContextGenOS"
$Binary     = "contextgenos.exe"
$Artifact   = "contextgenos-windows-x86_64.exe"
$InstallDir = "$env:LOCALAPPDATA\Programs\ContextGenOS"

function Write-Step($msg) { Write-Host "==> $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "error: $msg" -ForegroundColor Red; exit 1 }

# ── Architecture check ────────────────────────────────────────────────────────

if ($env:PROCESSOR_ARCHITECTURE -ne "AMD64") {
    Write-Fail ("Unsupported architecture: $env:PROCESSOR_ARCHITECTURE. " +
                "Only x86_64 (AMD64) is supported. " +
                "Download manually from https://github.com/$Repo/releases")
}

# ── Fetch latest release version ─────────────────────────────────────────────

Write-Step "Fetching latest release..."
try {
    $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    $version = $release.tag_name
} catch {
    Write-Fail "Could not fetch latest release. Check https://github.com/$Repo/releases"
}

if (-not $version) {
    Write-Fail "Could not determine latest release version."
}

# ── Download binary and checksum ─────────────────────────────────────────────

$baseUrl = "https://github.com/$Repo/releases/download/$version"
$tmpDir  = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $tmpDir | Out-Null

Write-Step "Downloading ContextGenOS $version..."
Invoke-WebRequest "$baseUrl/$Artifact"        -OutFile "$tmpDir\$Binary"        -UseBasicParsing
Invoke-WebRequest "$baseUrl/$Artifact.sha256" -OutFile "$tmpDir\$Artifact.sha256" -UseBasicParsing

# ── Verify checksum ───────────────────────────────────────────────────────────

Write-Step "Verifying checksum..."
$expected = ((Get-Content "$tmpDir\$Artifact.sha256") -split '\s+')[0].ToLower()
$actual   = (Get-FileHash "$tmpDir\$Binary" -Algorithm SHA256).Hash.ToLower()

if ($expected -ne $actual) {
    Remove-Item -Recurse -Force $tmpDir
    Write-Fail ("Checksum mismatch — download may be corrupt.`n" +
                "  Expected: $expected`n" +
                "  Got:      $actual")
}

# ── Install ───────────────────────────────────────────────────────────────────

Write-Step "Installing to $InstallDir..."
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}
Copy-Item "$tmpDir\$Binary" "$InstallDir\$Binary" -Force
Remove-Item -Recurse -Force $tmpDir

# ── Add to user PATH if missing ───────────────────────────────────────────────

$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$InstallDir*") {
    Write-Step "Adding $InstallDir to PATH..."
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$InstallDir", "User")
    $env:PATH += ";$InstallDir"
    Write-Host "  Restart your terminal for PATH changes to take effect." -ForegroundColor Yellow
}

# ── Done ──────────────────────────────────────────────────────────────────────

Write-Step "ContextGenOS $version installed."
& "$InstallDir\$Binary" --version

Write-Host @"

Next steps:
  contextgenos init       Set up your local context store
  contextgenos --help     See all commands
  contextgenos mcp serve  Start the MCP server for AI tool integration

Documentation: https://docs.contextgenos.dev
"@ -ForegroundColor Cyan
