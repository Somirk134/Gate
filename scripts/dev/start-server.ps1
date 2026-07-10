param(
    [string]$Addr = $env:GATE_SERVER_ADDR,
    [string]$Token = $env:GATE_AUTH_TOKEN,
    [switch]$Release
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($Addr)) {
    $Addr = "127.0.0.1:7000"
}

if ([string]::IsNullOrWhiteSpace($Token)) {
    $Token = "gate-alpha-token"
}

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $RepoRoot

$env:GATE_SERVER_ADDR = $Addr
$env:GATE_AUTH_TOKEN = $Token

Write-Host ""
Write-Host "Gate Server local startup"
Write-Host "  Address : $Addr"
# 终端输出不得回显实际口令，避免复制日志或截图时泄露凭据。
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
