$i=0
while ($i -lt 20) {
  try {
    Invoke-RestMethod 'http://127.0.0.1:8080/health' -UseBasicParsing -TimeoutSec 2 | Write-Output
    exit 0
  } catch {
    Start-Sleep -Seconds 1
    $i = $i + 1
  }
}
Write-Output 'health-checks: failed'
exit 1
