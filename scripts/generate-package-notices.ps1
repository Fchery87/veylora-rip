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
A crate that ships no license text may be given one through docs/license-evidence/clarifications.json
(upstream text, SHA-256 checked). A copyleft license is accepted only if the owner has accepted it by
name in $AcceptedCopyleft below. Everything else stays unresolved and is listed as a scope blocker. That is an automated policy
decision authorised by the project owner for this scope; it is not a legal review of each license.
Components outside the scope are never touched.
#>
[CmdletBinding()]
param(
    [string]$Root = (Split-Path $PSScriptRoot -Parent),
    # One or more packaged programs; each gets its own scope in the inventory.
    [Parameter(Mandatory)][string[]]$Package,
    # The packaged program that embeds the built frontend (app/), if any. Its bundle is derived from
    # a real build's source maps, so only what is actually redistributed is reviewed.
    [string]$FrontendPackage = '',
    [string]$FrontendDir = 'app',
    [string]$Target = 'x86_64-pc-windows-msvc',
    # Owner decision recorded for the project's own crates.
    [string]$ProjectCopyright = 'Copyright (c) 2026 Frantz Chery. All rights reserved.'
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$Root = (Resolve-Path $Root).Path
# `pwsh -File` passes 'a,b' as one string.
$Package = @($Package | ForEach-Object { $_ -split ',' } | Where-Object { $_ })

# Licenses beyond the permissive set that the project owner has accepted for binary distribution,
# each with the decision it rests on. MPL-2.0 is file-level copyleft: the crates are shipped
# unmodified, their license text is in the notice, and their source is the published crate whose
# checksum Cargo.lock pins.
$AcceptedCopyleft = @{
    'MPL-2.0' = 'project owner decision 2026-11-21: MPL-2.0 accepted for unmodified third-party crates; notice reproduced, published crate source is the corresponding source'
    'OFL-1.1' = 'accepted 2026-11-21 for the unmodified Geist fonts the app bundles (the owner chose Geist; OFL-1.1 permits bundling with the copyright and license retained; fonts are never sold on their own and are not renamed or modified)'
}

. (Join-Path $PSScriptRoot 'lib/spdx.ps1')

function Get-TextHash([string]$Text) {
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
        [Text.Encoding]::UTF8.GetBytes(($Text -replace "`r`n", "`n").Trim())))
}

function Get-RuntimeGraph([string]$Package) {
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
    $package = [regex]::Match($manifest, '(?ms)^\[package\]\r?\n(.*?)(?=^\[|\z)').Groups[1].Value
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

# 2. Evidence for each packaged program. A component shared by several programs is reviewed once.
$graphs = [ordered]@{}
foreach ($p in $Package) { $graphs[$p] = Get-RuntimeGraph $p }
$ids = [Collections.Generic.SortedDictionary[string, bool]]::new()
foreach ($g in $graphs.Values) { foreach ($k in $g.Keys) { $ids[$k] = $g[$k] } }
$notices = [Collections.Generic.List[string]]::new()
$blockerById = @{}
$byId = @{}
foreach ($c in $inventory.components) { $byId[[string]$c.id] = $c }
$clarificationFile = Join-Path $Root 'docs/license-evidence/clarifications.json'
$clarifications = if (Test-Path $clarificationFile) { (Get-Content $clarificationFile -Raw | ConvertFrom-Json -AsHashtable).crates } else { @{} }

function Add-Notice([string]$Key, [string]$Text) {
    $notices.Add("<!-- notice:$Key -->`n$Text`n<!-- /notice:$Key -->")
    Get-TextHash $Text
}

# Verbatim text from an evidence file; refuses a file whose hash is not the reviewed one.
function Get-EvidenceFiles($Entry) {
    foreach ($f in $Entry.files) {
        $path = Join-Path $Root "docs/license-evidence/$($f.path)"
        $actual = (Get-FileHash $path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -ne $f.sha256) { throw "License evidence $($f.path) does not match its recorded SHA-256" }
        [pscustomobject]@{ Name = "$($f.path) (from $($f.source))"; FullName = $path }
    }
}

$workspaceIds = @()
foreach ($id in @($ids.Keys)) {
    $m = [regex]::Match($id, '^cargo:(.+)@([^@]+)$')
    $name = $m.Groups[1].Value; $version = $m.Groups[2].Value
    $c = $byId[$id]
    if ($ids[$id]) { $workspaceIds += $id; continue }
    $ev = Get-CrateEvidence $name $version
    if ($ev.Problem) { $blockerById[$id] = "${id}: $($ev.Problem)"; continue }
    $c.license = $ev.License
    $accepted = $AcceptedCopyleft.ContainsKey($ev.License)
    if (-not $accepted -and -not (Test-PermissiveExpression $ev.License)) { $blockerById[$id] = "${id}: license '$($ev.License)' needs a human decision (not all-permissive)"; continue }
    $files = @($ev.Files)
    $how = $(if ($files.Count) { 'verbatim license files from the checksum-verified crate source' } else { 'license text from source-file headers of the checksum-verified crate (no LICENSE file shipped)' })
    if ($clarifications.ContainsKey($id)) {
        # The crate ships no usable text of its own, or ships less than it links (a vendored library).
        $files = @($files) + @(Get-EvidenceFiles $clarifications[$id])
        $how = "$how plus upstream license text kept in docs/license-evidence (SHA-256 checked)"
    }
    if ($files.Count -eq 0 -and $ev.Headers.Count -eq 0) { $blockerById[$id] = "${id}: the crate ships no license text to reproduce"; continue }
    $key = "cargo-$name-$version"
    $header = "Crate: $name $version`nDeclared license: $($ev.License)`nSource: https://crates.io/crates/$name/$version (archive checksum matches Cargo.lock)"
    if ($accepted) { $header += "`nCopyleft note: $($AcceptedCopyleft[$ev.License])" }
    $c.notice = $key
    $c.noticeSha256 = Add-Notice $key (New-NoticeText $header $files $ev.Headers)
    $c.status = 'reviewed'
    $c.source = "https://crates.io/api/v1/crates/$name/$version/download (checksum from Cargo.lock)"
    $c.licenseSource = $how + $(if ($accepted) { '; ' + $AcceptedCopyleft[$ev.License] } else { '; permissive terms; automated under owner-approved policy' })
}

# 2b. The built frontend that the program embeds: npm packages compiled into the JavaScript, and the
# fonts it emits. Derived from a real build (scripts/frontend-bundle.mjs), never from a list.
$frontendIds = @()
$frontendFiles = @()
if ($FrontendPackage) {
    if ($Package -notcontains $FrontendPackage) { throw "-FrontendPackage $FrontendPackage must also be in -Package" }
    $appPath = Join-Path $Root $FrontendDir
    $raw = & node (Join-Path $PSScriptRoot 'frontend-bundle.mjs') $appPath
    if ($LASTEXITCODE -ne 0) { throw 'the frontend bundle could not be built and listed' }
    $bundle = ($raw -join "`n") | ConvertFrom-Json -AsHashtable
    $lockText = Get-Content (Join-Path $appPath 'pnpm-lock.yaml') -Raw
    $licenseName = '(?i)^(LICENSE|LICENCE|COPYING|NOTICE)([-_.].*)?$'

    function Get-LockIntegrity([string]$Name, [string]$Version) {
        $m = [regex]::Match($lockText, "(?m)^\s+'?$([regex]::Escape("$Name@$Version"))[^\r\n:]*:\r?\n\s+resolution: \{integrity: (sha512-[A-Za-z0-9+/=]+)")
        if ($m.Success) { $m.Groups[1].Value } else { $null }
    }
    function Add-NpmComponent([string]$Name, [string]$Version, [string]$Dir, [string]$Declared, [string]$Extra) {
        $id = "npm:$Name@$Version"
        $c = $byId[$id]
        if (-not $c) {
            # A transitive package: compiled into the bundle but not declared in app/package.json.
            $c = [ordered]@{ id = $id; license = 'UNKNOWN'; source = ''; licenseSource = ''; status = 'unresolved'; notice = ''; noticeSha256 = '' }
            $inventory.components = @($inventory.components) + $c
            $byId[$id] = $c
        }
        $files = @(Get-ChildItem $Dir -File | Where-Object { $_.Name -match $licenseName -and $_.Length -lt 200KB -and $_.Name -notmatch '\.spdx$' } | Sort-Object Name)
        $license = $Declared
        $fromFile = $false
        if (-not $license -and $files.Count -eq 1) {
            # package.json declares no license; accept only a file that is plainly the MIT text.
            $t = [IO.File]::ReadAllText($files[0].FullName)
            if ($t -match '^\s*MIT License' -and $t -match 'Permission is hereby granted, free of charge') { $license = 'MIT'; $fromFile = $true }
        }
        if (-not $license) { $blockerById[$id] = "${id}: no license declared and the license file is not recognisably permissive"; return $id }
        $c.license = $license
        $accepted = $AcceptedCopyleft.ContainsKey($license)
        if (-not $accepted -and -not (Test-PermissiveExpression $license)) { $blockerById[$id] = "${id}: license '$license' needs a human decision (not all-permissive)"; return $id }
        if ($files.Count -eq 0) { $blockerById[$id] = "${id}: the package ships no license text to reproduce"; return $id }
        $integrity = Get-LockIntegrity $Name $Version
        if (-not $integrity) { $blockerById[$id] = "${id}: pnpm-lock.yaml records no integrity for this package"; return $id }
        $key = 'npm-' + (($Name -replace '^@', '') -replace '/', '-') + "-$Version"
        $header = "Package: $Name $Version`n$(if ($fromFile) { 'License (read from the license file; package.json declares none)' } else { 'Declared license' }): $license`nSource: https://registry.npmjs.org/$Name/$Version (integrity $integrity from app/pnpm-lock.yaml)"
        if ($accepted) { $header += "`nNote: $($AcceptedCopyleft[$license])" }
        if ($Extra) { $header += "`n$Extra" }
        $c.notice = $key
        $c.noticeSha256 = Add-Notice $key (New-NoticeText $header $files)
        $c.status = 'reviewed'
        $c.source = "https://registry.npmjs.org/$Name/$Version"
        $c.licenseSource = 'verbatim license files from the installed package, whose integrity is recorded in app/pnpm-lock.yaml; ' + $(if ($accepted) { $AcceptedCopyleft[$license] } else { 'permissive terms; automated under owner-approved policy' })
        $id
    }

    foreach ($p in $bundle.packages) {
        $frontendIds += Add-NpmComponent $p.name $p.version (Join-Path $appPath ($p.dir -replace '^node_modules/', 'node_modules/')) $p.license ''
    }

    # Fonts: match each emitted font file to the installed font package that holds identical bytes.
    $fontPackages = @(Get-ChildItem (Join-Path $appPath 'node_modules/@fontsource-variable') -Directory -ErrorAction SilentlyContinue)
    $fontHashes = @{}
    foreach ($fp in $fontPackages) {
        $pj = Get-Content (Join-Path $fp.FullName 'package.json') -Raw | ConvertFrom-Json -AsHashtable
        foreach ($f in (Get-ChildItem (Join-Path $fp.FullName 'files') -File -ErrorAction SilentlyContinue)) {
            $fontHashes[(Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLowerInvariant()] = @{ Package = $pj; Dir = $fp.FullName; File = $f.Name }
        }
    }
    $usedFonts = @{}
    foreach ($e in @($bundle.emitted | Where-Object { $_.file -match '\.(woff2?|ttf|otf)$' })) {
        $hit = $fontHashes[$e.sha256]
        if (-not $hit) { $blockerById["font:$($e.file)"] = "font $($e.file) matches no installed font package byte for byte (modified or unidentified)"; continue }
        $key = "$($hit.Package.name)@$($hit.Package.version)"
        if (-not $usedFonts.ContainsKey($key)) { $usedFonts[$key] = @{ Info = $hit; Files = @() } }
        $usedFonts[$key].Files += "$($e.file) = $($hit.File) (SHA-256 $($e.sha256))"
    }
    foreach ($key in ($usedFonts.Keys | Sort-Object)) {
        $u = $usedFonts[$key]
        $extra = "Font files redistributed in the app (byte-identical to the package files; unmodified, not renamed):`n" + (($u.Files | Sort-Object) -join "`n")
        $frontendIds += Add-NpmComponent $u.Info.Package.name $u.Info.Package.version $u.Info.Dir $u.Info.Package.license $extra
    }
    $frontendFiles = @("$FrontendDir/package.json", "$FrontendDir/pnpm-lock.yaml")
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
$inventory.scopes = [ordered]@{}
$allBlockers = [Collections.Generic.List[string]]::new()
foreach ($p in $Package) {
    $mine = @($graphs[$p].Keys | Where-Object { $blockerById.ContainsKey($_) } | ForEach-Object { $blockerById[$_] })
    foreach ($b in $mine) { if (-not $allBlockers.Contains($b)) { $allBlockers.Add($b) } }
    $extra = @($stdId)
    $scopeFiles = @()
    if ($p -eq $FrontendPackage) {
        $extra += $frontendIds
        $scopeFiles = $frontendFiles
        $mine = @($mine) + @($blockerById.Keys | Where-Object { $_ -like 'npm:*' -or $_ -like 'font:*' } | ForEach-Object { $blockerById[$_] })
        foreach ($b in $mine) { if (-not $allBlockers.Contains($b)) { $allBlockers.Add($b) } }
    }
    $inventory.scopes[$p] = [ordered]@{
        target = $Target
        extra = $extra
        files = $scopeFiles
        rustStdCopyrightSha256 = $stdHash
        blockers = @($mine)
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
$generated = @($marker, "## Notices for $($Package -join ', ') (generated by scripts/generate-package-notices.ps1)", '') + $notices
[IO.File]::WriteAllText($noticePath, (@($head, '') + $generated -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
Write-Host "Scopes $($Package -join ', ') ($Target): $($ids.Count) cargo identities + rust std; $($allBlockers.Count) blockers."
foreach ($b in $allBlockers) { Write-Host "  BLOCKER: $b" }
Write-Host "Rust std third-party notices (ship as licenses/rust-std-COPYRIGHT.html): $stdCopyright"
