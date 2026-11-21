#Requires -Version 7.2
[CmdletBinding()]
param(
    [string]$Root = (Split-Path $PSScriptRoot -Parent),
    [switch]$RefreshUnreviewedInventory,
    # Check only what one packaged program links (see docs/dependency-inventory.md, "Release scopes").
    [string]$Package,
    [string]$Target = 'x86_64-pc-windows-msvc'
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Get-TextHash([string]$Text) {
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
        [Text.Encoding]::UTF8.GetBytes(($Text -replace "`r`n", "`n").Trim())))
}

function Get-EvidenceFiles {
    $paths = @(& git -C $Root ls-files --cached --others --exclude-standard)
    if ($LASTEXITCODE -ne 0) { throw 'Cannot enumerate repository evidence with git' }
    @($paths | Where-Object {
        $_ -match '(^|/)(Cargo\.(toml|lock)|package\.json|pnpm-lock\.yaml|package-lock\.json|yarn\.lock|tauri\.conf\.json|build\.rs)$' -or
        $_ -match '\.(icc|icm|ttf|otf|woff2?|png|jpe?g|svg|ico|icns|dll|exe|lib|a|so|dylib)$' -or
        $_ -match '(^fixtures/|/tests/golden/|^\.github/workflows/)' -or
        $_ -eq 'app/src/main.ts'
    } | Sort-Object -Unique)
}

function Get-Dependencies {
    param([switch]$FrontendLockOnly)
    if (-not $FrontendLockOnly -and (Test-Path "$Root/Cargo.lock")) {
        foreach ($block in ((Get-Content "$Root/Cargo.lock" -Raw) -split '\[\[package\]\]')) {
            $name = [regex]::Match($block, '(?m)^name = "([^"]+)"').Groups[1].Value
            $version = [regex]::Match($block, '(?m)^version = "([^"]+)"').Groups[1].Value
            if ($name) { "cargo:$name@$version" }
        }
    }
    foreach ($file in (Get-EvidenceFiles | Where-Object { $_ -match '(^|/)package.json$' })) {
        if ($FrontendLockOnly) { continue }
        $manifest = Get-Content (Join-Path $Root $file) -Raw | ConvertFrom-Json -AsHashtable
        foreach ($section in @('dependencies', 'devDependencies', 'optionalDependencies', 'peerDependencies')) {
            if ($manifest.ContainsKey($section)) {
                foreach ($name in $manifest[$section].Keys) { "npm:$name@$($manifest[$section][$name])" }
            }
        }
        if ($manifest.ContainsKey('packageManager')) { "npm:$($manifest.packageManager)" }
    }
    foreach ($file in (Get-EvidenceFiles | Where-Object { $_ -match 'pnpm-lock.yaml$' })) {
        $text = Get-Content (Join-Path $Root $file) -Raw
        if ($text -notmatch "lockfileVersion: '?9\.0'?") { throw "Unsupported pnpm lock schema: $file" }
        $packages = [regex]::Match($text, '(?ms)^packages:\r?\n(.*?)(?=^\S|\z)').Groups[1].Value
        foreach ($match in [regex]::Matches($packages, '(?m)^  [''"]?([^\r\n]+?)[''"]?:\s*$')) {
            "npm:$($match.Groups[1].Value.Trim("'", '"'))"
        }
    }
}

function Get-RuntimeGraph {
    # Exact identities (and workspace manifests) of what $Package links for $Target, as cargo resolves it.
    $lines = @(& cargo tree --manifest-path (Join-Path $Root 'Cargo.toml') -p $Package -e normal --target $Target --offline --locked --prefix none --format '{p}' 2>&1)
    if ($LASTEXITCODE -ne 0) { throw "cargo tree failed: $($lines -join ' ')" }
    $ids = [Collections.Generic.SortedSet[string]]::new()
    $manifests = [Collections.Generic.SortedSet[string]]::new()
    foreach ($line in $lines) {
        $m = [regex]::Match([string]$line, '^(\S+) v(\S+)(?: \((?!\*\))([^)]+)\))?')
        if (-not $m.Success) { throw "Unrecognised cargo tree line: $line" }
        [void]$ids.Add("cargo:$($m.Groups[1].Value)@$($m.Groups[2].Value)")
        $where = $m.Groups[3].Value
        if ($where -and $where -notmatch '^(registry|git)\+|proc-macro') {
            $full = [IO.Path]::GetFullPath($where)
            $rootFull = [IO.Path]::GetFullPath($Root).TrimEnd('\', '/')
            if (-not $full.StartsWith($rootFull, [StringComparison]::OrdinalIgnoreCase)) { throw "Path dependency outside the repository: $where" }
            [void]$manifests.Add(($full.Substring($rootFull.Length + 1).Replace('\', '/') + '/Cargo.toml'))
        }
    }
    [pscustomobject]@{ Ids = @($ids); Manifests = @($manifests) }
}

function Test-ComponentReviewed($component, [string]$Notices, [Collections.Generic.List[string]]$Failures) {
    foreach ($field in @('id', 'license', 'source', 'licenseSource', 'status', 'notice', 'noticeSha256')) {
        if (-not $component.ContainsKey($field) -or [string]::IsNullOrWhiteSpace($component[$field])) {
            $Failures.Add("Missing $field metadata: $($component.id)")
        }
    }
    $id = [string]$component.id
    if ($id -match '(?i)(acro[ -]?rip|print[ -]?fab|digital[ -]?factory|cadlink)') { $Failures.Add("Forbidden commercial RIP: $id") }
    if ($component.license -match '(?i)(UNKNOWN|UNLICENSED|NOASSERTION|UNRESOLVED)' -or $component.status -ne 'reviewed') {
        $Failures.Add("Unresolved license review: $id")
    }
    $key = [regex]::Escape([string]$component.notice)
    $notice = [regex]::Matches($Notices, "(?s)<!-- notice:$key -->(.*?)<!-- /notice:$key -->")
    if ($notice.Count -ne 1 -or [string]::IsNullOrWhiteSpace($notice[0].Groups[1].Value)) { $Failures.Add("Missing notice: $id") }
    elseif ((Get-TextHash $notice[0].Groups[1].Value) -ne $component.noticeSha256) { $Failures.Add("Notice hash mismatch: $id") }
}

function Invoke-PackageCheck {
    $failures = [Collections.Generic.List[string]]::new()
    foreach ($required in @('docs/dependency-inventory.md', 'THIRD_PARTY_NOTICES.md')) {
        if (-not (Test-Path "$Root/$required" -PathType Leaf)) { throw "Missing required file: $required" }
    }
    $text = Get-Content "$Root/docs/dependency-inventory.md" -Raw
    $blocks = [regex]::Matches($text, '(?ms)^```json license-inventory\r?\n(.*?)^```\s*$')
    if ($blocks.Count -ne 1) { throw 'Expected one json license-inventory block' }
    $inventory = $blocks[0].Groups[1].Value | ConvertFrom-Json -AsHashtable
    if ($inventory.schema -ne 1) { throw 'Unsupported inventory schema' }
    if (-not $inventory.ContainsKey('scopes') -or -not $inventory.scopes.ContainsKey($Package)) { throw "The inventory has no release scope for $Package" }
    $scope = $inventory.scopes[$Package]
    if ($scope.target -ne $Target) { throw "The inventory scope for $Package is for $($scope.target), not $Target" }

    $graph = Get-RuntimeGraph
    # Every Rust program links the standard library of the toolchain that builds it, so that
    # component is required whatever the scope lists; a toolchain change must be reviewed.
    $rustVersion = [regex]::Match([string](& rustc -V), 'rustc (\S+)').Groups[1].Value
    if (-not $rustVersion) { throw 'Cannot determine the rustc version' }
    $required = @($graph.Ids) + @($scope.extra) + @("toolchain:rust-std@$rustVersion") | Sort-Object -Unique
    $notices = Get-Content "$Root/THIRD_PARTY_NOTICES.md" -Raw
    $components = @{}
    foreach ($c in $inventory.components) { $components[[string]$c.id] = $c }
    foreach ($id in $required) {
        if (-not $components.ContainsKey($id)) { $failures.Add("Uninventoried dependency: $id"); continue }
        Test-ComponentReviewed $components[$id] $notices $failures
    }
    # The review is bound to the exact lock and manifests it was done against.
    $evidenceHashes = @{}
    foreach ($f in $inventory.files) { $evidenceHashes[[string]$f.path] = [string]$f.sha256 }
    # Files the scope names (for the app: the frontend lock and manifest) are bound the same way.
    $scopeFiles = if ($scope.ContainsKey('files')) { @($scope.files | Where-Object { $_ }) } else { @() }
    foreach ($path in (@('Cargo.lock') + @($graph.Manifests) + $scopeFiles | Sort-Object -Unique)) {
        if (-not $evidenceHashes.ContainsKey($path)) { $failures.Add("Uninventoried file: $path") }
        elseif (-not (Test-Path "$Root/$path" -PathType Leaf)) { $failures.Add("Missing evidence: $path") }
        elseif ((Get-FileHash "$Root/$path" -Algorithm SHA256).Hash -ne $evidenceHashes[$path]) { $failures.Add("Inventory drift: $path") }
    }
    foreach ($blocker in $scope.blockers) { $failures.Add("Unresolved review blocker: $blocker") }
    if ($graph.Ids.Count -eq 0) { $failures.Add('No dependency graph found') }
    if ($failures.Count) {
        foreach ($failure in $failures) { Write-Output "BLOCKED: $failure" }
        Write-Output "License check for $Package failed: $($failures.Count) blockers. No distribution clearance."
        exit 1
    }
    Write-Output "License check for $Package passed: $($required.Count) reviewed components. Verify packaged notices separately."
    exit 0
}

try {
    $Root = (Resolve-Path $Root).Path
    if ($Package) { Invoke-PackageCheck }
    $evidence = @(Get-EvidenceFiles)
    $dependencies = @(Get-Dependencies | Sort-Object -Unique)
    if ($RefreshUnreviewedInventory) {
        # Explicit regeneration only. It cannot approve any component or grant rights.
        $records = foreach ($id in $dependencies) {
            $license = 'UNKNOWN'
            $source = 'app/package.json or app/pnpm-lock.yaml'
            $licenseSource = 'UNRESOLVED: inspect the exact published package LICENSE and package.json'
            if ($id -match '^cargo:(.+)@([^@]+)$') {
                $name = $Matches[1]
                $version = $Matches[2]
                $source = "https://docs.rs/crate/$name/$version/source/Cargo.toml"
                $licenseSource = "https://docs.rs/crate/$name/$version/source/ (inspect LICENSE*, COPYING*, NOTICE*)"
                $cache = @(Get-ChildItem "$HOME/.cargo/registry/src/*/$name-$version/Cargo.toml" -ErrorAction SilentlyContinue)
                if ($cache.Count -eq 1) {
                    $declared = [regex]::Match((Get-Content $cache[0] -Raw), '(?m)^license = "([^"]+)"').Groups[1].Value
                    if ($declared) { $license = $declared }
                }
                if ($name -like 'veylora-*') {
                    $license = 'UNLICENSED'
                    $source = 'Cargo.toml [workspace.package] and member Cargo.toml'
                    $licenseSource = 'UNRESOLVED: project owner distribution authorization'
                }
            } elseif ($id -match '^npm:(.+)@([^@]+)$') {
                $source = "https://registry.npmjs.org/$($Matches[1])/$($Matches[2])"
                $licenseSource = 'UNRESOLVED: exact package tarball LICENSE/NOTICE and nested bundled dependencies'
            }
            [ordered]@{ id=$id; license=$license; source=$source; licenseSource=$licenseSource; status='unresolved'; notice=''; noticeSha256='' }
        }
        $files = foreach ($path in $evidence) {
            [ordered]@{ path=$path; sha256=(Get-FileHash (Join-Path $Root $path) -Algorithm SHA256).Hash }
        }
        $inventory = [ordered]@{ schema=1; files=@($files); components=@($records); blockers=@(
            'Review every exact package license, copyright, NOTICE and redistribution obligation. Manifest declarations are not clearance.',
            'Frontend lock contains pnpm only, not the declared application graph. Resolve and review the complete frontend lock before release.',
            'Review native libraries, external tools, profiles, fonts and fixture rights described above. Verify final installer and About notices.',
            'Project workspace license is UNLICENSED. Obtain explicit project distribution authorization.'
        ) }
        $path = "$Root/docs/dependency-inventory.md"
        $existing = if (Test-Path $path) { Get-Content $path -Raw } else { '# Dependency inventory' }
        $prose = ($existing -split '(?m)^<!-- machine-inventory -->')[0].TrimEnd()
        $json = $inventory | ConvertTo-Json -Depth 10
        $document = @($prose, '', '<!-- machine-inventory -->', '```json license-inventory', $json, '```', '') -join "`n"
        [IO.File]::WriteAllText($path, $document, [Text.UTF8Encoding]::new($false))
        Write-Host "Wrote UNRESOLVED inventory: $($records.Count) components, $($files.Count) evidence files. This does not pass the release gate."
        exit 0
    }

    $failures = [Collections.Generic.List[string]]::new()
    $lockedFrontend = @(Get-Dependencies -FrontendLockOnly)
    foreach ($id in ($dependencies | Where-Object { $_ -like 'npm:*' })) {
        if ($id -notin $lockedFrontend) { $failures.Add("Frontend dependency absent from lock: $id") }
    }
    foreach ($required in @('docs/dependency-inventory.md', 'docs/profile-and-font-licensing.md', 'THIRD_PARTY_NOTICES.md')) {
        if (-not (Test-Path "$Root/$required" -PathType Leaf)) { $failures.Add("Missing required file: $required") }
        elseif ([string]::IsNullOrWhiteSpace((Get-Content "$Root/$required" -Raw))) { $failures.Add("Empty required file: $required") }
    }
    if (-not (Test-Path "$Root/docs/dependency-inventory.md")) { throw ($failures -join "`n") }
    $text = Get-Content "$Root/docs/dependency-inventory.md" -Raw
    $blocks = [regex]::Matches($text, '(?ms)^```json license-inventory\r?\n(.*?)^```\s*$')
    if ($blocks.Count -ne 1) { throw 'Expected one json license-inventory block' }
    $inventory = $blocks[0].Groups[1].Value | ConvertFrom-Json -AsHashtable
    if ($inventory.schema -ne 1) { throw 'Unsupported inventory schema' }
    $knownFiles = @{}
    foreach ($file in $inventory.files) {
        if (-not $file.path -or $knownFiles.ContainsKey($file.path)) { throw 'Missing or duplicate evidence path' }
        if ($file.path -match '(^/|^[A-Za-z]:|(^|[\\/])\.\.([\\/]|$))') { throw 'Unsafe evidence path' }
        $knownFiles[$file.path] = $true
        if (-not (Test-Path "$Root/$($file.path)" -PathType Leaf)) { $failures.Add("Missing evidence: $($file.path)") }
        elseif ((Get-FileHash "$Root/$($file.path)" -Algorithm SHA256).Hash -ne $file.sha256) { $failures.Add("Inventory drift: $($file.path)") }
    }
    foreach ($file in $evidence) {
        if (-not $knownFiles.ContainsKey($file)) { $failures.Add("Uninventoried file: $file") }
        if ($file -match '(package-lock.json|yarn.lock)$') { $failures.Add("Unsupported lock format requires review: $file") }
    }
    $notices = if (Test-Path "$Root/THIRD_PARTY_NOTICES.md") { Get-Content "$Root/THIRD_PARTY_NOTICES.md" -Raw } else { '' }
    $ids = @{}
    foreach ($component in $inventory.components) {
        foreach ($field in @('id', 'license', 'source', 'licenseSource', 'status', 'notice', 'noticeSha256')) {
            if (-not $component.ContainsKey($field) -or [string]::IsNullOrWhiteSpace($component[$field])) {
                $failures.Add("Missing $field metadata: $($component.id)")
            }
        }
        $id = [string]$component.id
        if (-not $id -or $ids.ContainsKey($id)) { throw 'Missing or duplicate component ID' }
        $ids[$id] = $true
        if ($id -match '(?i)(acro[ -]?rip|print[ -]?fab|digital[ -]?factory|cadlink)') { $failures.Add("Forbidden commercial RIP: $id") }
        if ($component.license -match '(?i)(UNKNOWN|UNLICENSED|NOASSERTION|UNRESOLVED)' -or $component.status -ne 'reviewed') {
            $failures.Add("Unresolved license review: $id")
        }
        $key = [regex]::Escape([string]$component.notice)
        $notice = [regex]::Matches($notices, "(?s)<!-- notice:$key -->(.*?)<!-- /notice:$key -->")
        if ($notice.Count -ne 1 -or [string]::IsNullOrWhiteSpace($notice[0].Groups[1].Value)) { $failures.Add("Missing notice: $id") }
        elseif ((Get-TextHash $notice[0].Groups[1].Value) -ne $component.noticeSha256) { $failures.Add("Notice hash mismatch: $id") }
    }
    foreach ($id in $dependencies) {
        if (-not $ids.ContainsKey($id)) { $failures.Add("Uninventoried dependency: $id") }
    }
    foreach ($blocker in $inventory.blockers) { $failures.Add("Unresolved review blocker: $blocker") }
    if ($dependencies.Count -eq 0) { $failures.Add('No dependency graph found') }
    if ($failures.Count) {
        foreach ($failure in $failures) { Write-Output "BLOCKED: $failure" }
        Write-Output "License check failed: $($failures.Count) blockers. No distribution clearance."
        exit 1
    }
    Write-Output "License check passed: $($inventory.components.Count) reviewed components. Verify packaged notices separately."
    exit 0
} catch {
    Write-Output "License check failed: $($_.Exception.Message)"
    exit 1
}
