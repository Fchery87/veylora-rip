#Requires -Version 7.2
<#
.SYNOPSIS
Builds a release directory and zip for one version, then verifies what it built.

.DESCRIPTION
Fails closed: the license check must pass, nothing from development tooling is staged, and the
result must pass verify-release.ps1 before the zip and its .sha256 are written. Only the headless
veylora-cli is packaged. The desktop app is not, because the installer technology is undecided
(see installer/README.md).
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
Invoke-Checked 'license check' { & $hostExe -NoProfile -File (Join-Path $PSScriptRoot 'check-licenses.ps1') -Root $root }
Invoke-Checked 'release build' { & cargo build --release --locked -p veylora-cli --manifest-path (Join-Path $root 'Cargo.toml') }

$name = "veylora-rip-$Version"
$stage = Join-Path $OutDir $name
if (Test-Path $stage) { throw "$stage already exists; releases are never overwritten" }
New-Item $stage -ItemType Directory | Out-Null
$cli = Join-Path $stage 'veylora-cli.exe'
Copy-Item (Join-Path $root 'target/release/veylora-cli.exe') $cli
Copy-Item (Join-Path $root 'THIRD_PARTY_NOTICES.md') $stage
New-Item (Join-Path $stage 'docs') -ItemType Directory | Out-Null
Copy-Item (Join-Path $root 'docs/job-format.md') (Join-Path $stage 'docs')

Invoke-Checked 'manifest' { & $cli release manifest --dir $stage --version $Version --max-data-schema $MaxDataSchema --smoke-test veylora-cli.exe --smoke-arg --version | Out-Null }
Invoke-Checked 'verification' { & $hostExe -NoProfile -File (Join-Path $PSScriptRoot 'verify-release.ps1') -Path $stage -Cli $cli -RunSmokeTest }

$zip = Join-Path $OutDir "$name.zip"
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
$hash = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -LiteralPath "$zip.sha256" "$hash  $name.zip" -NoNewline
Invoke-Checked 'zip verification' { & $hostExe -NoProfile -File (Join-Path $PSScriptRoot 'verify-release.ps1') -Path $zip -Cli $cli -RunSmokeTest }
Write-Host "Packaged $zip (sha256 $hash). Unsigned: publish the hash separately."
