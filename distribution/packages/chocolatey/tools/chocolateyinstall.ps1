# HiveTechs Consensus Chocolatey Installation Script

$ErrorActionPreference = 'Stop'

$packageName = 'hive-ai'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url64 = 'https://github.com/hivetechs/hive/releases/download/v2.0.0/hive-windows-x86_64.exe'
$checksum64 = 'PLACEHOLDER_SHA256'

$packageArgs = @{
  packageName   = $packageName
  unzipLocation = $toolsDir
  fileType      = 'EXE'
  url64bit      = $url64
  
  softwareName  = 'HiveTechs Consensus*'
  
  checksum64    = $checksum64
  checksumType64= 'sha256'
  
  silentArgs    = "/S"
  validExitCodes= @(0)
}

Install-ChocolateyPackage @packageArgs

# Install to PATH
$exePath = Join-Path $toolsDir "hive.exe"
Install-ChocolateyPath $toolsDir 'Machine'

# Create shim for hive command
Install-ChocolateyShortcut -shortcutFilePath "$env:ChocolateyInstall\bin\hive.exe" -targetPath $exePath

Write-Host ""
Write-Host "🐝 HiveTechs Consensus installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Get started:" -ForegroundColor Yellow
Write-Host "  hive --help         # Show available commands"
Write-Host "  hive setup          # Configure API keys"
Write-Host "  hive ask 'Hello'    # Test consensus engine"
Write-Host ""
Write-Host "Documentation: https://hive.ai/docs" -ForegroundColor Cyan
Write-Host "Support: https://github.com/hivetechs/hive/issues" -ForegroundColor Cyan
Write-Host ""