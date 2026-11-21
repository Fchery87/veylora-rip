#Requires -Version 7.2
<#
.SYNOPSIS
Verifies a release directory or zip without trusting the program inside it.

.DESCRIPTION
Recomputes every file hash against release-manifest.json in PowerShell, refuses extra or missing
files, unsafe paths and development tooling, and for a zip also checks the archive hash against a
hash given out of band. Pass -Cli to cross-check with a trusted veylora-cli. A release that cannot
be shown intact fails; there is no skip switch.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Path,
    # SHA-256 of the zip, from the publisher. Defaults to the "<zip>.sha256" sidecar.
    [string]$ExpectedSha256,
    # A trusted veylora-cli, used to cross-check the manifest with the Rust verifier.
    [string]$Cli,
    # Run the manifest's smoke test from the extracted release.
    [switch]$RunSmokeTest,
    # Fail unless every .exe/.dll carries a valid Authenticode signature.
    [switch]$RequireSignature
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$devDirs = @('node_modules', '.git', 'target')
$devFiles = @('cargo.toml', 'cargo.lock', 'package.json', 'pnpm-lock.yaml', 'package-lock.json', 'yarn.lock', 'tsconfig.json')
$devExt = @('.rs', '.ts', '.pdb', '.rlib', '.rmeta', '.ilk')

function Fail([string]$Message) { Write-Error "RELEASE REJECTED: $Message" -ErrorAction Continue; exit 1 }

function Test-RelativePath([string]$Rel) {
    if (-not $Rel -or $Rel.Length -gt 240) { return $false }
    if ($Rel -match '[\\:<>"|?*\x00-\x1f]') { return $false }
    foreach ($part in $Rel.Split('/')) {
        if (-not $part -or $part -eq '.' -or $part -eq '..' -or $part.EndsWith('.') -or $part.EndsWith(' ')) { return $false }
    }
    $true
}

function Test-DevTool([string]$Rel) {
    $parts = $Rel.ToLowerInvariant().Split('/')
    $file = $parts[-1]
    if ($parts.Length -gt 1 -and ($parts[0..($parts.Length - 2)] | Where-Object { $devDirs -contains $_ })) { return $true }
    $stem = $file.Split('.')[0]
    ($devFiles -contains $file) -or $stem -eq 'rust-toolchain' -or ($stem -eq 'vite' -and $file.Contains('.config.')) -or ($devExt -contains [IO.Path]::GetExtension($file))
}

$cleanup = $null
try {
    $item = Get-Item -LiteralPath $Path
    if (-not $item.PSIsContainer) {
        if ($item.Extension -ne '.zip') { Fail 'Path must be a release directory or a .zip' }
        if (-not $ExpectedSha256) {
            $sidecar = "$($item.FullName).sha256"
            if (-not (Test-Path -LiteralPath $sidecar)) { Fail "no hash to check the zip against: give -ExpectedSha256 or provide $sidecar" }
            $ExpectedSha256 = ((Get-Content -LiteralPath $sidecar -Raw).Trim() -split '\s+')[0]
        }
        $actual = (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash
        if ($actual -ne $ExpectedSha256.Trim().ToUpperInvariant()) { Fail "zip hash $actual does not match the expected $ExpectedSha256" }
        $cleanup = Join-Path ([IO.Path]::GetTempPath()) ('veylora-verify-' + [guid]::NewGuid())
        New-Item $cleanup -ItemType Directory | Out-Null
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $zip = [IO.Compression.ZipFile]::OpenRead($item.FullName)
        try {
            $base = [IO.Path]::GetFullPath($cleanup) + [IO.Path]::DirectorySeparatorChar
            foreach ($entry in $zip.Entries) {
                $target = [IO.Path]::GetFullPath((Join-Path $cleanup $entry.FullName))
                if (-not $target.StartsWith($base, [StringComparison]::OrdinalIgnoreCase)) { Fail "zip entry $($entry.FullName) would extract outside the destination" }
                if ($entry.FullName.EndsWith('/')) { New-Item $target -ItemType Directory -Force | Out-Null; continue }
                New-Item (Split-Path $target) -ItemType Directory -Force | Out-Null
                [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $target)
            }
        } finally { $zip.Dispose() }
        $dir = $cleanup
    } else {
        $dir = $item.FullName
    }

    $manifestPath = Join-Path $dir 'release-manifest.json'
    if (-not (Test-Path -LiteralPath $manifestPath)) { Fail 'release-manifest.json is missing' }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    if ($manifest.schema_version -ne 1 -or $manifest.product -ne 'veylora-rip') { Fail 'unsupported manifest or wrong product' }
    if ($manifest.version -notmatch '^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$') { Fail "version '$($manifest.version)' is not MAJOR.MINOR.PATCH" }

    $listed = @{}
    foreach ($f in $manifest.files) {
        if (-not (Test-RelativePath $f.path)) { Fail "unsafe path in manifest: $($f.path)" }
        if (Test-DevTool $f.path) { Fail "$($f.path) is a development tool or build leftover" }
        if ($listed.ContainsKey($f.path)) { Fail "$($f.path) is listed twice" }
        $listed[$f.path] = $f
    }
    $root = [IO.Path]::GetFullPath($dir).TrimEnd('\', '/')
    $present = @(Get-ChildItem -LiteralPath $dir -Recurse -File -Force | Where-Object { $_.FullName -ne $manifestPath })
    foreach ($file in $present) {
        $rel = $file.FullName.Substring($root.Length + 1).Replace('\', '/')
        if ($file.Attributes -band [IO.FileAttributes]::ReparsePoint) { Fail "$rel is a link, not a regular file" }
        if (-not $listed.ContainsKey($rel)) { Fail "$rel is present but not listed in the manifest" }
    }
    foreach ($f in $manifest.files) {
        $full = Join-Path $dir $f.path
        if (-not (Test-Path -LiteralPath $full -PathType Leaf)) { Fail "$($f.path) is listed but missing" }
        $info = Get-Item -LiteralPath $full
        if ($info.Length -ne $f.bytes) { Fail "$($f.path) has the wrong size" }
        if ((Get-FileHash -LiteralPath $full -Algorithm SHA256).Hash -ne $f.sha256.ToUpperInvariant()) { Fail "$($f.path) does not match its recorded hash" }
    }

    $unsigned = @()
    foreach ($f in ($manifest.files | Where-Object { $_.path -match '\.(exe|dll)$' })) {
        $sig = Get-AuthenticodeSignature -LiteralPath (Join-Path $dir $f.path)
        if ($sig.Status -ne 'Valid') { $unsigned += $f.path }
    }
    if ($RequireSignature -and $unsigned.Count) { Fail "not validly signed: $($unsigned -join ', ')" }

    if ($Cli) {
        & $Cli release verify --dir $dir | Out-Null
        if ($LASTEXITCODE -ne 0) { Fail 'the Rust verifier rejected the release' }
    }
    if ($RunSmokeTest) {
        if (-not $manifest.smoke_test) { Fail 'the manifest declares no smoke test' }
        $program = Join-Path $dir $manifest.smoke_test[0]
        $smokeArgs = @($manifest.smoke_test | Select-Object -Skip 1)
        & $program @smokeArgs | Out-Null
        if ($LASTEXITCODE -ne 0) { Fail "the smoke test exited with $LASTEXITCODE" }
    }
    # The desktop app is a GUI program: `&` would not wait for it or report its exit code, so it is
    # started and waited on directly. --version proves it loads without opening a window.
    if ($RunSmokeTest -and ($manifest.files | Where-Object { $_.path -eq 'veylora-app.exe' })) {
        $proc = Start-Process -FilePath (Join-Path $dir 'veylora-app.exe') -ArgumentList '--version' -Wait -PassThru -WindowStyle Hidden
        if ($proc.ExitCode -ne 0) { Fail "veylora-app.exe --version exited with $($proc.ExitCode)" }
    }
    Write-Host "RELEASE OK: veylora-rip $($manifest.version), $($manifest.files.Count) files; unsigned executables: $($unsigned.Count)"
    exit 0
} finally {
    if ($cleanup -and (Test-Path -LiteralPath $cleanup)) { Remove-Item -LiteralPath $cleanup -Recurse -Force }
}
