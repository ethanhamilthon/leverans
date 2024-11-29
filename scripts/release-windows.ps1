param (
    [string]$Version
)

if (-not $Version) {
    Write-Host "Usage: .\script.ps1 <version>"
    exit 1
}

$binPath = "bin\$Version"
New-Item -ItemType Directory -Force -Path $binPath | Out-Null

$targets = @(
    "x86_64-pc-windows-msvc",
)

foreach ($target in $targets) {
    Write-Host "Building for target: $target"
    
    $env:CARGO_TARGET_DIR = "$binPath"
    $env:LEV_VERSION = $Version

    & cross build --release --target $target -p lev
}

Write-Host "Done"
