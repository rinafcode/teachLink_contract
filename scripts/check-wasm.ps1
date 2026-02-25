# Verify the teachlink contract compiles to WASM (no host DLL).
# Use this on Windows when you don't have Visual Studio Build Tools,
# to avoid the MinGW "export ordinal too large" error when running cargo test.
# Full tests run in CI (Linux) or with VS Build Tools installed.
Set-Location $PSScriptRoot\..

$target = "wasm32-unknown-unknown"
Write-Host "[*] Building teachlink-contract for $target ..." -ForegroundColor Cyan
cargo build -p teachlink-contract --target $target
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] WASM build succeeded. Contract is ready for deployment." -ForegroundColor Green
} else {
    Write-Host "[FAIL] Build failed." -ForegroundColor Red
    exit 1
}
