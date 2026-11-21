# Self-contained checks for verify-release.ps1. Needs a built veylora-cli (cargo build -p veylora-cli).
param([string]$Cli = (Join-Path $PSScriptRoot '../../target/debug/veylora-cli.exe'))
$ErrorActionPreference = 'Stop'
$verifier = Join-Path $PSScriptRoot '../verify-release.ps1'
if (-not (Test-Path $verifier)) { throw 'RED: verify-release.ps1 is missing' }
if (-not (Test-Path $Cli)) { throw "build veylora-cli first: $Cli not found" }
$hostExe = (Get-Process -Id $PID).Path
$work = Join-Path ([IO.Path]::GetTempPath()) ('veylora-release-' + [guid]::NewGuid())
$passed = 0
function Check([string]$Name, [int]$Exit, [string]$Message, [string[]]$VerifyArgs) {
    $output = & $hostExe -NoProfile -File $verifier @VerifyArgs 2>&1 | Out-String
    if ($LASTEXITCODE -ne $Exit -or $output -notmatch [regex]::Escape($Message)) {
        throw "$Name expected exit $Exit / '$Message', got $LASTEXITCODE : $output"
    }
    $script:passed++
    Write-Host "PASS $Name"
}
function New-Release([string]$Name) {
    $d = Join-Path $work $Name
    New-Item $d -ItemType Directory -Force | Out-Null
    Copy-Item $Cli (Join-Path $d 'veylora-cli.exe')
    Set-Content (Join-Path $d 'LICENSE.txt') 'license'
    & $Cli release manifest --dir $d --version 1.2.3 --max-data-schema 1 --smoke-test veylora-cli.exe --smoke-arg --version | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'manifest failed' }
    $d
}
try {
    New-Item $work -ItemType Directory | Out-Null
    $good = New-Release 'good'
    Check 'intact directory' 0 'RELEASE OK' @('-Path', $good, '-Cli', $Cli, '-RunSmokeTest')
    Check 'unsigned is reported, not hidden' 0 'unsigned executables: 1' @('-Path', $good)
    Check 'signature can be required' 1 'not validly signed' @('-Path', $good, '-RequireSignature')

    $t = New-Release 'tampered'; Set-Content (Join-Path $t 'LICENSE.txt') 'changed!'
    Check 'changed file' 1 'LICENSE.txt has the wrong size' @('-Path', $t)
    $same = New-Release 'samesize'; Set-Content (Join-Path $same 'LICENSE.txt') 'licensX'
    Check 'same size, different bytes' 1 'does not match its recorded hash' @('-Path', $same)
    $m = New-Release 'missing'; Remove-Item (Join-Path $m 'LICENSE.txt')
    Check 'missing file' 1 'listed but missing' @('-Path', $m)
    $e = New-Release 'extra'; Set-Content (Join-Path $e 'extra.dll') 'x'
    Check 'unlisted file' 1 'not listed' @('-Path', $e)
    $n = New-Release 'nomanifest'; Remove-Item (Join-Path $n 'release-manifest.json')
    Check 'missing manifest' 1 'release-manifest.json is missing' @('-Path', $n)

    $dev = New-Release 'dev'
    $man = Get-Content (Join-Path $dev 'release-manifest.json') -Raw | ConvertFrom-Json -AsHashtable
    New-Item (Join-Path $dev 'node_modules') -ItemType Directory | Out-Null
    Set-Content (Join-Path $dev 'node_modules/x.js') 'x'
    $man.files += @{ path = 'node_modules/x.js'; bytes = 3; sha256 = (Get-FileHash (Join-Path $dev 'node_modules/x.js')).Hash.ToLower() }
    $man | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $dev 'release-manifest.json')
    Check 'development tooling in a manifest' 1 'development tool' @('-Path', $dev)

    $unsafe = New-Release 'unsafe'
    $man = Get-Content (Join-Path $unsafe 'release-manifest.json') -Raw | ConvertFrom-Json -AsHashtable
    $man.files[0].path = '../evil.exe'
    $man | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $unsafe 'release-manifest.json')
    Check 'path traversal in a manifest' 1 'unsafe path' @('-Path', $unsafe)

    # A zip must match a hash given out of band; no hash means no trust.
    $zip = Join-Path $work 'good.zip'
    Compress-Archive -Path (Join-Path $good '*') -DestinationPath $zip
    $hash = (Get-FileHash $zip -Algorithm SHA256).Hash
    Check 'zip without a hash' 1 'no hash to check' @('-Path', $zip)
    Check 'zip with a wrong hash' 1 'does not match the expected' @('-Path', $zip, '-ExpectedSha256', ('0' * 64))
    Check 'zip with the right hash' 0 'RELEASE OK' @('-Path', $zip, '-ExpectedSha256', $hash)
    Set-Content "$zip.sha256" "$($hash.ToLower())  good.zip" -NoNewline
    Check 'zip with a sidecar hash' 0 'RELEASE OK' @('-Path', $zip)

    # A zip that would write outside its destination.
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $slip = Join-Path $work 'slip.zip'
    $z = [IO.Compression.ZipFile]::Open($slip, 'Create')
    $entry = $z.CreateEntry('../escaped.txt'); $w = [IO.StreamWriter]::new($entry.Open()); $w.Write('x'); $w.Dispose()
    $z.Dispose()
    $slipHash = (Get-FileHash $slip -Algorithm SHA256).Hash
    Check 'zip slip' 1 'outside the destination' @('-Path', $slip, '-ExpectedSha256', $slipHash)
    if (Test-Path (Join-Path $work '../escaped.txt')) { throw 'zip slip wrote outside the destination' }
    Write-Host "$passed verify-release checks passed"
} finally {
    if (Test-Path $work) { Remove-Item $work -Recurse -Force }
}
