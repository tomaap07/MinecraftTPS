# TP LAUNCHER 0.1.2 · TP LABS

Fecha de validación: 27 de octubre de 2026.

El instalador sustituye el antiguo personaje por un render de la skin Fariiuh autorizado por el responsable del proyecto. Incorpora un asistente más grande, tipografía Segoe UI, controles negros y violetas, barra de desplazamiento propia y botones de ventana personalizados. El launcher también usa su propio marco. Se conservan la licencia, los accesos directos, instalación por usuario, WebView2, actualización y desinstalación.

La detección de Java ejecuta sus procesos sin consola, con límite de tiempo, drenaje de salida y caché. La enumeración de instalaciones y runtimes pasa al trabajo en segundo plano para no bloquear el hilo de interfaz. Las cuentas Microsoft siguen utilizando el almacén cifrado local existente para renovar la sesión; no se guardan contraseñas. La persistencia requiere conservar el token de renovación cifrado en ese equipo.

El primer inicio detecta el idioma de Windows y permite escoger idioma, instalar Java opcionalmente y elegir un perfil local o Microsoft. Los diez idiomas principales tienen 1457 entradas locales; catalán y esperanto conservan cobertura parcial claramente indicada. Las traducciones se prepararon con asistencia automática y revisión de términos críticos, sin sustituir una revisión lingüística por hablantes nativos. MyMemory traduce únicamente resúmenes públicos de contenido cuando el usuario lo solicita o activa la opción; presenta el original inmediatamente, usa caché y limita las consultas. Véase [traducciones](translations.md).

El catálogo ofrece Instalar y utiliza el archivo más reciente compatible. Para mods, shaders y recursos, el diálogo muestra la imagen y solo permite destinos instalados compatibles, incluidos modpacks listos. Los shaders requieren un motor compatible. Los modpacks usan su versión más reciente disponible. La selección se valida de nuevo en Rust antes de descargar.

La navegación usa iconos vectoriales semánticos y elimina duplicados. La imagen de Misode se conserva con su licencia MIT; las otras herramientas comunitarias sin permiso explícito verificado mantienen iconos neutrales. La revisión y fuentes están en [logos de comunidad](community-logos-2026-10-27.md). No se certifica jurídicamente toda la aplicación ni se deduce autorización general de la existencia de una API.

Se corrige además el ciclo de ResizeObserver de los visores 3D y del HUD: las medidas se agrupan por cuadro, se omiten tamaños idénticos u ocultos y se cancelan los observadores al desmontar.

## Validación

- `npm run check`: 0 errores y 0 advertencias.
- `npm run check:ui`: claves completas en diez idiomas principales y variantes de inglés; cobertura parcial adicional conservada.
- `cargo test --quiet`: 104 aprobadas, 19 ignoradas, 0 fallidas. Incluye ejecución Windows sin consola y cancelación por tiempo límite.
- Pruebas de primer inicio contra la interfaz de producción mediante Edge aislado e IPC simulado: diez idiomas, detección de Windows, Java opcional, omisión sin descargar, perfil local, persistencia y minimizar; sin errores de ejecución del navegador.
- Pruebas de compatibilidad del instalador de contenido y del observador de tamaño aprobadas. Las pruebas previas de iconos, consola, skins, HUD y auditor de secretos también aprobaron.
- Compilación de producción Rust/Svelte y conectores completada; NSIS final reempaquetado después del último ajuste visual. La compilación de Vite advierte del tamaño de los chunks de la interfaz y visor 3D.
- Inspección visual del instalador final: bienvenida Fariiuh, licencia desplazable, colores y marco propio. No se instaló ni desinstaló la aplicación durante esta inspección.
- Contenido NSIS extraído: ejecutable idéntico al de producción, DLL nativa y conectores idénticos a sus artefactos de compilación. Se verificaron 19 chunks de interfaz y se descomprimieron 52 recursos realmente presentes en el ejecutable.
- Auditoría de secretos en archivos preparados para Git, historial, ejecutable y recursos descomprimidos: 587 archivos de texto analizados, 430 blobs históricos, 803 archivos comparados contra cuatro secretos locales, 0 coincidencias y 0 hallazgos de severidad alta. Las tres informaciones restantes son identificadores OAuth públicos. Un escáner heurístico no constituye garantía absoluta de ausencia de secretos.
- Comentarios: sin comentarios ordinarios pendientes en los formatos admitidos por el auditor; se conservan avisos legales y archivos de terceros.
- Microsoft Defender: análisis personalizado local del instalador final terminado sin amenazas detectadas. No se modificaron protecciones, exclusiones ni certificados. Esto no constituye una certificación de seguridad.

No se ejecutaron todas las versiones de Minecraft en esta actualización. Las pruebas anteriores de clientes y manifiestos se documentan en la versión 0.1.1; la compatibilidad de todas las versiones y módulos no se considera certificada por estas comprobaciones. No se automatizó un inicio de sesión Microsoft real ni la instalación final sobre el perfil del usuario.

## Artefacto

- Archivo: `TP-LAUNCHER-0.1.2-Windows-x64-Setup.exe`.
- Tamaño: 220691996 bytes.
- SHA-256: `a4a41017a7849027ab64d370783b1f45a79bbf68c7b7eef2c27e927250a4f049`.
- Ejecutable incluido SHA-256: `ba9d1bc37d0a701395b9df23f49237853bf5eff109eef7162b12eb568e7e657a`.
- Sin firma Authenticode configurada. El aviso «Windows protegió su PC» puede continuar por reputación SmartScreen. Véase [distribución Windows](distribution-windows.md).

El código y el instalador se publican en el mismo repositorio privado. No se incluyen `.env`, claves de plataformas, cachés de cuentas, tokens, datos de juego ni archivos de configuración privada. Los identificadores públicos de aplicación Microsoft se mantienen como identificadores públicos.
