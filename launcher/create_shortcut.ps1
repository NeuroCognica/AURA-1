$desktop = Join-Path $env:USERPROFILE 'Desktop'
$path = Join-Path $desktop 'AURA Launcher.lnk'
$W = New-Object -ComObject WScript.Shell
$S = $W.CreateShortcut($path)

# Point shortcut at the packaged Electron binary for dev (node_modules electron.exe)
$electronExe = Join-Path $PSScriptRoot 'node_modules\electron\dist\electron.exe'
if (-Not (Test-Path $electronExe)) {
	Write-Error "Electron executable not found at $electronExe. Run 'npm install' in launcher first."
	exit 1
}

$S.TargetPath = $electronExe
$S.Arguments = '.'
$S.WorkingDirectory = 'C:\AURA-1\launcher'
$S.Description = 'AURA Launcher (Electron)'
$S.IconLocation = Join-Path $PSScriptRoot 'assets\icon.ico'
$S.Save()
Write-Output "Shortcut created: $path"
