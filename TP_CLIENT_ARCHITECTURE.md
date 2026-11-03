# Arquitectura de TP Client

## Separación de responsabilidades

1. **Svelte:** `TpClientPanel`, `CapeStudio`, `PerformanceCenter` y `TpClips` muestran estados reales e invocan comandos locales. No reciben tokens Microsoft.
2. **Rust:** `tp_client.rs` valida instancia/cargador/artefacto, instala con consentimiento y conserva respaldo; `cosmetics_library.rs` normaliza PNG; `performance_center.rs` valida muestras y aplica opciones reversibles; `tp_clips.rs` administra un encoder externo opcional.
3. **Java/Fabric:** `TpClient`, `ClientScreen`, `HudEditorScreen`, `HudRenderer`, `HudLayout`, `ConfigStore`, `Cosmetics` y el módulo `menu/` operan dentro del juego. No necesitan una ventana Tauri para dibujar widgets.
4. **Servicios comunitarios:** el esquema/contrato de staging se mantiene separado del catálogo público de servidores. Tener migraciones locales no significa que un servicio social esté desplegado.

El overlay y el puente anteriores permanecen como código de compatibilidad y reversión, pero sus controles de HUD fueron retirados del panel principal. La carga del perfil heredado desactiva su activación automática. La configuración del HUD nuevo pertenece al mod de la instancia; los mods externos opcionales conservan su selección por separado.

## Render y configuración

El mod usa eventos de inicialización, tick y render Fabric; las mediciones del estado no ejecutan consultas HTTP por cuadro. `HudRenderer` mide texto/equipo y prepara solicitudes para el algoritmo de distribución. `HudLayout` reserva zonas de juego y evita que armadura, herramientas y widgets de varias filas se coloquen usando una altura fija común. `HudEditorScreen` modifica las posiciones guardadas; el menú nativo usa controles oscuros originales y búsqueda/categorías.

`ConfigStore` conserva JSON versionado por instancia, valida tamaño y nombres de perfiles y usa escritura temporal y reemplazo atómico cuando el sistema lo permite. Una configuración dañada no impide iniciar Minecraft; se conserva copia antes de utilizar valores seguros. Los perfiles no importan código ejecutable.

## Artefacto, instalación y reversión

El JAR se construye localmente con Fabric Loom/Yarn para 1.21.1. Rust incorpora una huella SHA-256 generada en esa misma compilación, además de versión/loader/Java esperados. No se descarga código TP de una URL arbitraria ni se declara firmado un archivo por tener una huella.

La instalación comprueba carpetas, límites y metadatos antes de cambiar archivos. Conserva la configuración original y la edición de modo aplicada, y crea un registro por instancia. Si la copia nueva falla, intenta revertir exclusivamente los bytes que escribió y conserva respaldos si otra edición o Windows impiden la reversión. Retirar el mod no sobreescribe una configuración editada después. Un error de respaldo se detecta antes de restaurar ajustes.

No se generan loaders silenciosamente en Vanilla; las mejoras que requieren Fabric se aplican a una instancia elegida expresamente. Instalar TP Client y sustituir Companion son acciones distintas que necesitan selección en la interfaz.

## Canal de rendimiento y servicios ausentes

La primera integración del nuevo mod usa configuración estática y una muestra JSON local acotada cada cinco segundos si el usuario la activa. Rust comprueba schema, números finitos, edad de la muestra y que su PID/carpeta correspondan a un juego seguido por el launcher. El FPS Doctor descarta calentamiento, falta de mundo/foco y límites de FPS deliberados. No es un canal bidireccional social ni una prueba antifraude.

No se copia ningún token oficial al mod. Los cosméticos locales nunca se publican por equiparlos. Las operaciones remotas de amigos, aprobación, misiones, roles o salas necesitan identidad TP y validación de servidor. El inventario oficial de Minecraft sigue siendo independiente.

## Vídeo y costes conocidos

La implementación inicial de TP Clips graba manualmente los próximos 15/30/60/120 segundos de una ventana Minecraft seguida por el launcher, mediante FFmpeg/ffprobe externos. No tiene audio ni micrófono, no captura el escritorio entero y permanece apagada por defecto. El proceso se oculta y se vincula al launcher mediante un Job Object de Windows para que no siga grabando tras cerrar TP. El MP4 se verifica antes de incorporarlo a la lista.

Esto **no implementa** un búfer instantáneo en RAM, captura WGC/Media Foundation, encoder hardware ni WASAPI. La captura GDI de una ventana ocluida, minimizada o con fullscreen exclusivo puede producir imagen negra. El consumo de CPU/FPS no se ha medido mediante benchmark A/B.

## Licencias y procedencia

Código TP: GPL-3.0-only, con el aviso de licencia incluido en el JAR. Gradle Wrapper y bibliotecas mantienen sus licencias. Las APIs de Fabric/Minecraft se usan como dependencia del entorno; no se incorpora código, marca, servicio privado, música ni assets de Millida, Lunar o Essential. Los recursos de cosméticos y del tema se generan dentro del proyecto y su inventario se registra en el módulo correspondiente. Esta descripción técnica no constituye una garantía jurídica.
