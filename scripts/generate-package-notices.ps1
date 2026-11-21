#Requires -Version 7.2
<#
.SYNOPSIS
Regenerates the dependency inventory and the generated part of THIRD_PARTY_NOTICES.md for one
packaged program, from the exact crate sources that Cargo.lock pins.

.DESCRIPTION
This is the single writer of the license inventory. It first rewrites the whole inventory as
unresolved (check-licenses.ps1 -RefreshUnreviewedInventory), then, for the packaged program only,
marks a component reviewed when ALL of these hold:
  - its crate archive in the local cargo cache matches the checksum in Cargo.lock;
  - its declared SPDX expression can be satisfied with permissive licenses only (an OR may be
    resolved by electing a permissive alternative);
  - the crate source contains license text, which is copied verbatim into the notice.
Everything else stays unresolved and is listed as a scope blocker. That is an automated policy
decision authorised by the project owner for this scope; it is not a legal review of each license.
Components outside the scope are never touched.
#>
[CmdletBinding()]
param(
    [string]$Root = (Split-Path $PSScriptRoot -Parent),
    [Parameter(Mandatory)][string]$Package,
    [string]$Target = 'x86_64-pc-windows-msvc',
    # Owner decision recorded for the project's own crates.
    [string]$ProjectCopyright = 'Copyright (c) 2026 Frantz Chery. All rights reserved.'
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$Root = (Resolve-Path $Root).Path

. (Join-Path $PSScriptRoot 'lib/spdx.ps1')

function Get-TextHash([string]$Text) {
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
        [Text.Encoding]::UTF8.GetBytes(($Text -replace "`r`n", "`n").Trim())))
}

function Get-RuntimeGraph {
    $lines = @(& cargo tree --manifest-path (Join-Path $Root 'Cargo.toml') -p $Package -e normal --target $Target --offline --locked --prefix none --format '{p}' 2>&1)
    if ($LASTEXITCODE -ne 0) { throw "cargo tree failed: $($lines -join ' ')" }
    $found = [Collections.Generic.SortedDictionary[string, bool]]::new()
    $rootFull = [IO.Path]::GetFullPath($Root).TrimEnd([char]92, [char]47)
    foreach ($line in $lines) {
        $m = [regex]::Match([string]$line, '^(\S+) v(\S+)(?: \((?!\*\))([^)]+)\))?')
        if (-not $m.Success) { throw "Unrecognised cargo tree line: $line" }
        $where = $m.Groups[3].Value
        $isPath = [bool]($where -and $where -notmatch '^(registry|git)\+|proc-macro')
        if ($isPath -and -not [IO.Path]::GetFullPath($where).StartsWith($rootFull, [StringComparison]::OrdinalIgnoreCase)) { throw "Path dependency outside the repository: $where" }
        $found["cargo:$($m.Groups[1].Value)@$($m.Groups[2].Value)"] = $isPath
    }
    $found
}

function Get-LockChecksum([string]$Name, [string]$Version) {
    $lock = Get-Content "$Root/Cargo.lock" -Raw
    $m = [regex]::Match($lock, "(?ms)\[\[package\]\]\r?\nname = `"$([regex]::Escape($Name))`"\r?\nversion = `"$([regex]::Escape($Version))`"\r?\n(.*?)(?=\[\[package\]\]|\z)")
    if (-not $m.Success) { return $null }
    $c = [regex]::Match($m.Groups[1].Value, '(?m)^checksum = "([0-9a-f]{64})"')
    if ($c.Success) { $c.Groups[1].Value } else { $null }
}

$licenseFile = '^(LICENSE|LICENCE|COPYING|NOTICE|UNLICENSE)([-_.].*)?$'

function Get-CrateEvidence([string]$Name, [string]$Version) {
    $src = @(Get-ChildItem "$HOME/.cargo/registry/src/*/$Name-$Version" -Directory -ErrorAction SilentlyContinue)
    $archive = @(Get-ChildItem "$HOME/.cargo/registry/cache/*/$Name-$Version.crate" -ErrorAction SilentlyContinue)
    if ($src.Count -ne 1 -or $archive.Count -ne 1) { return @{ Problem = 'exact crate source or archive is not in the local cargo cache' } }
    $expected = Get-LockChecksum $Name $Version
    if (-not $expected) { return @{ Problem = 'Cargo.lock records no checksum for this package' } }
    $actual = (Get-FileHash $archive[0].FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { return @{ Problem = 'cached crate archive does not match the Cargo.lock checksum' } }
    $manifest = Get-Content (Join-Path $src[0].FullName 'Cargo.toml') -Raw
    $package = [regex]::Match($manifest, '(?ms)^\[package\]\r?\n(.*?)(?=^\[)').Groups[1].Value
    $declared = [regex]::Match($package, '(?m)^license = "([^"]+)"').Groups[1].Value
    if (-not $declared) { return @{ Problem = 'the crate manifest declares no SPDX license expression' } }
    $files = @(Get-ChildItem $src[0].FullName -File | Where-Object { $_.Name -match $licenseFile -and $_.Length -lt 200KB } | Sort-Object Name)
    # Some crates ship no LICENSE file but carry the grant in each source file's header. Those
    # headers are the license text; they are reproduced (comment markers removed), one per
    # distinct text, with the files that carry it.
    $headers = @()
    if ($files.Count -eq 0) {
        $found = @{}
        foreach ($f in (Get-ChildItem (Join-Path $src[0].FullName 'src') -Recurse -Filter *.rs -File)) {
            $lines = [IO.File]::ReadAllLines($f.FullName)
            $head = [Collections.Generic.List[string]]::new()
            foreach ($line in $lines) { if ($line -match '^\s*//(?!/)') { $head.Add(($line -replace '^\s*//\s?', '').TrimEnd()) } else { break } }
            $text = ($head -join "`n").Trim()
            if ($text -notmatch 'Permission is hereby granted') { continue }
            if (-not $found.ContainsKey($text)) { $found[$text] = [Collections.Generic.List[string]]::new() }
            $found[$text].Add($f.FullName.Substring($src[0].FullName.Length + 1).Replace([string][char]92, '/'))
        }
        $headers = @($found.GetEnumerator() | Sort-Object { $_.Value[0] } | ForEach-Object { "--- license header carried by: $(($_.Value | Sort-Object) -join ', ') ---`n$($_.Key)" })
    }
    @{ Problem = $null; License = $declared; Files = $files; Headers = $headers }
}

function New-NoticeText([string]$Header, $Files, [string[]]$Headers = @()) {
    $parts = [Collections.Generic.List[string]]::new()
    $parts.Add($Header)
    foreach ($f in $Files) {
        $parts.Add("--- $($f.Name) ---")
        $parts.Add(([IO.File]::ReadAllText($f.FullName).Replace("`r`n", "`n")))
    }
    foreach ($h in $Headers) { $parts.Add($h) }
    ($parts -join "`n`n").Trim()
}

# 1. Rewrite the whole inventory as unresolved, so nothing stale survives.
$hostExe = (Get-Process -Id $PID).Path
& $hostExe -NoProfile -File (Join-Path $PSScriptRoot 'check-licenses.ps1') -Root $Root -RefreshUnreviewedInventory | Out-Host
if ($LASTEXITCODE -ne 0) { throw 'inventory refresh failed' }
$invPath = "$Root/docs/dependency-inventory.md"
$doc = Get-Content $invPath -Raw
$block = [regex]::Match($doc, '(?ms)^```json license-inventory\r?\n(.*?)^```\s*$')
$inventory = $block.Groups[1].Value | ConvertFrom-Json -AsHashtable

# 2. Evidence for the packaged program.
$ids = Get-RuntimeGraph
$notices = [Collections.Generic.List[string]]::new()
$blockers = [Collections.Generic.List[string]]::new()
$byId = @{}
foreach ($c in $inventory.components) { $byId[[string]$c.id] = $c }

function Add-Notice([string]$Key, [string]$Text) {
    $notices.Add("<!-- notice:$Key -->`n$Text`n<!-- /notice:$Key -->")
    Get-TextHash $Text
}

$workspaceIds = @()
foreach ($id in @($ids.Keys)) {
    $m = [regex]::Match($id, '^cargo:(.+)@([^@]+)$')
    $name = $m.Groups[1].Value; $version = $m.Groups[2].Value
    $c = $byId[$id]
    if ($ids[$id]) { $workspaceIds += $id; continue }
    $ev = Get-CrateEvidence $name $version
    if ($ev.Problem) { $blockers.Add("${id}: $($ev.Problem)"); continue }
    $c.license = $ev.License
    if (-not (Test-PermissiveExpression $ev.License)) { $blockers.Add("${id}: license '$($ev.License)' needs a human decision (not all-permissive)"); continue }
    if ($ev.Files.Count -eq 0 -and $ev.Headers.Count -eq 0) { $blockers.Add("${id}: the crate ships no license text to reproduce"); continue }
    $key = "cargo-$name-$version"
    $header = "Crate: $name $version`nDeclared license: $($ev.License)`nSource: https://crates.io/crates/$name/$version (archive checksum matches Cargo.lock)"
    $c.notice = $key
    $c.noticeSha256 = Add-Notice $key (New-NoticeText $header $ev.Files $ev.Headers)
    $c.status = 'reviewed'
    $c.source = "https://crates.io/api/v1/crates/$name/$version/download (checksum from Cargo.lock)"
    $c.licenseSource = $(if ($ev.Files.Count) { 'verbatim license files from the checksum-verified crate source' } else { 'license text from source-file headers of the checksum-verified crate (no LICENSE file shipped)' }) + '; permissive terms; automated under owner-approved policy'
}

# The project's own crates: proprietary, owner-authorised binary distribution.
$projectText = "Veylora RIP`n`n$ProjectCopyright`n`nVeylora RIP is proprietary software. It is distributed in binary form only. No license to its source code is granted, and no part of it may be copied, modified or redistributed except as the copyright holder authorises in writing.`n`nThird-party components linked into this program are licensed to you under their own terms, reproduced in the notices that follow."
$projectHash = Add-Notice 'veylora-rip' $projectText
foreach ($id in $workspaceIds) {
    $c = $byId[$id]
    $c.license = 'LicenseRef-Veylora-Proprietary'
    $c.source = 'Cargo.toml [workspace.package] and member Cargo.toml'
    $c.licenseSource = 'project owner decision 2026-11-20: proprietary, all rights reserved, binary distribution authorised'
    $c.status = 'reviewed'
    $c.notice = 'veylora-rip'
    $c.noticeSha256 = $projectHash
}

# The Rust standard library is linked into every Rust program.
$sysroot = (& rustc --print sysroot).Trim()
$rustVersion = [regex]::Match((& rustc -V), 'rustc (\S+)').Groups[1].Value
$docs = Join-Path $sysroot 'share/doc/rust'
$stdFiles = @('licenses/MIT.txt', 'licenses/Apache-2.0.txt') | ForEach-Object { Get-Item (Join-Path $docs $_) }
$stdCopyright = Join-Path $docs 'COPYRIGHT-library.html'
$stdHash = (Get-FileHash $stdCopyright -Algorithm SHA256).Hash
$stdId = "toolchain:rust-std@$rustVersion"
$stdHeader = "Rust standard library $rustVersion`nDeclared license: Apache-2.0 OR MIT`nCopyright: The Rust Project Developers (https://thanks.rust-lang.org)`nThird-party code inside the standard library is listed in licenses/rust-std-COPYRIGHT.html, shipped with the program (SHA-256 $stdHash)."
$stdNoticeHash = Add-Notice "toolchain-rust-std-$rustVersion" (New-NoticeText $stdHeader $stdFiles)
$inventory.components = @($inventory.components) + [ordered]@{
    id = $stdId; license = 'Apache-2.0 OR MIT'; source = "https://github.com/rust-lang/rust/tree/$rustVersion/library"
    licenseSource = "rustc $rustVersion distribution: share/doc/rust/licenses and COPYRIGHT-library.html"
    status = 'reviewed'; notice = "toolchain-rust-std-$rustVersion"; noticeSha256 = $stdNoticeHash
}

# 3. Scope, then write.
$inventory.scopes = [ordered]@{
    $Package = [ordered]@{
        target = $Target
        extra = @($stdId)
        rustStdCopyrightSha256 = $stdHash
        blockers = @($blockers)
    }
}
$json = $inventory | ConvertTo-Json -Depth 12
$proseEnd = $doc.IndexOf('<!-- machine-inventory -->')
$newDoc = $doc.Substring(0, $proseEnd) + "<!-- machine-inventory -->`n``````json license-inventory`n$json`n```````n"
[IO.File]::WriteAllText($invPath, $newDoc, [Text.UTF8Encoding]::new($false))

$noticePath = "$Root/THIRD_PARTY_NOTICES.md"
$existing = Get-Content $noticePath -Raw
$marker = '<!-- generated-notices -->'
$head = ($existing -split [regex]::Escape($marker))[0].TrimEnd()
$generated = @($marker, "## Notices for $Package (generated by scripts/generate-package-notices.ps1)", '') + $notices
[IO.File]::WriteAllText($noticePath, (@($head, '') + $generated -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
Write-Host "Scope $Package ($Target): $($ids.Count) cargo identities + rust std; $($blockers.Count) blockers."
foreach ($b in $blockers) { Write-Host "  BLOCKER: $b" }
Write-Host "Rust std third-party notices (ship as licenses/rust-std-COPYRIGHT.html): $stdCopyright"
