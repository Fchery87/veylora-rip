param([switch]$AssertWorkspaceBlocked)
$ErrorActionPreference = 'Stop'
$checker = Join-Path $PSScriptRoot '../check-licenses.ps1'
if (-not (Test-Path $checker)) { throw 'RED: executable license checker is missing' }
$hostExe = (Get-Process -Id $PID).Path
$root = Join-Path ([IO.Path]::GetTempPath()) ('veylora-license-' + [guid]::NewGuid())
$passed = 0
function Save-Fixture {
    param($Inventory)
    $json = $Inventory | ConvertTo-Json -Depth 20
    Set-Content "$root/docs/dependency-inventory.md" ((@('# Fixture inventory', '```json license-inventory', $json, '```')) -join "`n")
}
function Check {
    param([string]$Name, [int]$Exit, [string]$Message)
    $output = & $hostExe -NoProfile -File $checker -Root $root 2>&1 | Out-String
    if ($LASTEXITCODE -ne $Exit -or $output -notmatch [regex]::Escape($Message)) {
        throw "$Name expected exit $Exit / $Message, got $LASTEXITCODE : $output"
    }
    $script:passed++
    Write-Host "PASS $Name"
}
try {
    New-Item "$root/docs" -ItemType Directory -Force | Out-Null
    & git -C $root init --quiet
    Set-Content "$root/Cargo.lock" "version = 3`n`n[[package]]`nname = `"fixture`"`nversion = `"1.0.0`"`nsource = `"registry+https://github.com/rust-lang/crates.io-index`""
    Set-Content "$root/THIRD_PARTY_NOTICES.md" "# Notices`n<!-- notice:fixture -->`nSynthetic test license, not a permission grant.`n<!-- /notice:fixture -->"
    Set-Content "$root/docs/profile-and-font-licensing.md" '# Fixture asset policy'
    $inventory = @{ schema = 1; files = @(@{path='Cargo.lock'; sha256=(Get-FileHash "$root/Cargo.lock").Hash}); components = @(@{id='cargo:fixture@1.0.0'; license='MIT'; source='https://example.invalid/fixture/manifest'; licenseSource='https://example.invalid/fixture/LICENSE'; status='reviewed'; notice='fixture'; noticeSha256='' }); blockers=@() }
    $text = 'Synthetic test license, not a permission grant.'
    $inventory.components[0].noticeSha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($text)))
    Save-Fixture $inventory
    Check 'known synthetic fixture' 0 'License check passed'
    Set-Content "$root/docs/profile-and-font-licensing.md" ''
    Check 'empty required policy' 1 'Empty required file'
    Set-Content "$root/docs/profile-and-font-licensing.md" '# Fixture asset policy'
    Remove-Item "$root/THIRD_PARTY_NOTICES.md"
    Check 'missing notice file' 1 'Missing required file'
    Set-Content "$root/THIRD_PARTY_NOTICES.md" '# Empty notices'
    Check 'missing individual notice' 1 'Missing notice'
    Set-Content "$root/THIRD_PARTY_NOTICES.md" "<!-- notice:fixture -->`nAltered text`n<!-- /notice:fixture -->"
    Check 'changed notice' 1 'Notice hash mismatch'
    $inventory.components[0].status = 'unresolved'
    Save-Fixture $inventory
    Check 'unresolved review' 1 'Unresolved license review'
    $inventory.components[0].status = 'reviewed'
    $inventory.components[0].source = ''
    Save-Fixture $inventory
    Check 'missing authoritative source' 1 'Missing source metadata'
    $inventory.components[0].source = 'https://example.invalid/fixture/manifest'
    $inventory.components[0].license = ''
    Save-Fixture $inventory
    Check 'missing license metadata' 1 'Missing license'
    $inventory.components[0].license = 'UNKNOWN'
    Save-Fixture $inventory
    Check 'unknown license' 1 'Unresolved license'
    $inventory.components[0].license = 'MIT'
    $inventory.components[0].id = 'cargo:AcroRIP@1.0.0'
    Save-Fixture $inventory
    Check 'forbidden commercial RIP' 1 'Forbidden commercial RIP'
    $inventory.components = @()
    Save-Fixture $inventory
    Check 'uninventoried dependency' 1 'Uninventoried dependency'
    Add-Content "$root/Cargo.lock" '# drift'
    Check 'changed dependency evidence' 1 'Inventory drift'
    Set-Content "$root/new-font.woff2" 'new asset'
    Check 'new asset' 1 'Uninventoried file'
    New-Item "$root/app" -ItemType Directory | Out-Null
    Set-Content "$root/app/package.json" '{"dependencies":{"fixture-direct":"2.0.0"}}'
    Set-Content "$root/app/pnpm-lock.yaml" "lockfileVersion: '9.0'`npackages:`n  '@scope/fixture@3.0.0':`n    resolution: {integrity: synthetic}`nsnapshots: {}`n"
    Check 'frontend declaration discovery' 1 'Uninventoried dependency: npm:fixture-direct@2.0.0'
    Check 'frontend graph absent from lock' 1 'Frontend dependency absent from lock: npm:fixture-direct@2.0.0'
    Check 'scoped frontend lock discovery' 1 'Uninventoried dependency: npm:@scope/fixture@3.0.0'
    Set-Content "$root/docs/dependency-inventory.md" '# No structured inventory'
    Check 'malformed inventory' 1 'Expected one json license-inventory block'
    Write-Host "$passed license fixture checks passed"
    if ($AssertWorkspaceBlocked) {
        $output = & $hostExe -NoProfile -File $checker 2>&1 | Out-String
        if ($LASTEXITCODE -ne 1 -or $output -notmatch 'Unresolved review blocker' -or $output -match 'Inventory drift|Uninventoried') {
            throw "Unexpected current workspace result: $output"
        }
        Write-Host 'PASS current workspace is blocked by unresolved review, without inventory drift'
    }
} finally {
    Remove-Item $root -Recurse -Force
}
