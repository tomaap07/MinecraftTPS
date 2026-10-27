# Instalador TP LAUNCHER · TP LABS

El asistente NSIS usa un marco propio, superficies negras, acentos violetas, botones accesibles y Segoe UI con ClearType. La ilustración es un render de la skin pública Fariiuh, incluida por petición de su titular. No se utiliza el antiguo personaje TP Explorer.

La instalación conserva las páginas de licencia GPL, destino, menú Inicio, resumen y accesos directos. Se instala por usuario y mantiene la integración de WebView2, actualización, modo silencioso y desinstalación de Tauri. El idioma inicial sigue Windows entre los diez idiomas configurados. El primer inicio del launcher pregunta el idioma y ofrece Java opcional y perfil local o Microsoft.

## Arte y licencias

- `static/branding/fariiuh-skin.png`: skin facilitada y autorizada por el autor del proyecto para su bienvenida. No contiene sesión, UUID ni credenciales.
- `fariiuh-welcome.png`: render transparente mediante skinview3d/Three.js; regenerar con `node scripts/render-installer-character.mjs`.
- `sidebar.bmp`, `header.bmp`, `skin-install.bmp`: composiciones del logo propio y ese render, a cuatro veces la resolución original para mejorar el escalado. Regenerar con `python scripts/prepare-installer-assets.py`.
- `native-ui.cpp`: tema nativo de los controles y marco del propio instalador. No cambia el tema ni la configuración de seguridad de Windows.
- `dialog-templates.json`: recursos de diálogo de NSIS 3.13, con geometría y tipografía modificadas por TP LABS. Se conserva `NSIS-LICENSE.txt`.
- La plantilla de Tauri conserva su aviso y `TAURI-LICENSE-MIT.txt`.
- Segoe UI se rasteriza desde la instalación de Windows. No se redistribuye su archivo de fuente.

## Compilación

`npm run tauri build` compila primero el tema nativo x86 de NSIS con las herramientas C++ de Visual Studio, después los conectores, la interfaz y el instalador. Los objetos y DLL generados quedan en `installer/build`, ignorado por Git. La compresión LZMA por archivo evita tener que descomprimir todo el paquete para cargar un recurso tardío del asistente.

La salida actual es `src-tauri/target/release/bundle/nsis/TP LAUNCHER_0.1.2_x64-setup.exe`. Las pruebas de páginas previas no instalan la aplicación. Un ejecutable de prueba que use un recurso WebView2 simulado no es un instalador distribuible.

No se dispone de certificado de firma Authenticode. Compilar y comprobar malware no establece reputación SmartScreen; no se garantiza que Windows deje de avisar sobre un programa nuevo sin firma. No se añaden exclusiones ni se desactiva ninguna protección.
