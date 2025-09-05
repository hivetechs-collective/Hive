#!/bin/bash
set -e

echo "🪟 Creating Windows MSI installer for Hive IDE..."

# Configuration
VERSION="${HIVE_VERSION:-2.0.2}"
PRODUCT_NAME="Hive IDE"
MANUFACTURER="HiveTechs Collective LLC"
UPGRADE_CODE="A1B2C3D4-5E6F-4A5B-8C9D-1E2F3A4B5C6D"  # Unique GUID for Hive IDE - DO NOT CHANGE

# Create directories
mkdir -p dist/windows-installer
mkdir -p dist/windows-build

echo "📋 Preparing Windows installer files..."

# First, we need the Windows binary
if [ ! -f "dist/hive-windows-x64.exe" ]; then
    echo "❌ Windows binary not found: dist/hive-windows-x64.exe"
    echo "   You'll need to cross-compile for Windows first:"
    echo "   cargo build --release --target x86_64-pc-windows-gnu --features desktop"
    echo "   Or build on a Windows machine"
    exit 1
fi

# Copy Windows binary
cp "dist/hive-windows-x64.exe" "dist/windows-build/hive.exe"

# Create WiX source file (.wxs)
cat > "dist/windows-installer/hive.wxs" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" 
           Name="$PRODUCT_NAME" 
           Language="1033" 
           Version="$VERSION" 
           Manufacturer="$MANUFACTURER" 
           UpgradeCode="$UPGRADE_CODE">
    
    <Package InstallerVersion="200" 
             Compressed="yes" 
             InstallScope="perMachine" 
             Description="$PRODUCT_NAME Installer"
             Comments="AI-powered development environment with revolutionary 4-stage consensus"
             Manufacturer="$MANUFACTURER" />

    <!-- Major upgrade rules -->
    <MajorUpgrade DowngradeErrorMessage="A newer version of [ProductName] is already installed." />
    
    <!-- Media -->
    <MediaTemplate EmbedCab="yes" />

    <!-- Features -->
    <Feature Id="ProductFeature" Title="$PRODUCT_NAME" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
      <ComponentRef Id="ApplicationShortcut" />
      <ComponentRef Id="PathEnvironment" />
    </Feature>

    <!-- Directory structure -->
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFiles64Folder">
        <Directory Id="ManufacturerFolder" Name="$MANUFACTURER">
          <Directory Id="INSTALLFOLDER" Name="$PRODUCT_NAME" />
        </Directory>
      </Directory>
      
      <Directory Id="ProgramMenuFolder">
        <Directory Id="ApplicationProgramsFolder" Name="$PRODUCT_NAME"/>
      </Directory>
      
      <Directory Id="DesktopFolder" Name="Desktop" />
    </Directory>

    <!-- Components -->
    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <Component Id="MainExecutable" Guid="*">
        <File Id="HiveExe" 
              Source="$(var.SourceDir)\\dist\\windows-build\\hive.exe" 
              KeyPath="yes" 
              Checksum="yes">
          <Shortcut Id="DesktopShortcut"
                    Directory="DesktopFolder"
                    Name="$PRODUCT_NAME"
                    Description="AI-powered development environment"
                    WorkingDirectory="INSTALLFOLDER"
                    Icon="HiveIcon" />
        </File>
      </Component>
    </ComponentGroup>

    <!-- Start Menu shortcut -->
    <Component Id="ApplicationShortcut" Directory="ApplicationProgramsFolder" Guid="*">
      <Shortcut Id="ApplicationStartMenuShortcut"
                Name="$PRODUCT_NAME"
                Description="AI-powered development environment with 4-stage consensus"
                Target="[#HiveExe]"
                WorkingDirectory="INSTALLFOLDER"
                Icon="HiveIcon" />
      <RemoveFolder Id="ApplicationProgramsFolder" On="uninstall"/>
      <RegistryValue Root="HKCU" 
                     Key="Software\\$MANUFACTURER\\$PRODUCT_NAME" 
                     Name="installed" 
                     Type="integer" 
                     Value="1" 
                     KeyPath="yes"/>
    </Component>

    <!-- PATH environment variable -->
    <Component Id="PathEnvironment" Directory="INSTALLFOLDER" Guid="*">
      <Environment Id="PATH" 
                   Name="PATH" 
                   Value="[INSTALLFOLDER]" 
                   Permanent="no" 
                   Part="last" 
                   Action="set" 
                   System="yes" />
      <RegistryValue Root="HKLM" 
                     Key="Software\\$MANUFACTURER\\$PRODUCT_NAME" 
                     Name="path_added" 
                     Type="integer" 
                     Value="1" 
                     KeyPath="yes"/>
    </Component>

    <!-- Icon -->
    <Icon Id="HiveIcon" SourceFile="$(var.SourceDir)\\assets\\icon.ico" />
    <Property Id="ARPPRODUCTICON" Value="HiveIcon" />
    
    <!-- Add/Remove Programs properties -->
    <Property Id="ARPHELPLINK" Value="https://hivetechs.io/support" />
    <Property Id="ARPURLINFOABOUT" Value="https://hivetechs.io" />
    <Property Id="ARPNOREPAIR" Value="1" />
    
    <!-- Custom actions for post-install -->
    <CustomAction Id="ShowReadme" 
                  FileKey="HiveExe" 
                  ExeCommand="--version" 
                  Execute="deferred" 
                  Return="ignore" />

    <!-- Install sequence -->
    <InstallExecuteSequence>
      <Custom Action="ShowReadme" After="InstallFinalize">NOT REMOVE</Custom>
    </InstallExecuteSequence>

  </Product>
</Wix>
EOF

# Create icon file if it doesn't exist
if [ ! -f "assets/icon.ico" ]; then
    echo "⚠️  No icon.ico found, creating placeholder..."
    mkdir -p assets
    # Create a simple placeholder icon (you should replace this with your real icon)
    echo "Creating placeholder icon - replace assets/icon.ico with your real icon"
    touch assets/icon.ico
fi

# Check if WiX is available (this would be run on Windows)
echo "📝 WiX installer script created!"
echo ""
echo "🔧 To build the MSI installer on Windows:"
echo "1. Install WiX Toolset: https://wixtoolset.org/releases/"
echo "2. Run these commands in Windows Command Prompt:"
echo "   candle.exe dist\\windows-installer\\hive.wxs -out dist\\windows-installer\\hive.wixobj"
echo "   light.exe dist\\windows-installer\\hive.wixobj -out dist\\Hive-IDE-$VERSION.msi"
echo ""
echo "📦 The MSI installer will be created as: dist/Hive-IDE-$VERSION.msi"

# Create a build batch file for Windows
cat > "dist/windows-installer/build.bat" << EOF
@echo off
echo Building Hive IDE MSI installer...

REM Check if WiX is installed
where candle >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: WiX Toolset not found in PATH
    echo Please install WiX from https://wixtoolset.org/releases/
    pause
    exit /b 1
)

REM Compile WiX source
echo Compiling WiX source...
candle.exe hive.wxs -out hive.wixobj
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Failed to compile WiX source
    pause
    exit /b 1
)

REM Link to create MSI
echo Creating MSI installer...
light.exe hive.wixobj -out ..\\Hive-IDE-$VERSION.msi
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Failed to create MSI
    pause
    exit /b 1
)

echo SUCCESS: MSI installer created as Hive-IDE-$VERSION.msi
pause
EOF

echo "✅ Windows installer configuration complete!"
echo "📁 Files created:"
echo "   - dist/windows-installer/hive.wxs (WiX source)"
echo "   - dist/windows-installer/build.bat (Windows build script)"
echo ""
echo "🎯 Next steps:"
echo "1. Cross-compile Windows binary: cargo build --target x86_64-pc-windows-gnu"
echo "2. Run build.bat on Windows machine with WiX installed"
echo "3. Test MSI installer on Windows"