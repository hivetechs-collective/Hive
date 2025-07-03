#
# Windows Build Script for HiveTechs Consensus
# Builds optimized binaries for Windows platforms
#

param(
    [string]$Version = $env:HIVE_VERSION
)

# Set error action preference
$ErrorActionPreference = "Stop"

# Get script directory and project root
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $ScriptDir)
$BuildDir = Join-Path $ProjectRoot "target\distribution\windows"

# Get version from Cargo.toml if not provided
if (-not $Version) {
    $CargoToml = Get-Content (Join-Path $ProjectRoot "Cargo.toml") -Raw
    if ($CargoToml -match 'version\s*=\s*"([^"]+)"') {
        $Version = $Matches[1]
    } else {
        $Version = "0.1.0"
    }
}

# Colors for output
$Colors = @{
    Red = [ConsoleColor]::Red
    Green = [ConsoleColor]::Green
    Yellow = [ConsoleColor]::Yellow
    Blue = [ConsoleColor]::Blue
    White = [ConsoleColor]::White
}

function Write-Status {
    param([string]$Message)
    Write-Host "[BUILD] $Message" -ForegroundColor $Colors.Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $Colors.Green
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $Colors.Red
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $Colors.Yellow
}

# Create build directory
if (-not (Test-Path $BuildDir)) {
    New-Item -ItemType Directory -Path $BuildDir -Force | Out-Null
}

Set-Location $ProjectRoot

Write-Status "Building HiveTechs Consensus v$Version for Windows"

# Check for required tools
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
}

# Add Windows targets
Write-Status "Adding Rust targets..."
rustup target add x86_64-pc-windows-msvc 2>$null
rustup target add i686-pc-windows-msvc 2>$null

# Build for 64-bit Windows
Write-Status "Building for 64-bit Windows..."
cargo build `
    --profile production `
    --target x86_64-pc-windows-msvc `
    --features production `
    --bin hive

# Build for 32-bit Windows
Write-Status "Building for 32-bit Windows..."
try {
    cargo build `
        --profile production `
        --target i686-pc-windows-msvc `
        --features production `
        --bin hive
    $Build32Bit = $true
} catch {
    Write-Warning "32-bit build failed, continuing with 64-bit only"
    $Build32Bit = $false
}

# Copy binaries to distribution directory
$Source64 = Join-Path $ProjectRoot "target\x86_64-pc-windows-msvc\production\hive.exe"
$Dest64 = Join-Path $BuildDir "hive-x64.exe"
Copy-Item $Source64 $Dest64

if ($Build32Bit) {
    $Source32 = Join-Path $ProjectRoot "target\i686-pc-windows-msvc\production\hive.exe"
    $Dest32 = Join-Path $BuildDir "hive-x86.exe"
    Copy-Item $Source32 $Dest32
}

# Create main executable (64-bit by default)
$MainExe = Join-Path $BuildDir "hive.exe"
Copy-Item $Dest64 $MainExe

# Check binary size
$BinarySize = (Get-Item $MainExe).Length / 1MB
Write-Status "Binary size: $($BinarySize.ToString('F1')) MB"

# Create shell completions
Write-Status "Generating shell completions..."
$CompletionsDir = Join-Path $BuildDir "completions"
if (-not (Test-Path $CompletionsDir)) {
    New-Item -ItemType Directory -Path $CompletionsDir -Force | Out-Null
}

try {
    & $MainExe completion powershell > (Join-Path $CompletionsDir "hive.ps1")
    & $MainExe completion bash > (Join-Path $CompletionsDir "hive.bash")
} catch {
    Write-Warning "Failed to generate completions"
}

# Create Windows installer configuration
Write-Status "Creating installer configuration..."
$InstallerDir = Join-Path $BuildDir "installer"
if (-not (Test-Path $InstallerDir)) {
    New-Item -ItemType Directory -Path $InstallerDir -Force | Out-Null
}

# Create WiX installer configuration
$WixConfig = @"
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" 
           Name="HiveTechs Consensus" 
           Language="1033" 
           Version="$Version" 
           Manufacturer="HiveTechs Collective" 
           UpgradeCode="12345678-1234-1234-1234-123456789012">
    
    <Package InstallerVersion="200" 
             Compressed="yes" 
             InstallScope="perMachine" 
             Description="AI-powered codebase intelligence platform" />
    
    <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
    <MediaTemplate EmbedCab="yes" />
    
    <Feature Id="ProductFeature" Title="HiveTechs Consensus" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>
    
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="HiveTechs Consensus" />
      </Directory>
      <Directory Id="ProgramMenuFolder">
        <Directory Id="ApplicationProgramsFolder" Name="HiveTechs Consensus" />
      </Directory>
    </Directory>
    
    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <Component Id="HiveExecutable" Guid="*">
        <File Id="HiveExe" 
              Source="$MainExe" 
              KeyPath="yes" />
        <Environment Id="PATH" 
                     Name="PATH" 
                     Value="[INSTALLFOLDER]" 
                     Permanent="no" 
                     Part="last" 
                     Action="set" 
                     System="yes" />
      </Component>
    </ComponentGroup>
    
    <Icon Id="hive.exe" SourceFile="$MainExe" />
    <Property Id="ARPPRODUCTICON" Value="hive.exe" />
    
  </Product>
</Wix>
"@

$WixFile = Join-Path $InstallerDir "hive.wxs"
$WixConfig | Out-File -FilePath $WixFile -Encoding UTF8

# Create NSIS installer script
$NSISScript = @"
;HiveTechs Consensus Installer
;Generated by build script

!define APPNAME "HiveTechs Consensus"
!define COMPANYNAME "HiveTechs Collective"
!define DESCRIPTION "AI-powered codebase intelligence platform"
!define VERSIONMAJOR 0
!define VERSIONMINOR 1
!define VERSIONBUILD 0
!define HELPURL "https://github.com/hivetechs/hive"
!define UPDATEURL "https://github.com/hivetechs/hive/releases"
!define ABOUTURL "https://hivetechs.com"
!define INSTALLSIZE 50000

RequestExecutionLevel admin
InstallDir "`$PROGRAMFILES64\`${COMPANYNAME}\`${APPNAME}"
Name "`${APPNAME}"
outFile "hive-$Version-windows-x64-installer.exe"

page directory
page instfiles

!macro VerifyUserIsAdmin
UserInfo::GetAccountType
pop `$0
`${If} `$0 != "admin"
    messageBox mb_iconstop "Administrator rights required!"
    setErrorLevel 740
    quit
`${EndIf}
!macroend

function .onInit
    !insertmacro VerifyUserIsAdmin
functionEnd

section "install"
    setOutPath `$INSTDIR
    file "$MainExe"
    
    # Add to PATH
    nsExec::ExecToLog 'setx PATH "`$INSTDIR;%PATH%" /M'
    
    # Create uninstaller
    writeUninstaller "`$INSTDIR\uninstall.exe"
    
    # Registry entries
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "DisplayName" "`${APPNAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "UninstallString" "`$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "InstallLocation" "`$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "DisplayIcon" "`$INSTDIR\hive.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "Publisher" "`${COMPANYNAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "HelpLink" "`${HELPURL}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "URLUpdateInfo" "`${UPDATEURL}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "URLInfoAbout" "`${ABOUTURL}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "DisplayVersion" "`${VERSIONMAJOR}.`${VERSIONMINOR}.`${VERSIONBUILD}"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "VersionMajor" `${VERSIONMAJOR}
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "VersionMinor" `${VERSIONMINOR}
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "NoRepair" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}" "EstimatedSize" `${INSTALLSIZE}
sectionEnd

section "uninstall"
    delete "`$INSTDIR\hive.exe"
    delete "`$INSTDIR\uninstall.exe"
    rmDir "`$INSTDIR"
    
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`${COMPANYNAME} `${APPNAME}"
sectionEnd
"@

$NSISFile = Join-Path $InstallerDir "hive.nsi"
$NSISScript | Out-File -FilePath $NSISFile -Encoding UTF8

# Create PowerShell module for system integration
Write-Status "Creating PowerShell module..."
$ModuleDir = Join-Path $BuildDir "powershell"
if (-not (Test-Path $ModuleDir)) {
    New-Item -ItemType Directory -Path $ModuleDir -Force | Out-Null
}

$ModuleScript = @"
# HiveTechs Consensus PowerShell Module
# Auto-generated by build script

function Install-Hive {
    [CmdletBinding()]
    param(
        [string]`$InstallPath = "`$env:LOCALAPPDATA\HiveTechs\Consensus"
    )
    
    Write-Host "Installing HiveTechs Consensus..." -ForegroundColor Green
    
    # Create install directory
    if (-not (Test-Path `$InstallPath)) {
        New-Item -ItemType Directory -Path `$InstallPath -Force | Out-Null
    }
    
    # Copy binary (assuming current directory has the binary)
    `$BinaryPath = Join-Path `$PSScriptRoot "hive.exe"
    if (Test-Path `$BinaryPath) {
        Copy-Item `$BinaryPath (Join-Path `$InstallPath "hive.exe")
        Write-Host "Binary installed to `$InstallPath" -ForegroundColor Green
        
        # Add to PATH
        `$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if (`$UserPath -notlike "*`$InstallPath*") {
            [Environment]::SetEnvironmentVariable("PATH", "`$UserPath;`$InstallPath", "User")
            Write-Host "Added to PATH" -ForegroundColor Green
        }
        
        # Install completions
        `$CompletionsPath = Join-Path `$InstallPath "hive.ps1"
        & (Join-Path `$InstallPath "hive.exe") completion powershell > `$CompletionsPath
        
        Write-Host "Installation complete! Please restart your shell." -ForegroundColor Green
    } else {
        Write-Error "hive.exe not found in module directory"
    }
}

function Uninstall-Hive {
    [CmdletBinding()]
    param(
        [string]`$InstallPath = "`$env:LOCALAPPDATA\HiveTechs\Consensus"
    )
    
    Write-Host "Uninstalling HiveTechs Consensus..." -ForegroundColor Yellow
    
    if (Test-Path `$InstallPath) {
        Remove-Item `$InstallPath -Recurse -Force
        Write-Host "Files removed" -ForegroundColor Green
        
        # Remove from PATH
        `$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        `$NewPath = `$UserPath -replace [regex]::Escape(";`$InstallPath"), ""
        [Environment]::SetEnvironmentVariable("PATH", `$NewPath, "User")
        Write-Host "Removed from PATH" -ForegroundColor Green
        
        Write-Host "Uninstallation complete!" -ForegroundColor Green
    } else {
        Write-Warning "Installation directory not found"
    }
}

Export-ModuleMember -Function Install-Hive, Uninstall-Hive
"@

$ModuleFile = Join-Path $ModuleDir "HiveTechs.psm1"
$ModuleScript | Out-File -FilePath $ModuleFile -Encoding UTF8

# Create module manifest
$ManifestData = @{
    Path = Join-Path $ModuleDir "HiveTechs.psd1"
    RootModule = "HiveTechs.psm1"
    ModuleVersion = $Version
    Author = "HiveTechs Collective"
    CompanyName = "HiveTechs Collective"
    Copyright = "(c) 2024 HiveTechs Collective. All rights reserved."
    Description = "PowerShell module for HiveTechs Consensus installation and management"
    FunctionsToExport = @('Install-Hive', 'Uninstall-Hive')
    CmdletsToExport = @()
    VariablesToExport = @()
    AliasesToExport = @()
}

New-ModuleManifest @ManifestData

# Create ZIP archives
Write-Status "Creating distribution archives..."
$ArchiveDir = Join-Path $BuildDir "archives"
if (-not (Test-Path $ArchiveDir)) {
    New-Item -ItemType Directory -Path $ArchiveDir -Force | Out-Null
}

# 64-bit archive
$Archive64 = Join-Path $ArchiveDir "hive-$Version-windows-x64.zip"
Compress-Archive -Path @($Dest64, $CompletionsDir) -DestinationPath $Archive64 -Force

# 32-bit archive (if available)
if ($Build32Bit) {
    $Archive32 = Join-Path $ArchiveDir "hive-$Version-windows-x86.zip"
    Compress-Archive -Path @($Dest32, $CompletionsDir) -DestinationPath $Archive32 -Force
}

# PowerShell module archive
$ModuleArchive = Join-Path $ArchiveDir "hive-$Version-powershell-module.zip"
Compress-Archive -Path @($ModuleDir, $MainExe) -DestinationPath $ModuleArchive -Force

# Calculate checksums
Write-Status "Calculating checksums..."
$ChecksumFile = Join-Path $BuildDir "checksums.sha256"
Get-ChildItem $ArchiveDir -Filter "*.zip" | ForEach-Object {
    $Hash = Get-FileHash $_.FullName -Algorithm SHA256
    "$($Hash.Hash.ToLower())  $($_.Name)" | Out-File -FilePath $ChecksumFile -Append -Encoding UTF8
}

Write-Success "Windows build complete!"
Write-Status "Distribution files:"
Get-ChildItem $BuildDir -Recurse | Where-Object { -not $_.PSIsContainer } | Format-Table Name, Length, LastWriteTime

# Performance test
Write-Status "Running performance test..."
try {
    Measure-Command { & $MainExe --version | Out-Null } | ForEach-Object {
        Write-Status "Startup time: $($_.TotalMilliseconds)ms"
    }
} catch {
    Write-Warning "Performance test failed"
}

Write-Success "Build artifacts ready at: $BuildDir"
"@