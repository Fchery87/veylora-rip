#Requires -Version 7.2
<#
.SYNOPSIS
Builds a release directory and zip for one version, then verifies what it built.

.DESCRIPTION
Fails closed: the license check must pass, nothing from development tooling is staged, and the
result must pass verify-release.ps1 before the zip and its .sha256 are written. A release holds the
headless veylora-cli and the desktop app veylora-app.exe (the installer is a zip with the scripts in
installer/README.md). The app is the Tauri shell with the built frontend embedded, so no other file
is needed beside it except the WebView2 runtime, which Windows 11 provides and which is not shipped.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$')][string]$Version,
    [string]$OutDir,
    # The newest workspace data schema this build reads (see veylora_persistence::current_schema_version).
    [Parameter(Mandatory)][int]$MaxDataSchema
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path $PSScriptRoot -Parent
if (-not $OutDir) { $OutDir = Join-Path $root 'dist' }

function Invoke-Checked([string]$What, [scriptblock]$Block) {
    & $Block
    if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

$hostExe = (Get-Process -Id $PID).Path
# Fails closed: the program being packaged must have every linked component reviewed with its notice.
foreach ($program in 'veylora-cli', 'veylora-app') {
    Invoke-Checked "license check for $program" { & $hostExe -NoProfile -File (Join-Path $PSScriptRoot 'check-licenses.ps1') -Root $root -Package $program }
}
# The C runtime is linked statically so a clean machine needs no Visual C++ redistributable.
$env:RUSTFLAGS = '-C target-feature=+crt-static'
# The desktop app embeds the built frontend at compile time, so build that first. Without
# tauri/custom-protocol a release app would try to load the development server instead of its
# embedded pages, so the feature is part of the build, not optional.
Invoke-Checked 'frontend build' { & pnpm --dir (Join-Path $root 'app') run build }
Invoke-Checked 'release build' { & cargo build --release --locked -p veylora-cli -p veylora-app --features tauri/custom-protocol --manifest-path (Join-Path $root 'Cargo.toml') }
$built = Join-Path $root 'target/release/veylora-cli.exe'
$builtApp = Join-Path $root 'target/release/veylora-app.exe'
foreach ($exe in $built, $builtApp) {
    $imports = [Text.Encoding]::Latin1.GetString([IO.File]::ReadAllBytes($exe))
    if ($imports -match '(?i)(vcruntime|msvcp)\d*\.dll') { throw "$(Split-Path $exe -Leaf) still imports the Visual C++ runtime" }
}

$name = "veylora-rip-$Version"
$stage = Join-Path $OutDir $name
if (Test-Path $stage) { throw "$stage already exists; releases are never overwritten" }
New-Item $stage -ItemType Directory | Out-Null
$cli = Join-Path $stage 'veylora-cli.exe'
Copy-Item $built $cli
Copy-Item $builtApp (Join-Path $stage 'veylora-app.exe')
Copy-Item (Join-Path $root 'THIRD_PARTY_NOTICES.md') $stage
# The Rust standard library's own third-party notices. They must be the ones that were reviewed.
$inventoryText = Get-Content (Join-Path $root 'docs/dependency-inventory.md') -Raw
$reviewed = [regex]::Match($inventoryText, '"rustStdCopyrightSha256":\s*"([0-9A-Fa-f]{64})"').Groups[1].Value
$stdNotice = Join-Path ((& rustc --print sysroot).Trim()) 'share/doc/rust/COPYRIGHT-library.html'
if (-not $reviewed -or (Get-FileHash $stdNotice -Algorithm SHA256).Hash -ne $reviewed.ToUpperInvariant()) {
    throw "the toolchain's Rust standard library notices are not the ones that were reviewed; regenerate with scripts/generate-package-notices.ps1"
}
New-Item (Join-Path $stage 'licenses') -ItemType Directory | Out-Null
Copy-Item $stdNotice (Join-Path $stage 'licenses/rust-std-COPYRIGHT.html')
New-Item (Join-Path $stage 'docs') -ItemType Directory | Out-Null
Copy-Item (Join-Path $root 'docs/job-format.md') (Join-Path $stage 'docs')

# A release app built without tauri/custom-protocol would load the development server; only the
# running app shows which (the address is plain text in a correct build too).
Invoke-Checked 'app launch check' { & $hostExe -NoProfile -File (Join-Path $root 'installer/windows/app-launch-check.ps1') -Exe (Join-Path $stage 'veylora-app.exe') }
Invoke-Checked 'manifest' { & $cli release manifest --dir $stage --version $Version --max-data-schema $MaxDataSchema --smoke-test veylora-cli.exe --smoke-arg --version | Out-Null }
Invoke-Checked 'verification' { & $hostExe -NoProfile -File (Join-Path $PSScriptRoot 'verify-release.ps1') -Path $stage -Cli $cli -RunSmokeTest }

$zip = Join-Path $OutDir "$name.zip"
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
$hash = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -LiteralPath "$zip.sha256" "$hash  $name.zip" -NoNewline
Invoke-Checked 'zip verification' { & $hostExe -NoProfile -File (Join-Path $PSScriptRoot 'verify-release.ps1') -Path $zip -Cli $cli -RunSmokeTest }
Write-Host "Packaged $zip (sha256 $hash). Unsigned: publish the hash separately."
