param(
    [switch]$NoOllamaStart
)

function Start-DetachedWindow($title, $cmd) {
    Write-Host "Launching window: $title"
    if ($title) {
        $arg = "title $title && $cmd"
    } else {
        $arg = $cmd
    }
    Start-Process cmd.exe -ArgumentList '/k', $arg -WindowStyle Normal
}

Write-Host "AURA Launcher - starting sequence: Ollama -> Backend -> Frontend"

# Step 1: Ollama
$ollama_url = $env:OLLAMA_URL -or 'http://127.0.0.1:11434'
try {
    $r = Invoke-RestMethod -Uri "$ollama_url/api/ping" -Method Get -TimeoutSec 2 -ErrorAction Stop
    Write-Host "Ollama already responding at $ollama_url"
} catch {
    Write-Host "AURA Launcher - starting sequence: Ollama -> Backend -> Frontend"

    # Step 1: Ollama (best-effort)
    $ollama_url = $env:OLLAMA_URL -or 'http://127.0.0.1:11434'
    try {
        $r = Invoke-RestMethod -Uri "$ollama_url/api/ping" -Method Get -TimeoutSec 2 -ErrorAction Stop
        Write-Host "Ollama already responding at $ollama_url"
    } catch {
        if ($NoOllamaStart) {
            Write-Warning "Ollama not responding and auto-start disabled. Continuing."
        } else {
            $ollamaCmd = (Get-Command ollama -ErrorAction SilentlyContinue).Path
            if ($ollamaCmd) {
                Write-Host "Starting Ollama in a new window"
                Start-Process cmd.exe -ArgumentList '/k', "title Ollama && cd /d $PWD && `"$ollamaCmd`" serve" -WindowStyle Normal
                Start-Sleep -Seconds 2
            } else {
                Write-Warning "'ollama' not found in PATH. Please install or start Ollama manually."
            }
        }
    }

    # Step 2: Backend
    Write-Host "Starting backend in a new window"
    $backendCmd = "title AURA Backend && cd /d $PWD\backend && set RUST_LOG=info && cargo run --bin aura-backend"
    Start-Process cmd.exe -ArgumentList '/k', $backendCmd -WindowStyle Normal

    Write-Host "Waiting for backend /health (127.0.0.1:8080)..."
    $backendAlive = $false
    for ($i = 0; $i -lt 60; $i++) {
        try {
            $h = Invoke-RestMethod -Uri 'http://127.0.0.1:8080/health' -Method Get -TimeoutSec 2 -ErrorAction Stop
            if ($h -eq 'ok') { $backendAlive = $true; break }
        } catch { }
        Start-Sleep -Seconds 1
        Write-Host -NoNewline '.'
    }
    if (-not $backendAlive) {
        Write-Error "Backend did not respond on http://127.0.0.1:8080/health within timeout. Check the backend console."
        exit 2
    }
    Write-Host "`nBackend is up.`n"

    # Step 3: Frontend build (non-blocking)
    if (Test-Path "$PWD\frontend\package.json") {
        Write-Host "Starting frontend build in a new window"
        $frontendCmd = "title AURA Frontend && cd /d $PWD\frontend && npm ci && npm run build"
        Start-Process cmd.exe -ArgumentList '/k', $frontendCmd -WindowStyle Normal
        Write-Host "Frontend build started in new window. Build output: $PWD\frontend\build"
    } else {
        Write-Host "No frontend/package.json found; skipping frontend step."
    }

    Write-Host "All steps initiated. Watch the separate windows for live logs."
}
