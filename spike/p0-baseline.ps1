#requires -Version 7.0
<#
Read-only P0 inventory. Emits sanitized JSON; never executes component binaries,
queries window titles/command lines, or enumerates unrelated process images.
Redirect the output to a NEW evidence file. Paths below are an explicit allowlist.
Hashes establish artifact identity, not reproducible-build provenance.
#>
[CmdletBinding()]
param(
    [string]$Repository = (Split-Path $PSScriptRoot -Parent),
    [string]$GlazeRepository = 'D:/dev/glazewm-wt-app-id'
)
$ErrorActionPreference = 'Stop'
$Repository = (Resolve-Path -LiteralPath $Repository).Path
$profileRoot = [Environment]::GetFolderPath('UserProfile')
$managedRoot = Join-Path $profileRoot '.winmakase'
$glzrRoot = Join-Path $profileRoot '.glzr'
function Public-Path([string]$value) {
    if (!$value) { return $value }
    $value = $value.Replace('\', '/')
    foreach ($pair in @(@($Repository, '<repo>'), @($GlazeRepository, '<glaze-source>'), @($profileRoot, '<user>'))) {
        $value = $value.Replace($pair[0].Replace('\', '/'), $pair[1], [StringComparison]::OrdinalIgnoreCase)
    }
    return $value
}
function Git-Value([string]$path, [string[]]$gitArgs) {
    $value = & git -C $path @gitArgs
    if ($LASTEXITCODE -ne 0) { throw "git failed in $(Public-Path $path)" }
    return ($value -join "`n")
}
function Text-Hash([string]$value) {
    return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($value)))
}
function File-Fact([string]$id, [string]$path, [switch]$Executable) {
    if (!(Test-Path -LiteralPath $path -PathType Leaf)) {
        return [ordered]@{ id = $id; path = Public-Path $path; present = $false }
    }
    $file = Get-Item -LiteralPath $path
    $fact = [ordered]@{
        id = $id; path = Public-Path $path; present = $true
        bytes = $file.Length; sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
    }
    if ($Executable) {
        $fact.fileVersion = $file.VersionInfo.FileVersion
        $fact.productVersion = $file.VersionInfo.ProductVersion
        $signature = Get-AuthenticodeSignature -LiteralPath $path
        $fact.signatureStatus = [string]$signature.Status
        # Record only executable manifest snippets, never arbitrary binary strings.
        $content = [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes($path))
        $fact.executionManifestSnippets = @([regex]::Matches($content, '<requestedExecutionLevel\s[^>]+>') | ForEach-Object Value | Sort-Object -Unique)
    }
    return $fact
}
function Acl-Fact([string]$path) {
    if (!(Test-Path -LiteralPath $path)) { return @{ path = Public-Path $path; present = $false } }
    $acl = Get-Acl -LiteralPath $path
    $currentSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $known = @{ 'S-1-5-18' = 'SYSTEM'; 'S-1-5-32-544' = 'Administrators'; 'S-1-5-32-545' = 'Users'; 'S-1-5-11' = 'Authenticated Users'; 'S-1-1-0' = 'Everyone'; 'S-1-3-0' = 'Creator Owner'; $currentSid = 'Current user' }
    $entries = @($acl.Access | ForEach-Object {
        $sid = $_.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value
        [ordered]@{ principal = $(if ($known.ContainsKey($sid)) { $known[$sid] } else { 'Other principal' }); type = [string]$_.AccessControlType; rights = [string]$_.FileSystemRights; inherited = $_.IsInherited }
    })
    return [ordered]@{ path = Public-Path $path; present = $true; inheritanceProtected = $acl.AreAccessRulesProtected; entries = $entries }
}
if (!('P0MonitorSnapshot' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class P0MonitorSnapshot {
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] struct Info {
    public int Size; public Rect Monitor, Work; public uint Flags;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string Device;
  }
  public sealed class Display { public string Device; public Rect Bounds, WorkArea; public bool Primary; public uint DpiX, DpiY; public int DpiResult; }
  delegate bool Callback(IntPtr monitor, IntPtr hdc, IntPtr rectangle, IntPtr data);
  [DllImport("user32.dll")] static extern bool EnumDisplayMonitors(IntPtr hdc, IntPtr clip, Callback cb, IntPtr data);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern bool GetMonitorInfo(IntPtr monitor, ref Info info);
  [DllImport("shcore.dll")] static extern int GetDpiForMonitor(IntPtr monitor, int kind, out uint x, out uint y);
  public static Display[] Read() {
    var values = new List<Display>();
    if (!EnumDisplayMonitors(IntPtr.Zero, IntPtr.Zero, (m,h,r,d) => {
      var info = new Info { Size = Marshal.SizeOf<Info>() };
      if (!GetMonitorInfo(m, ref info)) throw new InvalidOperationException("GetMonitorInfo failed");
      uint x,y; int result = GetDpiForMonitor(m,0,out x,out y);
      values.Add(new Display {Device=info.Device, Bounds=info.Monitor, WorkArea=info.Work, Primary=(info.Flags&1)!=0, DpiX=x,DpiY=y,DpiResult=result});
      return true;
    }, IntPtr.Zero)) throw new InvalidOperationException("EnumDisplayMonitors failed");
    return values.ToArray();
  }
}
'@
}
$artifacts = @(
    File-Fact 'winmakase-cli-deployed' "$managedRoot/bin/winmakase.exe" -Executable
    File-Fact 'winmakase-supervisor-deployed' "$managedRoot/bin/winmakased.exe" -Executable
    File-Fact 'winmakase-cli-local-release' "$Repository/target/release/winmakase.exe" -Executable
    File-Fact 'winmakase-supervisor-local-release' "$Repository/target/release/winmakased.exe" -Executable
    File-Fact 'glaze-deployed' "$managedRoot/bin/glazewm.exe" -Executable
    File-Fact 'glaze-cli-deployed' "$managedRoot/bin/glazewm-cli.exe" -Executable
    File-Fact 'glaze-watcher-deployed' "$managedRoot/bin/glazewm-watcher.exe" -Executable
    File-Fact 'glaze-local-release' "$GlazeRepository/target/release/glazewm.exe" -Executable
    File-Fact 'glaze-cli-local-release' "$GlazeRepository/target/release/glazewm-cli.exe" -Executable
    File-Fact 'glaze-watcher-local-release' "$GlazeRepository/target/release/glazewm-watcher.exe" -Executable
    File-Fact 'kanata-task-path-candidate' "$Repository/spike/tools/kanata/kanata_windows_gui_winIOv2_cmd_allowed_x64.exe" -Executable
    File-Fact 'zebar-installed' 'C:/Program Files/glzr.io/Zebar/zebar.exe' -Executable
    File-Fact 'powertoys-run-installed' 'C:/Program Files/PowerToys/PowerToys.PowerLauncher.exe' -Executable
)
$configs = @(
    File-Fact 'supervisor' "$managedRoot/config.toml"
    File-Fact 'local-keymap' "$managedRoot/keymap/local.toml"
    File-Fact 'glaze-base' "$managedRoot/glazewm/base.yaml"
    File-Fact 'glaze-generated' "$glzrRoot/glazewm/config.yaml"
    File-Fact 'kanata' "$Repository/spike/caps.kbd"
    File-Fact 'panic' "$Repository/spike/panic.ps1"
    File-Fact 'stock-keymap' "$Repository/keymap/omarchy.toml"
    File-Fact 'winmakase-lock' "$Repository/Cargo.lock"
    File-Fact 'glaze-lock' "$GlazeRepository/Cargo.lock"
)
$tasks = @(Get-ScheduledTask -TaskName 'Winmakase*' | ForEach-Object {
    # Exact UTF-8 export hash detects arguments, triggers, principals and settings
    # changes without publishing the private XML. Hashes are machine-specific.
    $xml = Export-ScheduledTask -TaskName $_.TaskName -TaskPath $_.TaskPath
    $actions = @($_.Actions | ForEach-Object { [ordered]@{ executable=Public-Path $_.Execute; argumentsSha256=Text-Hash ([string]$_.Arguments); workingDirectory=Public-Path $_.WorkingDirectory } })
    [ordered]@{ name=$_.TaskName; state=[string]$_.State; runLevel=[string]$_.Principal.RunLevel; logonType=[string]$_.Principal.LogonType; actions=$actions; exportedXmlSha256=Text-Hash $xml; argumentDetails='contents omitted; review exact definitions locally'; restartCount=$_.Settings.RestartCount; restartInterval=$_.Settings.RestartInterval }
})
$names = @('winmakased.exe','glazewm.exe','glazewm-watcher.exe','zebar.exe','PowerToys.PowerLauncher.exe','kanata_windows_gui_winIOv2_cmd_allowed_x64.exe')
$filter = ($names | ForEach-Object { "Name = '$_'" }) -join ' OR '
$processes = @(Get-CimInstance Win32_Process -Filter $filter | ForEach-Object {
    [ordered]@{ name=$_.Name; pid=$_.ProcessId; imagePath=Public-Path $_.ExecutablePath; imagePathReadable=!!$_.ExecutablePath; createdAt=$_.CreationDate.ToUniversalTime().ToString('o') }
})
$os = Get-CimInstance Win32_OperatingSystem
$osRegistry = Get-ItemProperty 'HKLM:/SOFTWARE/Microsoft/Windows NT/CurrentVersion'
$features = @(Get-CimInstance Win32_OptionalFeature | Where-Object Name -match '^(Microsoft-Hyper-V-All|Containers-DisposableClientVM|VirtualMachinePlatform)$' | Select-Object Name,InstallState)
$snapshot = [ordered]@{
    schemaVersion=1; capturedAt=[DateTime]::UtcNow.ToString('o'); readOnly=$true
    sanitization='Allowlisted component images only; no unrelated process paths, titles, command lines, account names, serial numbers or config content.'
    os=[ordered]@{ caption=$os.Caption; version=$os.Version; build=$os.BuildNumber; ubr=$osRegistry.UBR; architecture=$os.OSArchitecture; lastBootUtc=$os.LastBootUpTime.ToUniversalTime().ToString('o') }
    source=[ordered]@{ winmakaseHead=Git-Value $Repository @('rev-parse','HEAD'); winmakaseDirty=!!(Git-Value $Repository @('status','--porcelain')); glazeHead=Git-Value $GlazeRepository @('rev-parse','HEAD'); glazeDirty=!!(Git-Value $GlazeRepository @('status','--porcelain')) }
    artifacts=$artifacts; artifactMethod='Explicit candidate file paths. Compare each path with task/process facts; a matching local release is not proof of source-to-binary provenance. An unreadable elevated process path remains unverified.'; configs=$configs; tasks=$tasks; processes=$processes
    displays=@([P0MonitorSnapshot]::Read()); dpiMethod='GetDpiForMonitor MDT_EFFECTIVE_DPI in caller context; not a mixed-DPI acceptance test.'
    optionalFeatures=$features; optionalFeatureStates=@{ '1'='Enabled'; '2'='Disabled'; '3'='Absent'; '4'='Unknown' }
    acls=@($Repository, (Split-Path $Repository -Parent), "$Repository/spike", "$Repository/spike/tools/kanata", "$Repository/spike/caps.kbd", "$Repository/spike/panic.ps1", $managedRoot, "$managedRoot/bin" | ForEach-Object { Acl-Fact $_ })
    aclMethod='Listed DACL entries, not a complete effective-access/security audit.'
}
$snapshot | ConvertTo-Json -Depth 12
