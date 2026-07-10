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
    throw "GATE_AUTH_TOKEN or -Token is required."
}

# 开发入口与生产二进制使用相同的最低凭据强度，避免弱配置迁移到部署环境。
if ($Token.Trim().Length -lt 16 -or $Token -in @("gate-alpha-token", "change-me", "changeme", "replace-me", "replace-with-a-long-random-token")) {
    throw "Token must contain at least 16 characters and must not use a known default."
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
