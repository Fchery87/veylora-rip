# Shared SPDX helpers. Dot-source this file.
$permissive = @(
    'MIT', 'MIT-0', 'Apache-2.0', 'BSD-2-Clause', 'BSD-3-Clause', 'ISC', 'Zlib', '0BSD', 'Unlicense', 'CC0-1.0',
    'BSL-1.0', 'Unicode-3.0', 'Unicode-DFS-2016', 'NCSA'
)
$allowedExceptions = @('LLVM-exception')

# SPDX expression evaluation: can the expression be satisfied using permissive licenses only?
function Test-PermissiveExpression([string]$Expression) {
    $tokens = [regex]::Matches($Expression.Replace('/', ' OR '), '\(|\)|[A-Za-z0-9.\-+]+') | ForEach-Object { $_.Value }
    $script:pos = 0
    $script:toks = @($tokens)
    function Peek { if ($script:pos -lt $script:toks.Count) { $script:toks[$script:pos] } else { $null } }
    function Take { $t = Peek; $script:pos++; $t }
    function ParseOr {
        $result = ParseAnd
        while ((Peek) -eq 'OR') { [void](Take); $right = ParseAnd; $result = ($result -or $right) }
        $result
    }
    function ParseAnd {
        $result = ParseAtom
        while ((Peek) -eq 'AND') { [void](Take); $right = ParseAtom; $result = ($result -and $right) }
        $result
    }
    function ParseAtom {
        $t = Take
        if ($t -eq '(') { $v = ParseOr; if ((Take) -ne ')') { throw "Unbalanced SPDX expression: $Expression" }; return $v }
        if (-not $t) { throw "Unexpected end of SPDX expression: $Expression" }
        $ok = $permissive -contains $t
        if ((Peek) -eq 'WITH') { [void](Take); $ex = Take; $ok = $ok -and ($allowedExceptions -contains $ex) }
        $ok
    }
    $value = ParseOr
    if ($script:pos -ne $script:toks.Count) { throw "Unparsed SPDX expression: $Expression" }
    [bool]$value
}
