# Staged P0 Windows Sandbox kit

Prepared 2026-09-08; not launched. Chris enabled the Windows Sandbox feature. The host restart is still pending in the last checked state.

Local launch file: [Winmakase-P0.wsb](file:///D:/temp/Winmakase-P0.wsb). It maps only `D:/dev/winmakase/spike/p0` to `C:/WinmakaseSource` and `D:/temp/winmakase-p0-guest-assets` to `C:/WinmakaseAssets`, both read-only. The XML parses, both source directories exist, and networking, clipboard and printer redirection are disabled. Its logon command runs the guarded inbox-PowerShell fixture initializer, which compiles the four-action test executable and creates local shortcuts. It does not install dependencies, publish shortcuts or start UI.

The asset mapping contains these four archives only. The first three match the pinned downloads; the Glaze ZIP contains the locally built candidate and its manifest:

| Guest filename | Bytes | SHA-256 |
|---|---:|---|
| `zebar-v3.3.1-x64.msi` | 12,197,888 | `1D5584BE8DEF58F828D55ED09CEE52FE80FDD60EA118141F37963AA80E902C28` |
| `kanata-v1.12.0-x64.zip` | 8,484,766 | `13947ED78CFA3284BFEF854E3C542C74AB366236B72FD9F7E039F8638DEEAD9D` |
| `PowerToysUserSetup-0.100.2-x64.exe` | 284,920,432 | `945FDF327E4D38E4CED61B0727B7AB8A1222958782982052989DDF7CB7096F62` |
| `glazewm-p0-371a448-x64.zip` | 6,475,752 | `795E8A0CC44CCC1CF181A1252FBF1AAF44B0DE53F3F6B70ACB14D976DECB4998` |

This is fixture preparation, not a complete offline stack installer. PowerToys bundles the small WebView2 bootstrapper; whether the guest already supplies a usable runtime remains untested. Do not treat the mapped installer as proof that offline installation succeeds. The [reviewed Glaze candidate](direct-caps.md) has a [local build attestation](direct-caps-build.json); its ZIP contains only three release executables and the manifest, with no host configuration or autostart script. It has not been launched.

After restart, verify Sandbox availability and the UI connection, open the staged file, inspect initializer output, then follow the [fixture instructions](../../../../spike/p0/README.md). Keep all stack/crash/taskbar experiments inside the guest. Record installation failures and remaining prerequisites before changing the network setting or adding dependencies. Physical Caps, games and the actual display layout still require separate host acceptance.
