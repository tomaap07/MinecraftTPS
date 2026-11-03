# Auditoría del alcance TP Launcher / TP Client

Fecha: 2 de noviembre de 2026. Base inspeccionada: `4f17b1d`. Este documento registra evidencia de archivos y límites de comprobación; no certifica todas las versiones de Minecraft ni una auditoría jurídica. Los cambios en curso se detallan en [el registro de implementación](TP_LAUNCHER_IMPLEMENTATION.md).

## Arquitectura observada

| Plano | Implementación real | Responsabilidad |
| --- | --- | --- |
| Interfaz | Svelte 5 + TypeScript, `src/routes/+page.svelte`, `src/lib/` | Cuentas, instalaciones, catálogos, servidores, estudio 3D, HUD, ajustes |
| Núcleo | Rust + Tauri 2, `src-tauri/src/` | OAuth, DPAPI, archivos, versiones, Java, procesos, descargas, perfiles, overlay Windows |
| Juego | `tp-companion/`, `tp-bridge/`; nuevo `tp-client/` | Companion Fabric 1.21.1 existente; agente Java con adaptadores; TP Client se desarrolla como mod separado |
| Backend remoto | `tp-api/`, `supabase/migrations/` | Catálogo de servidores y worker, no un servicio social o de cosméticos |

Dependencias: la interfaz invoca comandos Rust. Rust determina rutas y compatibilidad, inicia Java y sigue procesos. El overlay recibe datos locales de un proceso seguido. El mod debe dibujar su HUD con APIs de Minecraft y funcionar con el launcher cerrado; no debe recibir credenciales oficiales. Los servicios comunitarios tienen una identidad distinta, cuya infraestructura no está presente en la base inspeccionada.

## Matriz de auditoría inicial

| Área | Estado inicial | Evidencia y límites |
| --- | --- | --- |
| Microsoft OAuth y licencia | Existe; acceso externo bloqueado | `microsoft_oauth.rs`, `microsoft_services.rs`, `account_licensing.rs`. Code + PKCE S256, OAuth de cliente público, Xbox/XSTS, MC Services, licencia y perfil. El servicio devuelve `INVALID_APP_REGISTRATION` para el ID efectivo. La aprobación comunicada por correo no enumera los IDs. No se completa una sesión con un 403. |
| Vault y cuentas | Existe | Módulo `token_storage` nativo DPAPI en `lib.rs`, `launcher_settings.rs`, `AccountWardrobe.svelte`. Perfiles locales condicionados a prueba válida de licencia. No tokens en UI. |
| Versiones y Java | Existe; cobertura parcial | `lib.rs`, `loader_catalogs.rs`, `process_probe.rs`, `first_run.rs`. Catálogos, Java por major, descarga portable y sondeos sin consola. La existencia de catálogos no prueba que todos los juegos arranquen. |
| Instancias/modpacks | Existe; incompleto frente al nuevo alcance | `mod_studio.rs`, `content_targets.rs`, `ModStudio.svelte`, `ContentInstallDialog.svelte`. Compatibilidad exacta y dependencias; mrpack y CurseForge; no se ha demostrado importar todos los formatos Prism/MultiMC. |
| Mundos | Parcial | Carpetas y exportaciones en `launcher_tools.rs`, `export_policy.rs`; no se observó un gestor completo de level.dat con seed, dificultad, duplicación y restauración como mundo separado. |
| Favoritos y servidores | Existe | `servers.rs`, `server_directory.rs`, `server_extras.rs`: ping, MOTD, icono, protocolo, servers.dat con pruebas de respaldo/concurrencia. Catálogo remoto distinto de favoritos. |
| LAN | Parcial | `servers.rs`: observación de puerto LAN y opción e4mc por catálogo. Esto no equivale a un túnel TP cifrado, señalización propia o relay operacional. |
| Centro de rendimiento | Nuevo | Había flags JVM y módulos opt-in (`launcher_tools.rs`, `managed_addons.rs`), no un centro de presets reversibles con comparación de options.txt y doctor local. Nueva implementación en `performance_center.rs` y `PerformanceCenter.svelte`. |
| HUD de escritorio | Existe; limitado por adaptador | `hud_windows.rs`, `launcher_overlay.rs`, `hud_bridge.rs`, `hud_transport.rs`, `TpHudOverlay.svelte`. La ventana transparente no es un mod in-game; fullscreen exclusivo tiene límites distintos de borderless. |
| Mod cliente | Existe Companion, nuevo TP Client | `tp-companion/build.gradle` fija Minecraft 1.21.1, Yarn build.3 y Java 21. No implica soporte 26.3, snapshots o Forge del mismo JAR. `tp-client/` debe compilar y validar por separado. |
| Skin/capas oficiales | Existe | `AccountWardrobe.svelte`, `skin_library.rs`; operaciones oficiales con cuenta. No concede capas no asignadas. |
| Cosméticos TP | Parcial | `launcher_social.rs`, `SocialProfile.svelte`, `TpCosmeticsPreview.svelte`: preferencias y preview local. Antes de esta tarea no demostraban un motor nativo del mod para capa, alas y corona. |
| Editor original PNG | Nuevo | El estudio de poses existente no equivale a un editor pixel-art de capas con undo/redo. Se implementa separadamente; hay que comprobar importación, dimensiones y exportación. |
| Galería pública moderada | No existe | No hay tablas de submissions, roles de moderación, buckets de pendientes, RLS ni panel de aprobación en la base observada. No debe simular éxito o publicar directamente. |
| Amigos/chat/voz | No existe servicio | `launcher_social.rs` guarda enlaces locales. Discord Rich Presence no es un sistema de amigos. Faltan autenticación TP, relaciones, bloqueos, moderación y señalización de voz. |
| Misiones con recompensas | No existe backend | `achievement_catalog.ts`, `AchievementsPanel.svelte` y secretos locales no son evidencia certificada por servidor ni un inventario de recompensas compartido. |
| Partners | Parcial | `HostingAdvisor.svelte`, `HostingOptions.svelte`, enlaces de comunidad; no acreditan acuerdos, conversiones, descuentos o permisos de marca. |
| Screenshots | Existe | `ScreenshotGallery.svelte`, `launcher_tools.rs`: páginas y previews de PNG local; no sube imágenes automáticamente. |
| Clips de vídeo | No existe | No se observó encoder H.264, captura de ventana, audio WASAPI o ring buffer de MP4. Un acceso a capturas F2 no equivale a TP Clips. |
| Distribución Windows | Existe; firma bloqueada | NSIS propio, `scripts/build-signed-windows.mjs`. No hay certificado confiable contratado/configurado; no se usará firma autofirmada para aparentar reputación. |
| Traducciones | Existe; ampliar con cadenas nuevas | Diccionarios locales + MyMemory opcional para contenido público; las vistas nuevas deben incorporar sus textos. Diez idiomas de la base no prueban traducción de nuevas cadenas. |

## Backend remoto comprobado en esta tarea

El agente coordinador consultó en modo lectura el proyecto Supabase `tp-api`: tablas observadas `tp_servers` (523 filas), `tp_api_keys` y `tp_rate_limits`, con RLS. No se observaron tablas ni configuración de identidad TP para amigos, cosméticos, misiones o salas. No se aplicaron migraciones de producción ni se desplegaron nuevos servicios.

Las migraciones locales existentes corresponden al catálogo de servidores, categorías, cron y extensión de red. Tener ese proyecto disponible no concede permiso para exponer una nueva galería ni elimina la necesidad de identidad, políticas RLS, buckets privados, pruebas multiusuario y moderación real.

## Riesgos de migración y mitigación

1. Vanilla usa `instances/<usuario>/options.txt` compartido por sus versiones; un preset no debe llamarse aislado por versión si cambia ese archivo. Los perfiles de loader y modpacks tienen sus carpetas existentes. El centro lo explica y conserva una copia exacta.
2. Presets: nunca editar con Minecraft abierto. Restaurar solo si el hash actual coincide con el último aplicado y el respaldo supera su hash. Conservar cambios manuales y rechazar enlaces de carpetas/archivos.
3. TP Client y Companion pueden competir por teclas y render. La instalación nueva debe ser opt-in y evitar activar ambos por accidente; conservar mods ajenos y no instalar un loader en Vanilla sin acción expresa.
4. JAR por versión: no instalar un JAR para 1.21.1 en 1.21.9/26.3, Forge o snapshots porque el launcher pueda iniciarlos. Compatibilidad del launcher y del mod son matrices distintas.
5. No cambiar authority, scopes, Client ID o entitlement por analogía con otro launcher; el rechazo de registro es un bloqueo externo conocido.
6. El cliente local no prueba acciones sociales ni gana privilegios de moderador. Un UUID o correo en JS no debe conceder roles.
7. Grabación, micrófono, telemetría externa, relay de pago y publicación de cosméticos necesitan consentimiento y presupuesto explícitos. Su ausencia debe mostrarse como no configurado.
8. La firma de un manifest remoto exige una raíz de confianza y una clave privada que no se publique; un SHA-256 sin firma sirve para integridad, no para atribuir autoría.

## Fuente comparativa

Los TXT del propietario incluyen referencias a Millida como investigación previa. Sus descripciones no son resultados de pruebas de TP Launcher. Esta implementación se escribe sobre el código propio y las APIs públicas de sus dependencias; no se incorporan recursos, JAR, endpoints privados, marca o claves de Millida. La inspección inicial cubre arquitectura y rutas críticas, no una lectura línea por línea de cada archivo del repositorio.

## Cierre verificable de esta cohorte

La tabla anterior conserva el estado inicial; no describe como ausente lo que ya se entregó después. El registro de implementación y `TP_CLIENT_TEST_REPORT.md` recogen el avance: JAR Fabric 1.21.1/Java 21 con 24 pruebas Java, 135 pruebas Rust aprobadas y 21 omitidas, ocho pruebas del centro de rendimiento, siete de instalación/reversión, MP4 sintético H.264 sin audio, editor de capas con IPC simulado y 29 pruebas de contratos comunitarios HTTP/PostgreSQL local. El backend remoto no cambió y el acceso Microsoft real sigue sujeto al rechazo de registro descrito.

Se corrigieron la reversión tras bloqueo de configuración, la restauración que podía pisar ediciones manuales, la lectura de respaldos de Companion antes de modificar ajustes, la validación de imágenes en deduplicación, límites de decodificación PNG y la aplicación de presets modernos a opciones legacy. Los procesos de captura se cierran con su dueño; el vídeo pequeño no se amplía artificialmente. Las pruebas utilizan archivos temporales, no mundos o cuentas personales.

El menú oscuro y sus controles se observaron en Minecraft de demostración; falta cerrar la aceptación visual del HUD dentro del mundo. Quedan fuera de la evidencia una sesión Microsoft válida, todas las versiones/loaders/shaders, NAT/relay, voz, producción social, firma Authenticode y benchmark A/B. La nueva CI se preparó, pero aún no se observó ejecutándose en GitHub.
