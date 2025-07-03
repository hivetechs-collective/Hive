;HiveTechs Consensus NSIS Installer Script
;Creates a professional Windows installer with proper uninstall support

!define APPNAME "HiveTechs Consensus"
!define COMPANYNAME "HiveTechs Collective"
!define DESCRIPTION "AI-powered codebase intelligence platform"
!define VERSIONMAJOR 0
!define VERSIONMINOR 1
!define VERSIONBUILD 0
!define HELPURL "https://github.com/hivetechs/hive"
!define UPDATEURL "https://github.com/hivetechs/hive/releases"
!define ABOUTURL "https://hivetechs.com"

; Estimate install size (in KB)
!define INSTALLSIZE 50000

; Include modern UI
!include "MUI2.nsh"
!include "WinVer.nsh"
!include "x64.nsh"
!include "LogicLib.nsh"

; Installer properties
Name "${APPNAME}"
OutFile "HiveTechs-Consensus-Setup.exe"
InstallDir "$PROGRAMFILES64\${COMPANYNAME}\${APPNAME}"
InstallDirRegKey HKLM "Software\${COMPANYNAME}\${APPNAME}" "InstallLocation"
RequestExecutionLevel admin

; Version information
VIProductVersion "${VERSIONMAJOR}.${VERSIONMINOR}.${VERSIONBUILD}.0"
VIAddVersionKey "ProductName" "${APPNAME}"
VIAddVersionKey "CompanyName" "${COMPANYNAME}"
VIAddVersionKey "ProductVersion" "${VERSIONMAJOR}.${VERSIONMINOR}.${VERSIONBUILD}"
VIAddVersionKey "FileDescription" "${DESCRIPTION}"
VIAddVersionKey "FileVersion" "${VERSIONMAJOR}.${VERSIONMINOR}.${VERSIONBUILD}.0"
VIAddVersionKey "LegalCopyright" "© 2024 ${COMPANYNAME}"

; Modern UI configuration
!define MUI_ABORTWARNING
!define MUI_UNABORTWARNING
!define MUI_WELCOMEPAGE_TITLE "Welcome to ${APPNAME} Setup"
!define MUI_WELCOMEPAGE_TEXT "This wizard will guide you through the installation of ${APPNAME}.$\r$\n$\r$\n${DESCRIPTION}$\r$\n$\r$\nClick Next to continue."

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "license.txt"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\hive.exe"
!define MUI_FINISHPAGE_RUN_PARAMETERS "--version"
!define MUI_FINISHPAGE_RUN_TEXT "Run ${APPNAME} to verify installation"
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Languages
!insertmacro MUI_LANGUAGE "English"

; Installer sections
Section "Core Application" SecCore
    SectionIn RO  ; Read-only section (mandatory)
    
    DetailPrint "Installing ${APPNAME}..."
    
    ; Set output path
    SetOutPath $INSTDIR
    
    ; Install main executable
    File "hive.exe"
    
    ; Create uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"
    
    ; Registry entries for uninstaller
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "DisplayName" "${APPNAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "QuietUninstallString" "$INSTDIR\uninstall.exe /S"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "DisplayIcon" "$INSTDIR\hive.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "Publisher" "${COMPANYNAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "HelpLink" "${HELPURL}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "URLUpdateInfo" "${UPDATEURL}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "URLInfoAbout" "${ABOUTURL}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "DisplayVersion" "${VERSIONMAJOR}.${VERSIONMINOR}.${VERSIONBUILD}"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "VersionMajor" ${VERSIONMAJOR}
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "VersionMinor" ${VERSIONMINOR}
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "NoRepair" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}" "EstimatedSize" ${INSTALLSIZE}
    
    ; Application registry entries
    WriteRegStr HKLM "Software\${COMPANYNAME}\${APPNAME}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "Software\${COMPANYNAME}\${APPNAME}" "Version" "${VERSIONMAJOR}.${VERSIONMINOR}.${VERSIONBUILD}"
    WriteRegStr HKLM "Software\${COMPANYNAME}\${APPNAME}" "InstallDate" "$\"$(GetDate)$\""
    
SectionEnd

Section "Add to PATH" SecPath
    DetailPrint "Adding to system PATH..."
    
    ; Get current PATH
    ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "PATH"
    
    ; Check if already in PATH
    ${StrStr} $1 $0 "$INSTDIR"
    ${If} $1 == ""
        ; Add to PATH
        WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "PATH" "$0;$INSTDIR"
        
        ; Broadcast environment change
        SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
    ${EndIf}
SectionEnd

Section "PowerShell Integration" SecPowerShell
    DetailPrint "Installing PowerShell completions..."
    
    ; Create PowerShell profile directory for all users
    CreateDirectory "$PROGRAMFILES\WindowsPowerShell\Modules\HiveTechs"
    
    ; Generate PowerShell completion
    ExecWait '"$INSTDIR\hive.exe" completion powershell' $0
    ${If} $0 == 0
        FileOpen $1 "$PROGRAMFILES\WindowsPowerShell\Modules\HiveTechs\HiveTechs.psm1" w
        ExecWait '"$INSTDIR\hive.exe" completion powershell' $0 "$PROGRAMFILES\WindowsPowerShell\Modules\HiveTechs\HiveTechs.psm1"
        FileClose $1
    ${EndIf}
    
    ; Create module manifest
    FileOpen $1 "$PROGRAMFILES\WindowsPowerShell\Modules\HiveTechs\HiveTechs.psd1" w
    FileWrite $1 "@{$\r$\n"
    FileWrite $1 "    ModuleVersion = '${VERSIONMAJOR}.${VERSIONMINOR}.${VERSIONBUILD}'$\r$\n"
    FileWrite $1 "    RootModule = 'HiveTechs.psm1'$\r$\n"
    FileWrite $1 "    Author = '${COMPANYNAME}'$\r$\n"
    FileWrite $1 "    Description = 'PowerShell completions for ${APPNAME}'$\r$\n"
    FileWrite $1 "}$\r$\n"
    FileClose $1
SectionEnd

Section "Desktop Integration" SecDesktop
    DetailPrint "Creating desktop integration..."
    
    ; Create start menu shortcut
    CreateDirectory "$SMPROGRAMS\${COMPANYNAME}"
    CreateShortcut "$SMPROGRAMS\${COMPANYNAME}\${APPNAME}.lnk" "$INSTDIR\hive.exe" "" "$INSTDIR\hive.exe" 0
    CreateShortcut "$SMPROGRAMS\${COMPANYNAME}\${APPNAME} Help.lnk" "${HELPURL}" "" "$INSTDIR\hive.exe" 0
    CreateShortcut "$SMPROGRAMS\${COMPANYNAME}\Uninstall ${APPNAME}.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0
    
    ; Create desktop shortcut (optional)
    CreateShortcut "$DESKTOP\${APPNAME}.lnk" "$INSTDIR\hive.exe" "" "$INSTDIR\hive.exe" 0
SectionEnd

Section "Auto-Update Service" SecAutoUpdate
    DetailPrint "Configuring auto-update service..."
    
    ; Create scheduled task for auto-updates
    ExecWait 'schtasks /create /tn "HiveTechs Consensus Auto-Update" /tr "$INSTDIR\hive.exe update --check" /sc daily /st 09:00 /ru SYSTEM /f' $0
    
    ; Initialize configuration
    ExecWait '"$INSTDIR\hive.exe" config init --silent' $0
SectionEnd

; Section descriptions
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SecCore} "Core ${APPNAME} application (required)"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecPath} "Add ${APPNAME} to system PATH for command-line access"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecPowerShell} "Install PowerShell tab completion support"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "Create Start Menu and Desktop shortcuts"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecAutoUpdate} "Configure automatic updates (recommended)"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; Installer functions
Function .onInit
    ; Check if 64-bit system
    ${If} ${RunningX64}
    ${Else}
        MessageBox MB_OK "This application requires a 64-bit version of Windows."
        Abort
    ${EndIf}
    
    ; Check Windows version (Windows 10 or later)
    ${IfNot} ${AtLeastWin10}
        MessageBox MB_OK "This application requires Windows 10 or later."
        Abort
    ${EndIf}
    
    ; Check if already installed
    ReadRegStr $0 HKLM "Software\${COMPANYNAME}\${APPNAME}" "InstallLocation"
    ${If} $0 != ""
        MessageBox MB_YESNO "${APPNAME} is already installed. Do you want to continue?" IDYES continue
        Abort
        continue:
    ${EndIf}
    
    ; Initialize sections
    IntOp $0 ${SF_SELECTED} | ${SF_RO}
    SectionSetFlags ${SecCore} $0
    SectionSetFlags ${SecPath} ${SF_SELECTED}
    SectionSetFlags ${SecPowerShell} ${SF_SELECTED}
    SectionSetFlags ${SecDesktop} ${SF_SELECTED}
    SectionSetFlags ${SecAutoUpdate} ${SF_SELECTED}
FunctionEnd

Function .onInstSuccess
    ; Show completion message
    MessageBox MB_OK "${APPNAME} has been installed successfully!$\r$\n$\r$\nGet started by opening a command prompt and typing 'hive --help'"
FunctionEnd

; Uninstaller
Section "Uninstall"
    ; Remove files
    Delete "$INSTDIR\hive.exe"
    Delete "$INSTDIR\uninstall.exe"
    
    ; Remove shortcuts
    Delete "$SMPROGRAMS\${COMPANYNAME}\${APPNAME}.lnk"
    Delete "$SMPROGRAMS\${COMPANYNAME}\${APPNAME} Help.lnk"
    Delete "$SMPROGRAMS\${COMPANYNAME}\Uninstall ${APPNAME}.lnk"
    RMDir "$SMPROGRAMS\${COMPANYNAME}"
    Delete "$DESKTOP\${APPNAME}.lnk"
    
    ; Remove PowerShell integration
    RMDir /r "$PROGRAMFILES\WindowsPowerShell\Modules\HiveTechs"
    
    ; Remove from PATH
    ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "PATH"
    ${StrRep} $1 $0 ";$INSTDIR" ""
    ${StrRep} $2 $1 "$INSTDIR;" ""
    ${StrRep} $3 $2 "$INSTDIR" ""
    WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "PATH" "$3"
    
    ; Remove scheduled task
    ExecWait 'schtasks /delete /tn "HiveTechs Consensus Auto-Update" /f' $0
    
    ; Remove registry entries
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${COMPANYNAME} ${APPNAME}"
    DeleteRegKey HKLM "Software\${COMPANYNAME}\${APPNAME}"
    DeleteRegKey /ifempty HKLM "Software\${COMPANYNAME}"
    
    ; Remove installation directory
    RMDir "$INSTDIR"
    
    ; Broadcast environment change
    SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
    
    MessageBox MB_OK "${APPNAME} has been uninstalled.$\r$\n$\r$\nUser configuration files in %USERPROFILE%\.hive have been preserved.$\r$\nDelete them manually if desired."
SectionEnd

Function un.onInit
    MessageBox MB_YESNO "Are you sure you want to uninstall ${APPNAME}?" IDYES +2
    Abort
FunctionEnd