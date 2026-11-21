#Requires -Version 7.2
<#
.SYNOPSIS
Installation test for a release zip, meant to run on a clean Windows machine (for example inside
Windows Sandbox) where no development tool is installed.

.DESCRIPTION
Verifies the zip against its published hash, installs it into a scratch install root, then
exercises the product with PATH reduced to the Windows directories so nothing can be found from a
development toolchain: version, a diagnostic target render, install state, and (with -UpdateZip)
an update, a refused downgrade, a rollback and recovery. Any failure exits non-zero.

Passing this script on a development machine proves the steps work, not that a clean machine does.
Gate F needs a recorded run on a clean machine.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Zip,
    [Parameter(Mandatory)][string]$ExpectedSha256,
    [string]$UpdateZip,
    [string]$UpdateSha256
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$verifier = Join-Path $PSScriptRoot '../../scripts/verify-release.ps1'
$hostExe = (Get-Process -Id $PID).Path
$work = Join-Path ([IO.Path]::GetTempPath()) ('veylora-clean-' + [guid]::NewGuid())
$steps = 0

function Step([string]$Name, [scriptblock]$Block) {
    & $Block
    $script:steps++
    Write-Host "PASS $Name"
}
function Expect-Failure([string]$Name, [string]$Message, [scriptblock]$Block) {
    $out = (& $Block 2>&1 | Out-String)
    if ($LASTEXITCODE -eq 0) { throw "$Name unexpectedly succeeded" }
    if ($out -notmatch [regex]::Escape($Message)) { throw "$Name failed for another reason: $out" }
    $script:steps++
    Write-Host "PASS $Name"
}
function Unpack([string]$ZipPath, [string]$Hash, [string]$Name) {
    & $hostExe -NoProfile -File $verifier -Path $ZipPath -ExpectedSha256 $Hash -RunSmokeTest | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "$Name did not verify" }
    $dest = Join-Path $work $Name
    Expand-Archive -LiteralPath $ZipPath -DestinationPath $dest
    $dest
}

try {
    New-Item $work -ItemType Directory | Out-Null
    $release = Unpack $Zip $ExpectedSha256 'release'
    $cli = Join-Path $release 'veylora-cli.exe'
    $root = Join-Path $work 'install'
    # Nothing from a development toolchain may be reachable from here on.
    $env:PATH = "$env:SystemRoot\System32;$env:SystemRoot"
    foreach ($tool in 'cargo', 'rustc', 'node', 'pnpm', 'npm') {
        if (Get-Command $tool -ErrorAction SilentlyContinue) { throw "$tool is still on PATH; this is not a clean environment" }
    }

    Step 'the shipped program starts' { & $cli --version | Out-Null; if ($LASTEXITCODE -ne 0) { throw 'no start' } }
    if (Test-Path (Join-Path $release 'veylora-app.exe')) {
        Step 'the desktop app starts without a window (--version)' {
            $p = Start-Process -FilePath (Join-Path $release 'veylora-app.exe') -ArgumentList '--version' -Wait -PassThru -WindowStyle Hidden
            if ($p.ExitCode -ne 0) { throw "exit code $($p.ExitCode)" }
        }
        Step 'the desktop app shows its embedded pages' {
            & $hostExe -NoProfile -File (Join-Path $PSScriptRoot 'app-launch-check.ps1') -Exe (Join-Path $release 'veylora-app.exe') | Out-Host
            if ($LASTEXITCODE -ne 0) { throw 'the app did not show its pages' }
        }
        # The app draws its UI with the WebView2 runtime, which is not shipped. Windows 11 provides it.
        Step 'the WebView2 runtime is installed' {
            $keys = 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
            $found = @($keys | Where-Object { $v = Get-ItemProperty -Path $_ -ErrorAction SilentlyContinue; $v -and $v.PSObject.Properties['pv'] -and $v.pv })
            if (-not $found) { throw 'the WebView2 runtime is missing; the desktop app cannot show its window without it' }
            Write-Host ("  WebView2 runtime " + (Get-ItemProperty -Path $found[0]).pv)
        }
    }
    Step 'install into a new root' {
        & $cli release install --install-root $root --from $release --no-workspace | Out-Null
        if ($LASTEXITCODE -ne 0) { throw 'install failed' }
    }
    $installed = (& $cli release status --install-root $root | ConvertFrom-Json)
    Step 'the installed release verifies in place' {
        if (-not $installed.current_verified) { throw 'not verified' }
    }
    $installedCli = Join-Path $root "releases/$($installed.current_version)/veylora-cli.exe"
    Step 'a diagnostic target renders from the installed copy' {
        $svg = Join-Path $work 'target.svg'
        & $installedCli --workspace (Join-Path $work "scratch-workspace") target generate --kind channel_identity --width-mm 100 --height-mm 60 --output $svg
        if ($LASTEXITCODE -ne 0 -or -not (Test-Path $svg) -or (Get-Item $svg).Length -eq 0) { throw 'no target' }
    }
    Expect-Failure 'reinstalling the same version is refused' 'not newer' {
        & $cli release install --install-root $root --from $release --no-workspace
    }
    Expect-Failure 'an update without a workspace decision is refused' '--workspace' {
        & $cli release install --install-root $root --from $release
    }

    if ($UpdateZip) {
        $update = Unpack $UpdateZip $UpdateSha256 'update'
        Step 'update to the newer release' {
            & $cli release install --install-root $root --from $update --no-workspace | Out-Null
            if ($LASTEXITCODE -ne 0) { throw 'update failed' }
        }
        Step 'roll back to the first release' {
            & $cli release rollback --install-root $root --no-workspace | Out-Null
            if ($LASTEXITCODE -ne 0) { throw 'rollback failed' }
            $now = (& $cli release status --install-root $root | ConvertFrom-Json)
            if ($now.current_version -ne $installed.current_version) { throw 'wrong release after rollback' }
        }
    }
    Step 'recovery on a healthy install changes nothing' {
        $r = (& $cli release recover --install-root $root | ConvertFrom-Json)
        if ($r.abandoned_update -or $r.fell_back_from_damaged_current) { throw 'recovery changed a healthy install' }
    }
    Write-Host "$steps installation checks passed. This is not a clean-machine record unless it ran on one."
} finally {
    if (Test-Path $work) { Remove-Item $work -Recurse -Force }
}
