@echo off
REM Wrapper to launch AURA start_all.ps1 in PowerShell (keeps a visible console)
set SCRIPT_DIR=%~dp0
pushd %SCRIPT_DIR%
powershell -NoProfile -ExecutionPolicy Bypass -Command "& '%SCRIPT_DIR%start_all.ps1'"
popd
pause