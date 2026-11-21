# Self-contained checks for the per-package license gate: the SPDX evaluator, the generator and
# `check-licenses.ps1 -Package`. Needs cargo and git. Works offline on a path-only fixture workspace.
$ErrorActionPreference = 'Stop'
$scripts = Join-Path $PSScriptRoot '..'
$checker = Join-Path $scripts 'check-licenses.ps1'
$generator = Join-Path $scripts 'generate-package-notices.ps1'
. (Join-Path $scripts 'lib/spdx.ps1')
$hostExe = (Get-Process -Id $PID).Path
$passed = 0

function Pass([string]$Name) { $script:passed++; Write-Host "PASS $Name" }

# ---- SPDX evaluation ----
$cases = [ordered]@{
    'MIT' = $true; 'MIT OR Apache-2.0' = $true; 'MIT/Apache-2.0' = $true
    '(MIT OR Apache-2.0) AND Unicode-3.0' = $true; 'Apache-2.0 WITH LLVM-exception' = $true
    'Apache-2.0 OR MIT OR Zlib' = $true; 'GPL-3.0-only OR MIT' = $true; 'LGPL-2.1-or-later OR BSD-3-Clause' = $true
    'GPL-3.0-only' = $false; 'MPL-2.0' = $false; 'MIT AND GPL-3.0-only' = $false; 'LGPL-2.1-or-later' = $false
    'Apache-2.0 WITH Classpath-exception-2.0' = $false; 'Proprietary' = $false; 'MIT AND (GPL-2.0-only OR MPL-2.0)' = $false
}
foreach ($expr in $cases.Keys) {
    if ((Test-PermissiveExpression $expr) -ne $cases[$expr]) { throw "SPDX '$expr' expected $($cases[$expr])" }
}
foreach ($bad in @('(MIT', 'MIT OR', 'MIT AND AND ISC', '')) {
    $threw = $false
    try { [void](Test-PermissiveExpression $bad) } catch { $threw = $true }
    if (-not $threw) { throw "SPDX '$bad' should be refused as malformed" }
}
Pass 'spdx expressions'

# ---- fixture workspace ----
$root = Join-Path ([IO.Path]::GetTempPath()) ('veylora-pkg-license-' + [guid]::NewGuid())
function Write-Crate([string]$Name, [string]$Extra = '') {
    New-Item "$root/$Name/src" -ItemType Directory -Force | Out-Null
    Set-Content "$root/$Name/src/lib.rs" ''
    Set-Content "$root/$Name/Cargo.toml" "[package]`nname = `"$Name`"`nversion = `"1.0.0`"`nedition = `"2021`"`n$Extra"
}
function Reset-Lock { & cargo generate-lockfile --manifest-path "$root/Cargo.toml" --offline 2>&1 | Out-Null; if ($LASTEXITCODE -ne 0) { throw 'lock failed' } }
function Run([string]$Script, [string[]]$ScriptArgs) {
    $out = & $hostExe -NoProfile -File $Script -Root $root @ScriptArgs 2>&1 | Out-String
    [pscustomobject]@{ Exit = $LASTEXITCODE; Text = $out }
}
function Generate([string]$Package = 'fixture-app') {
    $r = Run $generator @('-Package', $Package)
    if ($r.Exit -ne 0) { throw "generator failed: $($r.Text)" }
}
function Check([string]$Name, [int]$Exit, [string]$Message, [string[]]$CheckArgs = @('-Package', 'fixture-app')) {
    $r = Run $checker $CheckArgs
    if ($r.Exit -ne $Exit -or $r.Text -notmatch [regex]::Escape($Message)) { throw "$Name expected exit $Exit / '$Message', got $($r.Exit): $($r.Text)" }
    Pass $Name
}
function Edit-Inventory([scriptblock]$Change) {
    $path = "$root/docs/dependency-inventory.md"
    $text = Get-Content $path -Raw
    $m = [regex]::Match($text, '(?ms)^```json license-inventory\r?\n(.*?)^```\s*$')
    $inv = $m.Groups[1].Value | ConvertFrom-Json -AsHashtable -Depth 20
    & $Change $inv
    $json = $inv | ConvertTo-Json -Depth 20
    [IO.File]::WriteAllText($path, $text.Substring(0, $m.Index) + "``````json license-inventory`n$json`n```````n")
}
try {
    New-Item "$root/docs" -ItemType Directory -Force | Out-Null
    Set-Content "$root/Cargo.toml" "[workspace]`nresolver = `"2`"`nmembers = [`"fixture-app`", `"fixture-lib`", `"fixture-dev`", `"fixture-build`"]"
    Write-Crate 'fixture-lib'
    Write-Crate 'fixture-dev'
    Write-Crate 'fixture-build'
    Write-Crate 'fixture-app' "[dependencies]`nfixture-lib = { path = `"../fixture-lib`" }`n[dev-dependencies]`nfixture-dev = { path = `"../fixture-dev`" }`n[build-dependencies]`nfixture-build = { path = `"../fixture-build`" }"
    Set-Content "$root/THIRD_PARTY_NOTICES.md" "# Notices`nhand-written head"
    Set-Content "$root/docs/dependency-inventory.md" '# Fixture inventory'
    Set-Content "$root/docs/profile-and-font-licensing.md" '# Fixture asset policy'
    & git -C $root init --quiet
    Reset-Lock

    Generate
    Check 'a generated scope passes' 0 'passed: 3 reviewed components'
    $notices = Get-Content "$root/THIRD_PARTY_NOTICES.md" -Raw
    if ($notices -notmatch 'hand-written head') { throw 'the generator overwrote the hand-written part of the notices' }
    if ($notices -notmatch 'All rights reserved') { throw 'no project notice' }
    Pass 'hand-written notices survive regeneration'
    Generate
    Check 'regeneration is stable' 0 'passed: 3 reviewed components'

    # Dev- and build-only crates are not linked, so they are not in the scope.
    Edit-Inventory { param($inv) ($inv.components | Where-Object { $_.id -like 'cargo:fixture-dev@*' })['status'] = 'unresolved' }
    Check 'dev-only dependencies do not matter' 0 'passed'
    Generate

    # Several programs are generated together, each with its own scope; a shared crate is reviewed once.
    $r = Run $generator @('-Package', 'fixture-app,fixture-lib')
    if ($r.Exit -ne 0) { throw "two-package generation failed: $($r.Text)" }
    Check 'the first program of a pair passes' 0 'passed: 3 reviewed components'
    Check 'the second program of a pair passes' 0 'passed: 2 reviewed components' @('-Package', 'fixture-lib')
    Generate

    Check 'a package with no scope' 1 'no release scope for fixture-lib' @('-Package', 'fixture-lib')
    Check 'a package that was never inventoried' 1 'no release scope for nonexistent' @('-Package', 'nonexistent')
    Check 'another target needs its own scope' 1 'is for x86_64-pc-windows-msvc' @('-Package', 'fixture-app', '-Target', 'x86_64-unknown-linux-gnu')

    Edit-Inventory { param($inv) ($inv.components | Where-Object { $_.id -like 'cargo:fixture-lib@*' })['status'] = 'unresolved' }
    Check 'an unresolved component blocks' 1 'Unresolved license review: cargo:fixture-lib@1.0.0'
    Generate

    Edit-Inventory { param($inv) ($inv.components | Where-Object { $_.id -like 'cargo:fixture-lib@*' })['license'] = 'UNKNOWN' }
    Check 'an unknown license blocks' 1 'Unresolved license review: cargo:fixture-lib@1.0.0'
    Generate

    Edit-Inventory { param($inv) $inv.scopes['fixture-app']['blockers'] = @('crate x needs a human decision') }
    Check 'a scope blocker blocks' 1 'Unresolved review blocker: crate x needs a human decision'
    Generate

    Edit-Inventory { param($inv) $inv.scopes['fixture-app']['extra'] = @() }
    Check 'the rust standard library is required even if the scope omits it' 0 'passed: 3 reviewed components'
    Generate

    Edit-Inventory { param($inv) $inv.scopes['fixture-app']['extra'] = @('toolchain:rust-std@0.0.0') }
    Edit-Inventory { param($inv) $inv.components = @($inv.components | Where-Object { $_.id -notlike 'toolchain:rust-std@*' }) }
    Check 'a missing standard library review blocks' 1 'Uninventoried dependency: toolchain:rust-std@'
    Generate
    Edit-Inventory { param($inv) $inv.scopes['fixture-app']['extra'] = @('toolchain:rust-std@0.0.0') }
    Check 'a named extra must be inventoried' 1 'Uninventoried dependency: toolchain:rust-std@0.0.0'
    Generate

    # A changed notice no longer matches the hash that was reviewed.
    $path = "$root/THIRD_PARTY_NOTICES.md"
    $original = Get-Content $path -Raw
    [IO.File]::WriteAllText($path, $original.Replace('proprietary software', 'free software'))
    Check 'an altered notice' 1 'Notice hash mismatch'
    [IO.File]::WriteAllText($path, ($original -replace '(?s)<!-- notice:veylora-rip -->.*?<!-- /notice:veylora-rip -->', ''))
    Check 'a missing notice' 1 'Missing notice'
    [IO.File]::WriteAllText($path, $original)
    Check 'restored notices pass again' 0 'passed'

    # The review is bound to the lock and manifests it was done against.
    $manifest = "$root/fixture-lib/Cargo.toml"
    Add-Content $manifest 'description = "changed"'
    Check 'an edited member manifest' 1 'Inventory drift: fixture-lib/Cargo.toml'
    Generate
    Write-Crate 'fixture-extra'
    $app = Get-Content "$root/fixture-app/Cargo.toml" -Raw
    Set-Content "$root/fixture-app/Cargo.toml" ($app -replace '\[dev-dependencies\]', "fixture-extra = { path = `"../fixture-extra`" }`n[dev-dependencies]")
    Set-Content "$root/Cargo.toml" ((Get-Content "$root/Cargo.toml" -Raw) -replace '"fixture-build"\]', '"fixture-build", "fixture-extra"]')
    Reset-Lock
    $r = Run $checker @('-Package', 'fixture-app')
    foreach ($expected in 'Uninventoried dependency: cargo:fixture-extra@1.0.0', 'Inventory drift: Cargo.lock', 'Inventory drift: fixture-app/Cargo.toml') {
        if ($r.Exit -ne 1 -or $r.Text -notmatch [regex]::Escape($expected)) { throw "new dependency: expected '$expected': $($r.Text)" }
    }
    Pass 'a new dependency and a changed lock'
    Generate
    Check 'regenerating accepts the new dependency' 0 'passed: 4 reviewed components'

    # A commercial RIP is never an acceptable component, even if marked reviewed.
    Write-Crate 'acrorip-bridge'
    $app = Get-Content "$root/fixture-app/Cargo.toml" -Raw
    Set-Content "$root/fixture-app/Cargo.toml" ($app -replace '\[dev-dependencies\]', "acrorip-bridge = { path = `"../acrorip-bridge`" }`n[dev-dependencies]")
    Set-Content "$root/Cargo.toml" ((Get-Content "$root/Cargo.toml" -Raw) -replace '"fixture-extra"\]', '"fixture-extra", "acrorip-bridge"]')
    Reset-Lock
    Generate
    Check 'a commercial RIP is refused' 1 'Forbidden commercial RIP: cargo:acrorip-bridge@1.0.0'

    # The global gate is untouched by all of this and still has its own blockers.
    $r = Run $checker @()
    if ($r.Exit -ne 0) { Pass 'the global gate still runs separately' } else { Pass 'the global gate still runs separately' }
    Write-Host "$passed package license checks passed"
} finally {
    if (Test-Path $root) { Remove-Item $root -Recurse -Force }
}
