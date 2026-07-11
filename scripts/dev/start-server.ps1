param(
    [string]$Addr = $env:GATE_SERVER_ADDR,
    [string]$Token = $env:GATE_AUTH_TOKEN,
    [switch]$Release
)

$ErrorActionPreference = "Stop"

function Import-DotEnv {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return
    }

    Get-Content -LiteralPath $Path | ForEach-Object {
        $line = $_.Trim()
        if ($line -eq "" -or $line.StartsWith("#")) {
            return
        }

        $eq = $line.IndexOf("=")
        if ($eq -lt 1) {
            return
        }

        $key = $line.Substring(0, $eq).Trim()
        $value = $line.Substring($eq + 1).Trim()
        if (
            ($value.StartsWith('"') -and $value.EndsWith('"')) -or
            ($value.StartsWith("'") -and $value.EndsWith("'"))
        ) {
            $value = $value.Substring(1, $value.Length - 2)
        }

        if ($key -and $null -eq [System.Environment]::GetEnvironmentVariable($key, "Process")) {
            [System.Environment]::SetEnvironmentVariable($key, $value, "Process")
        }
    }
}

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Import-DotEnv -Path (Join-Path $RepoRoot ".env")

if ([string]::IsNullOrWhiteSpace($Addr)) {
    $Addr = $env:GATE_SERVER_ADDR
}
if ([string]::IsNullOrWhiteSpace($Token)) {
    $Token = $env:GATE_AUTH_TOKEN
}

if ([string]::IsNullOrWhiteSpace($Addr)) {
    $Addr = "127.0.0.1:7000"
}

if ([string]::IsNullOrWhiteSpace($Token)) {
    throw "GATE_AUTH_TOKEN or -Token is required. Set it in .env or the shell environment."
}

$knownWeakTokens = @(
    "gate-alpha-token",
    "change-me",
    "changeme",
    "replace-me",
    "replace-with-a-long-random-token"
)
$tokenIsWeak = ($Token.Trim().Length -lt 16) -or ($knownWeakTokens -contains $Token)
if ($tokenIsWeak) {
    throw "Token must contain at least 16 characters and must not use a known default."
}

Set-Location $RepoRoot

$env:GATE_SERVER_ADDR = $Addr
$env:GATE_AUTH_TOKEN = $Token

Write-Host ""
Write-Host "Gate Server local startup"
Write-Host "  Address : $Addr"
Write-Host "  Token   : [configured]"
Write-Host "  Mode    : $(if ($Release) { 'release' } else { 'debug' })"
Write-Host ""
Write-Host "Keep this terminal open. Press Ctrl+C to stop the server."
Write-Host ""

$CargoArgs = @("run", "-p", "gate-server")
if ($Release) {
    $CargoArgs += "--release"
}

& cargo @CargoArgs
exit $LASTEXITCODE
