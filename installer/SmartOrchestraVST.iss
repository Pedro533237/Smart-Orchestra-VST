#define MyAppName "SmartOrchestraVST"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Pedro Audio Labs"
#define MyAppURL "https://pedroaudiolabs.local"
#define DistDir "..\\dist\\windows-installer"

[Setup]
AppId={{B4D0460A-D756-4DBD-BF50-8E4CF7D4F42C}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableDirPage=no
DisableProgramGroupPage=yes
LicenseFile=
OutputDir={#DistDir}
OutputBaseFilename=SmartOrchestraVST-Setup-x64
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\\SmartOrchestraTestHost.exe
SetupIconFile=
PrivilegesRequired=admin

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\\BrazilianPortuguese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "vst3global"; Description: "Instalar plugin VST3 na pasta global (Common Files)"; Flags: checkedonce
Name: "desktopicon"; Description: "Criar atalho na Área de Trabalho para SmartOrchestraTestHost"; GroupDescription: "Atalhos:";

[Files]
; Instala o host de teste
Source: "..\\dist\\windows-package\\SmartOrchestraTestHost.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\\README.md"; DestDir: "{app}"; Flags: ignoreversion

; Plugin VST3 para diretório do aplicativo
Source: "..\\dist\\windows-package\\SmartOrchestraVST.vst3\\*"; DestDir: "{app}\\SmartOrchestraVST.vst3"; Flags: ignoreversion recursesubdirs createallsubdirs

; Plugin VST3 na pasta global recomendada (opcional)
Source: "..\\dist\\windows-package\\SmartOrchestraVST.vst3\\*"; DestDir: "{commoncf64}\\VST3\\SmartOrchestraVST.vst3"; Tasks: vst3global; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\\SmartOrchestraTestHost"; Filename: "{app}\\SmartOrchestraTestHost.exe"
Name: "{autodesktop}\\SmartOrchestraTestHost"; Filename: "{app}\\SmartOrchestraTestHost.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\\SmartOrchestraTestHost.exe"; Description: "Executar SmartOrchestraTestHost"; Flags: nowait postinstall skipifsilent unchecked

[UninstallDelete]
Type: filesandordirs; Name: "{commoncf64}\\VST3\\SmartOrchestraVST.vst3"
