#Requires -Version 5.1
# webrok Windows installer.
# Usage:  irm https://webrok.vercel.app/install.ps1 | iex
#
# Works in both Full Language Mode and Constrained Language Mode (CLM).

$ErrorActionPreference = 'Stop'
$BaseUrl = 'https://webrok.vercel.app'
$InstallDir = Join-Path $env:USERPROFILE '.webrok\bin'
$ExePath = Join-Path $InstallDir 'webrok.exe'

Write-Host "webrok installer" -ForegroundColor Cyan
Write-Host "  install dir : $InstallDir"
Write-Host "  download    : $BaseUrl/webrok.exe"

# 1. Create install dir
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

# 2. Download binary
Write-Host "`nDownloading..."
try {
    Invoke-WebRequest -Uri "$BaseUrl/webrok.exe" -OutFile $ExePath -UseBasicParsing
} catch {
    Write-Error "Download failed: $_"
    return
}
$size = (Get-Item $ExePath).Length
Write-Host ("  saved {0}  ({1:N0} bytes)" -f $ExePath, $size)

# 3. Add install dir to the user's PATH.
# Use registry cmdlets + reg.exe so this works under Constrained Language Mode
# (where [Environment]::SetEnvironmentVariable() is blocked).
$regPath = 'HKCU:\Environment'
$existing = ''
try {
    $existing = (Get-ItemProperty -LiteralPath $regPath -Name Path -ErrorAction Stop).Path
} catch {
    $existing = ''
}
if (-not $existing) { $existing = '' }

$parts = @()
foreach ($p in ($existing -split ';')) {
    if ($p -and ($p.Trim() -ne '')) { $parts += $p }
}

$needAdd = $true
foreach ($p in $parts) {
    if ($p.TrimEnd('\') -ieq $InstallDir.TrimEnd('\')) { $needAdd = $false; break }
}

if ($needAdd) {
    $parts += $InstallDir
    $new = ($parts -join ';')
    # reg.exe preserves REG_EXPAND_SZ type and avoids setx's 1024-char limit.
    & reg add 'HKCU\Environment' /v Path /t REG_EXPAND_SZ /d $new /f | Out-Null
    Write-Host "`nAdded to user PATH:" -ForegroundColor Green
    Write-Host "  $InstallDir"
    Write-Host "  (open a new terminal to pick it up)"
} else {
    Write-Host "`nPATH already contains $InstallDir"
}

# 4. Make it available in the CURRENT shell so you can run `webrok` right away.
if ($env:Path -notlike "*$InstallDir*") {
    $env:Path = "$env:Path;$InstallDir"
}

# 5. Sanity check
try {
    & $ExePath --version | Out-Null
} catch {
    Write-Host "Note: couldn't invoke webrok.exe directly — the binary is at:"
    Write-Host "  $ExePath"
}

Write-Host "`nInstalled." -ForegroundColor Green

# 6. Interactive login — prompt for a token. Blank input skips.
Write-Host ""
Write-Host "Get your token at $BaseUrl/dashboard  (sign up if you don't have one)."
$tok = Read-Host "Paste your webrok token (or press Enter to skip)"
$tok = $tok.Trim()

if ($tok) {
    try {
        & $ExePath --server $BaseUrl login --token $tok
    } catch {
        Write-Warning "login failed: $_"
    }
} else {
    Write-Host "Skipped. You can log in later with:"
    Write-Host "  webrok --server $BaseUrl login"
}

Write-Host ""
Write-Host "Done. Open a NEW terminal, then run:"
Write-Host "  webrok http <port> --name <app-name>"
