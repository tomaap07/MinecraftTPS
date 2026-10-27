!define TPS_ART_ROOT "${__FILEDIR__}"
!addplugindir "${TPS_ART_ROOT}\build"
!define MUI_UI "${TPS_ART_ROOT}\build\tp-installer-ui.exe"
!define MUI_UI_HEADERIMAGE "${TPS_ART_ROOT}\build\tp-installer-ui.exe"
!define MUI_BGCOLOR "100C19"
!define MUI_TEXTCOLOR "F5EAFE"
!define MUI_INSTFILESPAGE_COLORS "C480FF 100C19"
!define MUI_CUSTOMFUNCTION_GUIINIT TpsThemeInit
!define MUI_CUSTOMFUNCTION_UNGUIINIT un.TpsThemeInit
!define MUI_WELCOMEPAGE_TITLE "$(TpWelcomeTitle)"
!define MUI_WELCOMEPAGE_TEXT "$(TpWelcomeIntro)$\r$\n$\r$\nTP LAUNCHER v${VERSION}$\r$\n$\r$\n$(TpWelcomeBody)"
!define MUI_FINISHPAGE_TITLE "$(TpFinishTitle)"
!define MUI_FINISHPAGE_TEXT "$(TpFinishBody)"
!define MUI_FINISHPAGE_RUN_TEXT "$(TpRun)"
!define MUI_UNFINISHPAGE_TITLE "$(TpGoodbyeTitle)"
!define MUI_UNFINISHPAGE_TEXT "$(TpGoodbyeBody)"

!macro NSIS_HOOK_POSTINSTALL
  ${If} $TpsDesktopChoice = ${BST_CHECKED}
  ${AndIf} $PassiveMode <> 1
  ${AndIfNot} ${Silent}
    Call CreateOrUpdateDesktopShortcut
  ${EndIf}
!macroend
