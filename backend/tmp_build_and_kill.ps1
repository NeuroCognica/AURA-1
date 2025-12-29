$ErrorActionPreference = "Continue"
cd 'C:\AURA-1\backend'
Write-Output 'Starting cargo build --features persistence (30s timeout)'
$p = Start-Process -FilePath 'cargo' -ArgumentList 'build','--features','persistence' -NoNewWindow -PassThru -RedirectStandardOutput 'cargo-out.txt' -RedirectStandardError 'cargo-err.txt'
if(-not (Wait-Process -Id $p.Id -Timeout 30)) {
    Write-Output 'timed out, killing'
    Stop-Process -Id $p.Id -Force
}
Write-Output '--- STDOUT ---'
Get-Content cargo-out.txt -ErrorAction SilentlyContinue
Write-Output '--- STDERR ---'
Get-Content cargo-err.txt -ErrorAction SilentlyContinue
