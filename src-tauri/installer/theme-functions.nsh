Var TpsSkinControl
Var TpsSkinBitmap
Var TpsPageWidth
Var TpsPageHeight
Var TpsTextWidth
Var TpsPortraitWidth
Var TpsOptionsDesktopCheckbox

Function TpsOptionsCreate
  ${If} $PassiveMode = 1
  ${OrIf} $UpdateMode = 1
    Abort
  ${EndIf}
  !insertmacro MUI_HEADER_TEXT "Personaliza tu instalación" "Revisa el destino y elige tus accesos directos."
  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}
  SetCtlColors $0 F5EAFE 100C19
  ${NSD_CreateLabel} 0 0 100% 14u "TP LAUNCHER · v${VERSION}"
  Pop $0
  SetCtlColors $0 C480FF 100C19
  ${NSD_CreateLabel} 0 18u 100% 12u "DESTINO DE INSTALACIÓN"
  Pop $0
  SetCtlColors $0 BCA7CE 100C19
  ${NSD_CreateLabel} 0 32u 100% 22u "$INSTDIR"
  Pop $0
  SetCtlColors $0 F5EAFE 100C19
  ${NSD_CreateHLine} 0 60u 100% 1u ""
  Pop $0
  ${NSD_CreateCheckbox} 0 70u 100% 16u "Crear acceso directo en el Escritorio"
  Pop $TpsOptionsDesktopCheckbox
  System::Call 'uxtheme::SetWindowTheme(p $TpsOptionsDesktopCheckbox, w "", w "")'
  SetCtlColors $TpsOptionsDesktopCheckbox F5EAFE 100C19
  ${NSD_SetState} $TpsOptionsDesktopCheckbox $TpsDesktopChoice
  ${If} $NoShortcutMode = 1
    ${NSD_Uncheck} $TpsOptionsDesktopCheckbox
    EnableWindow $TpsOptionsDesktopCheckbox 0
  ${EndIf}
  StrCpy $1 $AppStartMenuFolder 1
  ${If} $1 == ">"
    StrCpy $1 "Menú Inicio: no crear acceso directo"
  ${Else}
    StrCpy $1 "Menú Inicio: $AppStartMenuFolder"
  ${EndIf}
  ${NSD_CreateLabel} 0 92u 100% 16u "$1"
  Pop $0
  SetCtlColors $0 BCA7CE 100C19
  ${NSD_CreateLabel} 0 110u 100% 24u "Pulsa Instalar para preparar el launcher. El contenido del juego se obtiene al elegir una versión."
  Pop $0
  SetCtlColors $0 BCA7CE 100C19
  nsDialogs::Show
FunctionEnd

Function TpsOptionsLeave
  ${NSD_GetState} $TpsOptionsDesktopCheckbox $TpsDesktopChoice
FunctionEnd

!macro TPS_THEME_FUNCTIONS PREFIX
Function ${PREFIX}TpsThemeInit
  Push $0
  SetCtlColors $HWNDPARENT F5EAFE 100C19
  GetDlgItem $0 $HWNDPARENT 1028
  SetCtlColors $0 BCA7CE 100C19
  System::Call '*(i 1) p.r0'
  System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 20, p r0, i 4)'
  System::Free $0
  Pop $0
FunctionEnd

Function ${PREFIX}TpsCopyShow
  Push $0
  Push $1
  Push $2
  Push $3
  Push $4
  SetCtlColors $mui.InstFilesPage F5EAFE 100C19
  SetCtlColors $mui.InstFilesPage.Text F5EAFE 100C19
  System::Alloc 16
  Pop $4
  System::Call 'user32::GetClientRect(p $mui.InstFilesPage, p r4)'
  System::Call '*$4(i, i, i.r2, i.r3)'
  StrCpy $TpsPageWidth $2
  StrCpy $TpsPageHeight $3
  IntOp $TpsPortraitWidth $3 / 2
  IntOp $TpsTextWidth $2 - $TpsPortraitWidth
  IntOp $TpsTextWidth $TpsTextWidth - 16
  System::Call 'user32::GetWindowRect(p $mui.InstFilesPage.Log, p r4)'
  System::Call '*$4(i, i.r0, i, i.r1)'
  IntOp $1 $1 - $0
  System::Call 'user32::SetWindowPos(p $mui.InstFilesPage.Log, p 0, i 0, i 0, i $TpsTextWidth, i r1, i 6)'
  System::Call 'user32::GetWindowRect(p $mui.InstFilesPage.ProgressBar, p r4)'
  System::Call '*$4(i, i.r0, i, i.r1)'
  IntOp $1 $1 - $0
  System::Call 'user32::SetWindowPos(p $mui.InstFilesPage.ProgressBar, p 0, i 0, i 0, i $TpsTextWidth, i r1, i 6)'
  System::Call 'user32::GetWindowRect(p $mui.InstFilesPage.Text, p r4)'
  System::Call '*$4(i, i.r0, i, i.r1)'
  IntOp $1 $1 - $0
  System::Call 'user32::SetWindowPos(p $mui.InstFilesPage.Text, p 0, i 0, i 0, i $TpsTextWidth, i r1, i 6)'
  System::Free $4
  SendMessage $mui.InstFilesPage.Log 0x1001 0 0x190C10
  SendMessage $mui.InstFilesPage.Log 0x1024 0 0xFEEAF5
  SendMessage $mui.InstFilesPage.Log 0x1026 0 0x190C10
  System::Call 'uxtheme::SetWindowTheme(p $mui.InstFilesPage.ProgressBar, w "", w "")'
  SendMessage $mui.InstFilesPage.ProgressBar 0x0409 0 0xFF80C4
  SendMessage $mui.InstFilesPage.ProgressBar 0x2001 0 0x302038
  InitPluginsDir
  File /oname=$PLUGINSDIR\skin-install.bmp "${TPS_ART_ROOT}\skin-install.bmp"
  IntOp $2 $TpsPageWidth - $TpsPortraitWidth
  System::Call 'user32::CreateWindowExW(i 0, w "STATIC", w "", i 0x5000000E, i r2, i 0, i $TpsPortraitWidth, i $TpsPageHeight, p $mui.InstFilesPage, p 0, p 0, p 0) p.r0'
  StrCpy $TpsSkinControl $0
  System::Call 'user32::LoadImageW(p 0, w "$PLUGINSDIR\skin-install.bmp", i 0, i $TpsPortraitWidth, i $TpsPageHeight, i 0x10) p.r0'
  StrCpy $TpsSkinBitmap $0
  SendMessage $TpsSkinControl 0x0172 0 $TpsSkinBitmap
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

Function ${PREFIX}TpsCopyLeave
  System::Call 'user32::DestroyWindow(p $TpsSkinControl)'
  System::Call 'gdi32::DeleteObject(p $TpsSkinBitmap)'
  StrCpy $TpsSkinControl 0
  StrCpy $TpsSkinBitmap 0
FunctionEnd
!macroend

!insertmacro TPS_THEME_FUNCTIONS ""
!insertmacro TPS_THEME_FUNCTIONS "un."
