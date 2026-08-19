[CmdletBinding()]
param(
    [switch]$UseSdkProject,
    [switch]$SkipAppBuild
)

$ErrorActionPreference = 'Stop'
$PackagingRoot = $PSScriptRoot
$RepoRoot = (Resolve-Path (Join-Path $PackagingRoot '..\..')).Path
$WixSource = Join-Path $PackagingRoot 'wix\FinalizedGameEngine.wxs'
$WixProject = Join-Path $PackagingRoot 'wix\FinalizedGameEngine.wixproj'
$StagingRoot = Join-Path $PackagingRoot 'staging'
$OutputDirectory = Join-Path $PackagingRoot 'output'
$MsiPath = Join-Path $OutputDirectory 'Finalized-Game-Engine-0.1.0-x64.msi'
$WindowsBinary = Join-Path $RepoRoot 'target\x86_64-pc-windows-gnu\release\hdge-studio.exe'

function Copy-StagedFile {
    param(
        [Parameter(Mandatory = $true)][string]$Source,
        [Parameter(Mandatory = $true)][string]$RelativeDestination
    )
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) {
        throw "Required packaging input is missing: $Source"
    }
    $destination = Join-Path $StagingRoot $RelativeDestination
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination) | Out-Null
    Copy-Item -LiteralPath $Source -Destination $destination -Force
}

if (-not $SkipAppBuild) {
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargo) {
        throw 'cargo was not found. Install Rust and the x86_64-pc-windows-gnu target, or rerun with -SkipAppBuild after creating the Windows executable.'
    }
    Write-Host 'Building the 64-bit Windows HDGE Studio executable...'
    Push-Location $RepoRoot
    try {
        $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = 'x86_64-w64-mingw32-gcc'
        & $cargo.Source build --release -p hdge-studio --target x86_64-pc-windows-gnu
        if ($LASTEXITCODE -ne 0) {
            throw "Rust Windows build failed with exit code $LASTEXITCODE."
        }
    }
    finally {
        Pop-Location
    }
}

if (-not (Test-Path -LiteralPath $WindowsBinary -PathType Leaf)) {
    throw "Windows executable is missing: $WindowsBinary"
}

Remove-Item -LiteralPath $StagingRoot -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $StagingRoot, $OutputDirectory | Out-Null

Copy-StagedFile $WindowsBinary 'bin\windows\Finalized-Game-Engine.exe'
Copy-StagedFile (Join-Path $RepoRoot 'examples\sphere-world-basic\world.sphereworld.json') 'worlds\sphere-world-basic.sphereworld.json'
Copy-StagedFile (Join-Path $PackagingRoot 'package.json') 'config\package.json'
Copy-StagedFile (Join-Path $PackagingRoot 'LICENSE-MIT.txt') 'licenses\LICENSE-MIT.txt'
Copy-StagedFile (Join-Path $PackagingRoot 'Release-README.md') 'README.md'
Copy-StagedFile (Join-Path $RepoRoot 'docs\hdge-studio-0.1.md') 'docs\HDGE-Studio-Guide.md'
Copy-StagedFile (Join-Path $RepoRoot 'docs\sphereworld-lab-0.1.md') 'docs\SphereWorld-Lab-Guide.md'
Copy-StagedFile (Join-Path $RepoRoot 'docs\sphereworld-slice-0.md') 'docs\SphereWorld-Slice-0.md'
Copy-StagedFile (Join-Path $RepoRoot 'docs\milestone-3-hdge-blueprint.md') 'docs\HDGE-Architecture-Blueprint.md'
Copy-StagedFile (Join-Path $RepoRoot 'docs\milestone-4-mm3e-backend-roadmap.md') 'docs\MM3E-Backend-Roadmap.md'

$revision = (git -C $RepoRoot rev-parse --short HEAD).Trim()
@(
    'Package: Finalized Game Engine',
    'Application: HDGE Studio',
    'Engine slice: SphereWorld Slice 0',
    'Version: 0.1.0',
    "Source revision: $revision",
    'Canonical-world schema: sphere-world/v1',
    'Windows target: x86_64-pc-windows-gnu'
) | Set-Content -LiteralPath (Join-Path $StagingRoot 'VERSION') -Encoding utf8

$checksumLines = Get-ChildItem -LiteralPath $StagingRoot -Recurse -File |
    Sort-Object FullName |
    ForEach-Object {
        $relative = $_.FullName.Substring($StagingRoot.Length).TrimStart('\').Replace('\', '/')
        $hash = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $relative"
    }
$checksumLines | Set-Content -LiteralPath (Join-Path $StagingRoot 'SHA256SUMS.txt') -Encoding ascii

$wixCommand = Get-Command wix -ErrorAction SilentlyContinue
if ($wixCommand -and -not $UseSdkProject) {
    Write-Host 'Building MSI with the WiX CLI...'
    & $wixCommand.Source build -arch x64 -o $MsiPath $WixSource
    if ($LASTEXITCODE -ne 0) {
        throw "WiX CLI build failed with exit code $LASTEXITCODE."
    }
}
else {
    $dotnetCommand = Get-Command dotnet -ErrorAction SilentlyContinue
    if (-not $dotnetCommand) {
        throw 'WiX CLI was not found and dotnet is unavailable. Install WiX Toolset or the .NET SDK, then retry.'
    }
    Write-Host 'Building MSI with the WiX SDK project...'
    & $dotnetCommand.Source build $WixProject -c Release -p:InstallerPlatform=x64
    if ($LASTEXITCODE -ne 0) {
        throw "WiX SDK build failed with exit code $LASTEXITCODE."
    }
    $sdkMsi = Get-ChildItem -Path (Join-Path $PackagingRoot 'wix\bin') -Filter '*.msi' -Recurse |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if (-not $sdkMsi) {
        throw 'WiX SDK build completed without producing an MSI file.'
    }
    Copy-Item -LiteralPath $sdkMsi.FullName -Destination $MsiPath -Force
}

$artifact = Get-Item -LiteralPath $MsiPath
$hash = Get-FileHash -Algorithm SHA256 -LiteralPath $MsiPath
Write-Host "MSI created: $($artifact.FullName)"
Write-Host "Size: $([Math]::Round($artifact.Length / 1MB, 2)) MiB"
Write-Host "SHA-256: $($hash.Hash)"
