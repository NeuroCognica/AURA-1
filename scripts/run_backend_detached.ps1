Param(
    [int]$TimeoutSeconds = 30,
    [string]$HealthUrl = 'http://127.0.0.1:3000/health',
    [string]$BackendDir = (Join-Path $PSScriptRoot '..\backend'),
    [string]$LogFile = (Join-Path $PSScriptRoot 'backend_run.log')
)

$log = @()
$timestamp = (Get-Date).ToString('o')
$log += "Starting backend from $BackendDir at $timestamp"

# Build the Start-Process argument list carefully
$cmd = "Set-Location -LiteralPath '$BackendDir'; cargo run"
$proc = Start-Process powershell -ArgumentList '-NoExit', '-Command', $cmd -PassThru
$log += "Started process Id=$($proc.Id)"

$sw = [diagnostics.stopwatch]::StartNew()
$success = $false
while ($sw.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
    Start-Sleep -Seconds 1
    try {
        $r = Invoke-RestMethod -Uri $HealthUrl -UseBasicParsing -TimeoutSec 2
        $log += "Health check success at $([math]::Round($sw.Elapsed.TotalSeconds,1)) sec: $r"
        $success = $true
        break
    } catch {
        $log += "Health check attempt at $([math]::Round($sw.Elapsed.TotalSeconds,1)) sec failed: $($_.Exception.Message)"
    }
}

if (-not $success) {
    $log += "Timeout reached ($TimeoutSeconds s); attempting to kill process Id=$($proc.Id)"
    try {
        Stop-Process -Id $proc.Id -Force -ErrorAction Stop
        $log += "Process killed (Id=$($proc.Id))."
    } catch {
        $log += "Failed to kill process Id=$($proc.Id): $($_.Exception.Message)"
    }

    # Collect debug info
    try {
        $listeners = Get-NetTCPConnection -LocalPort 3000 -ErrorAction SilentlyContinue
        $log += "Get-NetTCPConnection:"
        $log += ($listeners | Out-String)
    } catch {
        $log += "Get-NetTCPConnection failed: $($_.Exception.Message)"
    }
    try {
        $ps = Get-Process -Id $proc.Id -ErrorAction SilentlyContinue
        $log += "Get-Process:"
        $log += ($ps | Out-String)
    } catch {
        $log += "Get-Process failed: $($_.Exception.Message)"
    }
    try {
        $net = & netstat -ano | Select-String ':3000' -ErrorAction SilentlyContinue
        $log += "netstat -ano (port 3000 lines):"
        $log += ($net | Out-String)
    } catch {
        $log += "netstat failed: $($_.Exception.Message)"
    }
}

$log += "Diagnostic snapshot at $(Get-Date -Format o):"
$log | Out-File -FilePath $LogFile -Encoding utf8
$log | ForEach-Object { Write-Output $_ }

if ($success) { exit 0 } else { exit 1 }
