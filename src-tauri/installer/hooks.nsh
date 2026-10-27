!define TPS_ART_ROOT "${__FILEDIR__}"
!define MUI_BGCOLOR "100C19"
!define MUI_TEXTCOLOR "F5EAFE"
!define MUI_INSTFILESPAGE_COLORS "C480FF 100C19"
!define MUI_CUSTOMFUNCTION_GUIINIT TpsThemeInit
!define MUI_CUSTOMFUNCTION_UNGUIINIT un.TpsThemeInit
!define MUI_WELCOMEPAGE_TITLE "Tu próximo mundo empieza aquí."
!define MUI_WELCOMEPAGE_TEXT "Bienvenido a TP LAUNCHER v${VERSION}.$\r$\n$\r$\nVersiones, modpacks y herramientas para preparar tu siguiente aventura en Minecraft Java.$\r$\n$\r$\nElige dónde instalarlo, personaliza tus accesos directos y deja que TP Explorer te acompañe durante la instalación.$\r$\n$\r$\nPulsa Siguiente para comenzar."
!define MUI_FINISHPAGE_TITLE "TP LAUNCHER está listo"
!define MUI_FINISHPAGE_TEXT "La instalación de v${VERSION} ha finalizado.$\r$\n$\r$\nLos accesos directos se han preparado según tus preferencias.$\r$\n$\r$\nAbre el launcher para elegir tu versión y personalizar tu partida.$\r$\n$\r$\nTu próxima aventura te espera."
!define MUI_FINISHPAGE_RUN_TEXT "Abrir TP LAUNCHER"
!define MUI_INSTFILESPAGE_HEADER_TEXT "Preparando tu próxima aventura"
!define MUI_INSTFILESPAGE_HEADER_SUBTEXT "Instalando TP LAUNCHER. Puedes consultar los detalles del proceso."
!define MUI_INSTFILESPAGE_FINISHHEADER_TEXT "TP LAUNCHER instalado"
!define MUI_INSTFILESPAGE_FINISHHEADER_SUBTEXT "Todo listo para tu próximo mundo."
!define MUI_UNCONFIRMPAGE_TEXT_TOP "Se eliminará TP LAUNCHER de este equipo.$\r$\n$\r$\nRevisa el destino y las opciones de datos antes de continuar."
!define MUI_UNFINISHPAGE_TITLE "Hasta la próxima aventura"
!define MUI_UNFINISHPAGE_TEXT "TP LAUNCHER se ha desinstalado.$\r$\n$\r$\nGracias por usar TP LAUNCHER.$\r$\n$\r$\nTP Studios"

!macro NSIS_HOOK_POSTINSTALL
  ${If} $TpsDesktopChoice = ${BST_CHECKED}
  ${AndIf} $PassiveMode <> 1
  ${AndIfNot} ${Silent}
    Call CreateOrUpdateDesktopShortcut
  ${EndIf}
!macroend
