param(
    [string]$Archetype = "sentinel",
    [string]$BackendUrl = $(if ($env:BACKEND_URL) { $env:BACKEND_URL } else { '' }),
    [switch]$DryRun
)

$payload = @{ archetype = $Archetype }
$body = $payload | ConvertTo-Json -Depth 5

if ($DryRun) {
    Write-Host "DRY RUN: POST $BackendUrl/api/archetype/activate"
    Write-Host $body
    exit 0
}

# If BACKEND_URL not provided, probe common ports
if ([string]::IsNullOrWhiteSpace($BackendUrl)) {
    $candidates = @('http://127.0.0.1:8080','http://localhost:8080','http://127.0.0.1:3000','http://localhost:3000')
    $found = $false
    foreach ($cand in $candidates) {
        try {
            Write-Host "Probing $cand/health"
            $h = Invoke-RestMethod -Uri ("$cand/health") -Method Get -TimeoutSec 2 -ErrorAction Stop
            if ($h -eq 'ok') { $BackendUrl = $cand; $found = $true; break }
        } catch {
            Write-Host -NoNewline '.'
            Start-Sleep -Milliseconds 250
        }
    }
    if (-not $found) {
        Write-Error "No backend detected on common ports. Set BACKEND_URL or start the backend."
        exit 2
    }
}

Write-Host "POSTing to: $BackendUrl/api/archetype/activate"
try {
    $resp = Invoke-RestMethod -Uri ("$BackendUrl/api/archetype/activate") -Method Post -ContentType 'application/json' -Body $body -TimeoutSec 10
    Write-Host "Response:`n" ($resp | ConvertTo-Json -Depth 5)
} catch {
    Write-Error "Request failed: $($_.Exception.Message)"
    Write-Error "Tried BackendUrl: $BackendUrl"
    exit 1
}
