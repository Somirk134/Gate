param(
    [string]$SourceSvg = "assets/logo/logo.svg",
    [string]$OutputDir = "client/src-tauri/icons"
)

$ErrorActionPreference = "Stop"

$sizes = @(
    @{Name = "32x32.png";    Size = 32 },
    @{Name = "128x128.png";  Size = 128 },
    @{Name = "128x128@2x.png"; Size = 256 }
)

# Ensure output directory exists
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# Resolve paths
$source = Resolve-Path $SourceSvg
$svgExt = [System.IO.Path]::GetExtension($source).ToLower()

Write-Host "Generating icons from: $source" -ForegroundColor Cyan

# Try Inkscape first (cross-platform)
$inkscape = Get-Command "inkscape" -ErrorAction SilentlyContinue
if ($inkscape) {
    Write-Host "Using Inkscape..." -ForegroundColor Green
    foreach ($item in $sizes) {
        $output = Join-Path $OutputDir $item.Name
        Write-Host "  -> $($item.Size)x$($item.Size)  $output"
        & $inkscape $source --export-filename=$output -w $item.Size -h $item.Size
        if ($LASTEXITCODE -ne 0) { throw "Inkscape failed for $($item.Name)" }
    }

    # Generate ICO (Windows)
    if ($IsWindows -or $env:OS -match "Windows") {
        $icoPath = Join-Path $OutputDir "icon.ico"
        Write-Host "  -> icon.ico"
        # Convert 256x256 PNG to ICO using PowerShell+WinForms
        Add-Type -AssemblyName System.Drawing
        $img = [System.Drawing.Image]::FromFile((Join-Path $OutputDir "128x128@2x.png"))
        $icoStream = New-Object System.IO.MemoryStream
        $icon = [System.Drawing.Icon]::FromHandle($img.GetHicon())
        $icon.Save($icoStream)
        [System.IO.File]::WriteAllBytes($icoPath, $icoStream.ToArray())
        $img.Dispose(); $icon.Dispose(); $icoStream.Dispose()
    }

    # Generate ICNS (macOS)
    if ($IsMacOS) {
        $icnsPath = Join-Path $OutputDir "icon.icns"
        Write-Host "  -> icon.icns"
        $pngPath = Join-Path $OutputDir "128x128@2x.png"
        & "iconutil" -c icns -o $icnsPath (New-Item -ItemType Directory -Path "$OutputDir/icon.iconset" -Force)
        Copy-Item $pngPath "$OutputDir/icon.iconset/icon_256x256.png"
        & "iconutil" -c icns -o $icnsPath "$OutputDir/icon.iconset"
        Remove-Item -Recurse -Force "$OutputDir/icon.iconset"
    }

    Write-Host "Icons generated successfully!" -ForegroundColor Green
    return
}

# Try rsvg-convert (librsvg)
$rsvg = Get-Command "rsvg-convert" -ErrorAction SilentlyContinue
if ($rsvg) {
    Write-Host "Using rsvg-convert..." -ForegroundColor Green
    foreach ($item in $sizes) {
        $output = Join-Path $OutputDir $item.Name
        Write-Host "  -> $($item.Size)x$($item.Size)  $output"
        & $rsvg-convert -w $item.Size -h $item.Size -o $output $source
        if ($LASTEXITCODE -ne 0) { throw "rsvg-convert failed for $($item.Name)" }
    }
    Write-Host "Icons generated successfully!" -ForegroundColor Green
    return
}

# Try ImageMagick
$magick = Get-Command "magick" -ErrorAction SilentlyContinue
if ($magick) {
    Write-Host "Using ImageMagick..." -ForegroundColor Green
    foreach ($item in $sizes) {
        $output = Join-Path $OutputDir $item.Name
        Write-Host "  -> $($item.Size)x$($item.Size)  $output"
        & $magick convert -background none -size "$($item.Size)x$($item.Size)" $source $output
        if ($LASTEXITCODE -ne 0) { throw "ImageMagick failed for $($item.Name)" }
    }
    Write-Host "Icons generated successfully!" -ForegroundColor Green
    return
}

Write-Host @"

ERROR: No icon generation tool found.
Install one of:
  - Inkscape  (https://inkscape.org/)
  - librsvg   (https://wiki.gnome.org/Projects/LibRsvg)
  - ImageMagick (https://imagemagick.org/)

Or manually convert logo.svg to the required PNG sizes:
"@ -ForegroundColor Yellow

foreach ($item in $sizes) {
    Write-Host "  $($item.Size)x$($item.Size) -> $OutputDir/$($item.Name)" -ForegroundColor Yellow
}
Write-Host "  Multi        -> $OutputDir/icon.ico" -ForegroundColor Yellow
Write-Host "  Multi        -> $OutputDir/icon.icns" -ForegroundColor Yellow

exit 1
