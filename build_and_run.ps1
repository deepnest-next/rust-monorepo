Write-Host "Building common API..."
Push-Location common
cargo build
Pop-Location

Write-Host "Building plugin_svg..."
Push-Location plugin_svg
cargo build --target-dir ../target_svg
Pop-Location

Write-Host "Building plugin_math..."
Push-Location plugin_math
cargo build --target-dir ../target_math
Pop-Location

Write-Host "Building and running main..."
Push-Location main
cargo run
Pop-Location
