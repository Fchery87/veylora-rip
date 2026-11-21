#Requires -Version 7.2
<#
.SYNOPSIS
Starts the desktop app for real and checks that it shows its own embedded pages.

.DESCRIPTION
A release app that was built without tauri/custom-protocol loads the development server instead of
its embedded frontend and shows nothing on any other machine. The development address is also
present as plain text in a correct build, so the binary cannot be inspected for it; the running
app can. This starts the executable against an empty scratch workspace with the WebView2 debugging
port open on loopback, reads the page list, requires the page to be served from the app's own
origin (tauri.localhost) with the expected title, and always stops the app again.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Exe,
    [int]$Port = 9333,
    [int]$TimeoutSeconds = 45
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not (Test-Path -LiteralPath $Exe -PathType Leaf)) { throw "$Exe does not exist" }
$work = Join-Path ([IO.Path]::GetTempPath()) ('veylora-launch-' + [guid]::NewGuid())
New-Item $work -ItemType Directory | Out-Null
$proc = $null
$saved = @{ W = $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; V = $env:VEYLORA_WORKSPACE; D = $env:WEBVIEW2_USER_DATA_FOLDER }
try {
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$Port --remote-debugging-address=127.0.0.1"
    $env:WEBVIEW2_USER_DATA_FOLDER = Join-Path $work 'webview'
    $env:VEYLORA_WORKSPACE = Join-Path $work 'workspace'
    $proc = Start-Process -FilePath $Exe -PassThru
    $page = $null
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while ([DateTime]::UtcNow -lt $deadline -and -not $page) {
        if ($proc.HasExited) { throw "the app exited early with code $($proc.ExitCode)" }
        try {
            $pages = @(Invoke-RestMethod -Uri "http://127.0.0.1:$Port/json" -TimeoutSec 2)
            $page = $pages | Where-Object { $_.type -eq 'page' -and $_.title -eq 'Veylora RIP' } | Select-Object -First 1
        } catch { }
        if (-not $page) { Start-Sleep -Milliseconds 500 }
    }
    if (-not $page) { throw "no 'Veylora RIP' page appeared within $TimeoutSeconds s" }
    if ($page.url -notmatch '^https?://tauri\.localhost/' -and $page.url -notmatch '^tauri://localhost') { throw "the app is not serving its embedded pages: $($page.url)" }
    Write-Host "APP LAUNCH OK: $($page.url) titled '$($page.title)'"
} finally {
    if ($proc -and -not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $saved.W; $env:VEYLORA_WORKSPACE = $saved.V; $env:WEBVIEW2_USER_DATA_FOLDER = $saved.D
    Start-Sleep -Milliseconds 500
    Remove-Item $work -Recurse -Force -ErrorAction SilentlyContinue
}
