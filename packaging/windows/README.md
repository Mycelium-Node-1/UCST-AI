# Windows MSI Installer

This directory contains a **WiX v4-schema** MSI definition for the 64-bit Windows build of Finalized Game Engine 0.1.0. The build helper regenerates a local `staging/` payload from the current Rust workspace, canonical SphereWorld sample, package metadata, license, checksums, and runtime-facing documentation; it then builds an MSI from that generated payload. It intentionally excludes generated meshes, GPU buffers, source snapshots, and staged/output files from source control because those are not runtime world authority or reproducible source assets.

> The installer deploys the application into `Program Files\Finalized Game Engine`. The installed `worlds\sphere-world-basic.sphereworld.json` remains canonical data; derived viewport and mesh artifacts are generated only at runtime.

## Installer inputs

| Payload | Installed destination | Purpose |
|---|---|---|
| Generated `staging/bin/windows/Finalized-Game-Engine.exe` | Application root | Native 64-bit HDGE Studio executable. |
| Generated `staging/worlds/sphere-world-basic.sphereworld.json` | `worlds\` | Checked `sphere-world/v1` sample. |
| Generated `staging/config/package.json` | `config\` | Non-authoritative package metadata. |
| Generated `staging/docs/*.md` | `docs\` | Studio, SphereWorld, architecture, and backend guidance. |
| Generated `staging/README.md`, `VERSION`, `SHA256SUMS.txt` | Application root | Package identity and integrity information. |
| Generated `staging/licenses/LICENSE-MIT.txt` | `licenses\` | License notice. |

The authoring model follows WiX's SDK project workflow: the WiX documentation illustrates a `.wixproj` using `WixToolset.Sdk` and a standard `dotnet build` invocation.[1] The MSI uses `MajorUpgrade` with a stable `UpgradeCode`; WiX uses that code and the package version to detect related installed releases and prevent downgrades by default.[2]

## Build on Windows

Open PowerShell in the repository root and run:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\packaging\windows\Build-Windows-Installer.ps1
```

The helper first verifies every installer input, then uses `wix build` if the WiX CLI is already on `PATH`. If the CLI is unavailable, it falls back to `dotnet build` on `wix\FinalizedGameEngine.wixproj`; the WiX SDK package is restored automatically by the .NET SDK. The resulting MSI is written to:

```text
packaging\windows\output\Finalized-Game-Engine-0.1.0-x64.msi
```

To force the SDK path even when the WiX CLI is installed, run:

```powershell
.\packaging\windows\Build-Windows-Installer.ps1 -UseSdkProject
```

## Install, upgrade, and uninstall

Run the generated MSI from File Explorer or from an elevated PowerShell session. The package is per-machine, so Windows Installer may request administrator approval. The installer creates a Start Menu shortcut at **Schoff Research Program > Finalized Game Engine** and registers standard Windows uninstall information. A newer release must retain the `UpgradeCode` in `FinalizedGameEngine.wxs` while increasing the `Version`; the `MajorUpgrade` rule then handles replacement. Do not change that `UpgradeCode` for ordinary upgrades.[2]

Uninstall through **Settings > Apps > Installed apps**, **Control Panel > Programs and Features**, or with:

```powershell
msiexec /x packaging\windows\output\Finalized-Game-Engine-0.1.0-x64.msi
```

## Pre-release verification

Before publishing an MSI, validate the package inputs, run the SphereWorld pipeline suite, build the MSI on a 64-bit Windows test machine, install it, start the application from the Start Menu, load the checked world, uninstall it, and verify the application directory and Start Menu shortcut were removed. Signing is not configured in this package; obtain and apply an organization-controlled code-signing certificate before public distribution.

## References

[1]: https://docs.firegiant.com/quick-start/ "FireGiant Docs: Create your first installation package"
[2]: https://docs.firegiant.com/wix/schema/wxs/majorupgrade/ "FireGiant Docs: MajorUpgrade element"
