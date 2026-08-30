; Cymbra Music — Windows per-user installer (change: add-desktop-auto-update,
; design D4).
;
; ============================================================================
;  THE AppId GUID BELOW IS PERMANENT. DO NOT CHANGE IT. EVER.
;
;  Inno identifies an installation by AppId. Changing it makes every subsequent
;  installer install Cymbra *alongside* the existing one instead of over it:
;  two copies, two uninstall entries, and an auto-update that appears to work
;  while the user keeps launching the old build. There is no recovery short of
;  asking every user to uninstall by hand.
; ============================================================================
;
; Per-user by design. `PrivilegesRequired=lowest` + an install root under
; %LocalAppData% is the entire reason an update can be applied SILENTLY WITH NO
; UAC PROMPT — a Program Files install would elevate on every update, which in
; practice means nobody updates. It also means no code-signing certificate is
; required to install (SmartScreen still warns on first download; accepted).
;
; Built in CI by the `windows` job:
;   iscc /DAppVersion=1.25.0 /DBuildDir=... /DOutputDir=... /DOutputBaseFilename=...
;        apps\music\windows\installer\cymbra.iss
;
; The updater spawns this installer detached with
;   /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /CLOSEAPPLICATIONS /RESTARTAPPLICATIONS
; and then exits — a running .exe cannot be overwritten on Windows, so the
; installer, not the app, performs the swap.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef BuildDir
  #define BuildDir "..\\..\\build\\windows\\x64\\runner\\Release"
#endif
#ifndef OutputDir
  #define OutputDir "..\\..\\..\\..\\dist"
#endif
#ifndef OutputBaseFilename
  #define OutputBaseFilename "cymbra-music-setup"
#endif

[Setup]
; --- PERMANENT. See the header. ---
AppId={{1F700CD6-3EEC-4C0E-8B5D-AE64D8A1CEFA}
AppName=Cymbra Music
AppVersion={#AppVersion}
AppVerName=Cymbra Music {#AppVersion}
AppPublisher=NEETROF
AppPublisherURL=https://cymbra.app
AppSupportURL=https://cymbra.app
VersionInfoVersion={#AppVersion}

; Per-user install: no elevation, at install time or at update time.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
DefaultDirName={localappdata}\Programs\Cymbra
DefaultGroupName=Cymbra
DisableProgramGroupPage=yes
UsePreviousAppDir=yes

; Ask the Restart Manager to close a running Cymbra rather than failing on a
; locked file, and to bring it back afterwards. RestartApplications is the
; fallback path; the primary relaunch is the [Run] entry below, which also fires
; in silent mode.
CloseApplications=yes
RestartApplications=yes

; The uninstaller and its Add/Remove Programs entry.
UninstallDisplayName=Cymbra Music
UninstallDisplayIcon={app}\music.exe
SetupIconFile=..\runner\resources\app_icon.ico

OutputDir={#OutputDir}
OutputBaseFilename={#OutputBaseFilename}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "it"; MessagesFile: "compiler:Languages\Italian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#BuildDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
; Install-method marker (design D4). The updater reads this at runtime to decide
; whether it may self-install: present ⇒ installer-managed, so a silent
; reinstall is the update path. The portable zip has no marker and therefore
; takes the notify-only path — it must never try to overwrite its own files.
Source: "install_method.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Cymbra Music"; Filename: "{app}\music.exe"
Name: "{autodesktop}\Cymbra Music"; Filename: "{app}\music.exe"; Tasks: desktopicon

[Run]
; Relaunch after install. `nowait` so the installer can exit; deliberately WITHOUT
; `skipifsilent`, because a silent run IS the update path — with it, an auto-update
; would leave the user staring at a closed app.
Filename: "{app}\music.exe"; Description: "{cm:LaunchProgram,Cymbra Music}"; Flags: nowait postinstall
