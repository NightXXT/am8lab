@echo off
setlocal
where cargo >nul 2>nul
if errorlevel 1 (
  echo Instale Rust e Visual Studio Build Tools com C++ primeiro.
  exit /b 1
)
pushd "%~dp0src-tauri"
cargo %*
set "AM8_BUILD_RESULT=%ERRORLEVEL%"
popd
exit /b %AM8_BUILD_RESULT%
