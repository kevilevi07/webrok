#Requires -Version 5.1
# webrok Windows installer.
# Usage:  irm https://webrok.vercel.app/install.ps1 | iex

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
    exit 1
}
$size = (Get-Item $ExePath).Length
Write-Host ("  saved {0}  ({1:N0} bytes)" -f $ExePath, $size)

# 3. Add install dir to the user's PATH (persists across sessions)
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not $userPath) { $userPath = '' }
$parts = $userPath -split ';' | Where-Object { $_ -ne '' }
if ($parts -notcontains $InstallDir) {
    $newPath = (@($parts + $InstallDir) -join ';').TrimStart(';')
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    Write-Host "`nAdded to user PATH:" -ForegroundColor Green
    Write-Host "  $InstallDir"
    Write-Host "  (open a new terminal to pick it up)"
} else {
    Write-Host "`nPATH already contains $InstallDir"
}

# 4. Also set it for the CURRENT shell so the user can run it right away
if ($env:Path -notlike "*$InstallDir*") {
    $env:Path = "$env:Path;$InstallDir"
}

# 5. Set default server
try {
    & $ExePath --server $BaseUrl status | Out-Null
} catch { }

Write-Host "`nInstalled." -ForegroundColor Green
Write-Host "Next steps:"
Write-Host "  1. Visit $BaseUrl/dashboard to copy your token"
Write-Host "  2. Run:  webrok --server $BaseUrl login"
Write-Host "  3. Run:  webrok http <port> --name <app-name>"
