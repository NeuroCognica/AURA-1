try {
  $resp = Invoke-RestMethod -Uri 'http://127.0.0.1:8080/append_log' -Method Post -ContentType 'application/json' -Body '{"speaker":"tester","content":"api-test-3"}'
  Write-Host "append raw: $resp"
  if ($resp -is [string]) { try { $j = $resp | ConvertFrom-Json } catch { $j = $null } } else { $j = $resp }
  if ($j -ne $null) { $id = $j.id } else { $id = ($resp -replace '[^0-9]','') }
  Write-Host "id=$id"
  $root = Invoke-RestMethod 'http://127.0.0.1:8080/mmr_root'
  Write-Host "mmr_root: $root"
  $prove = Invoke-RestMethod "http://127.0.0.1:8080/prove/$id"
  Write-Host "prove: $prove"
  $snap = Invoke-RestMethod -Uri 'http://127.0.0.1:8080/snapshot' -Method Post -ContentType 'application/json' -Body '{"path":"data/checkpoint_from_api2"}'
  Write-Host "snapshot: $snap"
} catch {
  Write-Host "error: $($_.Exception.Message)"
}
