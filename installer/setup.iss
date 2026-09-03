#define MyAppName "dotXPANDER"
#define MyAppVersion "0.2.0"
#define MyAppPublisher "aiVOLUTION"
#define MyAppURL "https://github.com/ThMoJe/dotxpander"

; Expected compiler variables passed via command line, e.g.:\
; iscc /DMyAppArchitecture="x64" /DMyAppExePath="..\target\x86_64-pc-windows-msvc\release\dotxpander.exe" setup.iss

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
AppId={{ACFE89A0-7A1E-4E92-95FD-A4E5D477324B}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL=https://aivolution.dk
AppSupportURL=mailto:info@aivolution.dk
AppUpdatesURL={#MyAppURL}
; Country of Origin: Denmark/EU
; Install to localappdata (Per-User)
DefaultDirName={localappdata}\Programs\aiVOLUTION\dotXPANDER
; Avoid UAC prompt by requesting lowest execution level
PrivilegesRequired=lowest
OutputDir=..\dist_installers
OutputBaseFilename=dotXPANDER-Setup-{#MyAppArchitecture}
SetupIconFile=..\ui\icon.ico
WizardSmallImageFile=wizard_small.bmp
WizardImageFile=wizard_large.bmp
UninstallDisplayName=.XPANDER
UninstallDisplayIcon={app}\icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed={#MyAppArchitecture}
ArchitecturesInstallIn64BitMode={#MyAppArchitecture}
; Minimum supported OS: Windows 10 1809 (build 17763)
MinVersion=10.0.17763
; Detect a running instance and offer to close it before upgrading
; Must match the mutex name used in main.rs: Global\dotXPANDERSingleton
AppMutex=Global\dotXPANDERSingleton
; Offer to close any running dotXPANDER process during install/upgrade
CloseApplications=yes
RestartApplications=yes
; Display the MIT license agreement in the installer wizard
LicenseFile=..\LICENSE

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "autostart"; Description: "Start dotXPANDER automatically when Windows starts"; GroupDescription: "Autostart:"
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#MyAppExePath}"; DestDir: "{app}"; DestName: "dotxpander.exe"; Flags: ignoreversion
Source: "..\ui\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\dotxpander.exe"; IconFilename: "{app}\icon.ico"; AppUserModelID: "aiVOLUTION.dotXPANDER"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\dotxpander.exe"; Tasks: desktopicon; IconFilename: "{app}\icon.ico"; AppUserModelID: "aiVOLUTION.dotXPANDER"

[Registry]
; Autostart registry entry (only created when the "autostart" task is selected)
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "dotXPANDER"; ValueData: """{app}\dotxpander.exe"""; Flags: uninsdeletevalue; Tasks: autostart

; Config directory registry entry — written by Pascal script after the user
; picks a folder on the custom wizard page, so the value comes from GetConfigDir().
; Flags: uninsdeletekey  → the ENTIRE Software\aiVOLUTION\dotXPANDER key is removed on uninstall.
Root: HKCU; Subkey: "Software\aiVOLUTION\dotXPANDER"; ValueType: string; ValueName: "ConfigPath"; ValueData: "{code:GetConfigDir}"; Flags: uninsdeletekey

[Run]
Filename: "{app}\dotxpander.exe"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent; WorkingDir: "{app}"

[Code]
// ---------------------------------------------------------------------------
// Custom wizard page — Config directory selection
// ---------------------------------------------------------------------------
//
// This page is shown between the "Destination" and "Ready to Install" pages.
// The user can type a path or browse to a folder. The default is the standard
// %APPDATA%\aiVOLUTION\dotXPANDER directory (installed mode).
//
// If the user already has a config.toml at the chosen location, it is NOT
// overwritten (preserves existing snippets / settings).

var
  ConfigDirPage: TInputDirWizardPage;

// ---------------------------------------------------------------------------
// Downgrade guard — warn user if they are installing an older version over a
// newer one. Reads the currently installed version from the Uninstall registry
// key that Inno Setup writes on every install.
// ---------------------------------------------------------------------------
function IsDowngrade(): Boolean;
var
  InstalledVer: String;
begin
  Result := False;
  // Inno Setup writes the current version to this subkey during install.
  if RegQueryStringValue(HKCU,
    'Software\Microsoft\Windows\CurrentVersion\Uninstall\{ACFE89A0-7A1E-4E92-95FD-A4E5D477324B}_is1',
    'DisplayVersion', InstalledVer) then
  begin
    // A simple string comparison works for SemVer as long as segment counts
    // and zero-padding are consistent (e.g. "0.2.0" vs "0.1.0").
    if CompareStr(InstalledVer, '{#MyAppVersion}') > 0 then
      Result := True;
  end;
end;

function InitializeSetup(): Boolean;
var
  Msg: String;
begin
  Result := True;
  if IsDowngrade() then
  begin
    Msg := 'A newer version of ' + '{#MyAppName}' + ' is already installed.' + #13#10 +
           #13#10 +
           'Installing this older version may cause issues.' + #13#10 +
           'Are you sure you want to continue?';
    Result := MsgBox(Msg, mbConfirmation, MB_YESNO) = IDYES;
  end;
end;

// Called by the [Registry] section to retrieve the chosen directory.
function GetConfigDir(Param: String): String;
begin
  Result := ConfigDirPage.Values[0];
end;

// ---------------------------------------------------------------------------
// Wizard page creation
// ---------------------------------------------------------------------------
procedure InitializeWizard();
var
  DefaultConfigDir: String;
begin
  DefaultConfigDir := ExpandConstant('{userappdata}') + '\aiVOLUTION\dotXPANDER';

  ConfigDirPage := CreateInputDirPage(
    wpSelectDir,   // Insert after the "Destination" page
    'Choose Settings Location',
    'Where should dotXPANDER store your settings and snippets?',
    'Pick a folder for your configuration file.' +
    Chr(13) + Chr(10) + Chr(13) + Chr(10) +
    'Hint: choose a cloud-synced folder (e.g., OneDrive, Dropbox) to share ' +
    'your snippets and settings across multiple computers automatically.' +
    Chr(13) + Chr(10) + Chr(13) + Chr(10) +
    'Leave as default if you only use one computer.',
    False,   // Do not append the app name to the path
    ''       // No subfolder hint
  );
  ConfigDirPage.Add('');
  ConfigDirPage.Values[0] := DefaultConfigDir;
end;

// ---------------------------------------------------------------------------
// Post-install: write default config.toml only if one does not already exist
// ---------------------------------------------------------------------------
procedure CurStepChanged(CurStep: TSetupStep);
var
  ConfigDir:    String;
  ConfigFile:   String;
  DefaultToml:  String;
begin
  if CurStep = ssPostInstall then
  begin
    ConfigDir  := ConfigDirPage.Values[0];
    ConfigFile := ConfigDir + '\config.toml';

    // Ensure the chosen config directory exists.
    if not DirExists(ConfigDir) then
      ForceDirectories(ConfigDir);

    // Write a minimal default config.toml only when none exists at the target.
    // This preserves existing user snippets / settings on upgrades or re-installs.
    if not FileExists(ConfigFile) then
    begin
      DefaultToml :=
        'language = "en"'                                + Chr(13)+Chr(10) +
        ''                                               + Chr(13)+Chr(10) +
        '[hotkey]'                                       + Chr(13)+Chr(10) +
        'modifiers = 6'                                  + Chr(13)+Chr(10) +
        'virtual_key = 84'                               + Chr(13)+Chr(10) +
        ''                                               + Chr(13)+Chr(10) +
        'buffer_size = 10'                               + Chr(13)+Chr(10) +
        'clipboard_restore_delay_ms = 150'               + Chr(13)+Chr(10) +
        'snippet_hotkey_enabled = true'                  + Chr(13)+Chr(10) +
        'quick_switch_enabled = true'                    + Chr(13)+Chr(10) +
        'case_changer_enabled = true'                    + Chr(13)+Chr(10) +
        ''                                               + Chr(13)+Chr(10) +
        '[case_changer_hotkey]'                          + Chr(13)+Chr(10) +
        'modifiers = 2'                                  + Chr(13)+Chr(10) +
        'virtual_key = 20'                               + Chr(13)+Chr(10) +
        ''                                               + Chr(13)+Chr(10) +
        '[[snippets]]'                                   + Chr(13)+Chr(10) +
        'trigger = ".sig"'                               + Chr(13)+Chr(10) +
        'replacement = "Regards,\nYour Name"'            + Chr(13)+Chr(10) +
        'mode = "immediate"'                             + Chr(13)+Chr(10) +
        ''                                               + Chr(13)+Chr(10) +
        '[[snippets]]'                                   + Chr(13)+Chr(10) +
        'trigger = ".em"'                                + Chr(13)+Chr(10) +
        'replacement = "your.name@example.com"'          + Chr(13)+Chr(10) +
        'mode = "immediate"'                             + Chr(13)+Chr(10);

      SaveStringToFile(ConfigFile, DefaultToml, False);
    end;
  end;
end;

// ---------------------------------------------------------------------------
// Uninstall: always remove registry key; only delete config files if they
// are under the default %APPDATA%\aiVOLUTION\dotXPANDER path.
// Custom / cloud-synced locations are left intact.
// ---------------------------------------------------------------------------
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  DefaultConfigDir: String;
  RegConfigDir:     String;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    DefaultConfigDir := ExpandConstant('{userappdata}') + '\aiVOLUTION\dotXPANDER';

    // Read the registry key to find out where config actually lives.
    // (The [Registry] Flags: uninsdeletekey already removes the key, but we
    // read it first, before the standard uninstall logic runs, to get the path.)
    // At this point the key may already be gone; if so we fall back to default.
    if not RegQueryStringValue(HKCU, 'Software\aiVOLUTION\dotXPANDER', 'ConfigPath', RegConfigDir) then
      RegConfigDir := DefaultConfigDir;

    // Only wipe files when they are in the default location.
    // Cloud-synced / custom paths are left untouched.
    if SameText(RegConfigDir, DefaultConfigDir) then
    begin
      if DirExists(DefaultConfigDir) then
        DelTree(DefaultConfigDir, True, True, True);
    end;
  end;
end;
