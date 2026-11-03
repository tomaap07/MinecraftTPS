# Informe de verificación TP Client

Fecha: 2 de noviembre de 2026. El coordinador completa aquí los resultados finales de la cohorte. Este informe no certifica todas las versiones ni sustituye pruebas dentro de Minecraft.

## Resultados observados

| Comprobación | Resultado y alcance |
| --- | --- |
| Compilación Fabric final | `npm run build:client`: **24 pruebas Java aprobadas** y `tp-client/build/libs/tp-client-0.1.0.jar` para Minecraft 1.21.1 / Java 21, SHA-256 `7ebbf3a4c64f6d9e1ec3f3cd931037803bd684e6390145c218c30e6941d15ba6`. Incluye la última corrección del espacio de avisos, comprobada con la prueba de geometría indicada abajo. El resto del QA interactivo corresponde al mismo diseño antes de ese ajuste de avisos. |
| Desarrollo real | El cliente Fabric de demostración arrancó en `tp-client/run`, separado de cuentas y mundos de usuario. Se entró en un mundo de demostración y se observaron FPS, XYZ, bioma, dirección, reloj, armadura de diamante con cuatro piezas al 100%, manos vacías, CPS y teclas. La latencia individual se mostró como no aplicable. Son observaciones de la compilación de desarrollo, no una sesión Microsoft premium. |
| Distribución del HUD | `HudLayoutTest`: **cuatro pruebas** para veinte widgets y zonas de chat, avisos y hotbar. Las tres pruebas iniciales cubren tamaños desde 256×180 hasta 1920×1080. `noticesUseTheReservedSpaceAndNeverCoverMinecraftToasts` verifica anchos 180/224/256/320/428/640/960/1920: el aviso queda dentro de la reserva compartida y no tapa el toast de Minecraft. Esta última geometría aún no tiene QA visual posterior al ajuste. |
| Menú principal nuevo | Se observó el TitleScreen oscuro nuevo con modelo 3D en Minecraft Fabric 1.21.1/OpenGL de demostración, se abrió Personalizar y Opciones originales, y se comprobó Audio OFF/ON y su persistencia JSON al volver. Jugar demostración permitió entrar en el mundo; Multijugador y Realms conservaron el bloqueo del juego para esa sesión. No se escuchó ni midió el audio como parte de esta comprobación. |
| Editor y render del HUD | F8 abrió el editor real. En ventana maximizada de 1920×1032 cupieron los veinte widgets; se arrastró FPS desde la esquina a una posición central, se comprobó su posición guardada y se restableció con R. Antes del último ajuste de avisos, la ventana pequeña de 856×512 informó «Sin espacio: 4» y mantuvo libres las dos líneas del pie del editor: no se forzaron widgets sobre esos controles. |
| Menú de pausa y categorías | En la compilación anterior al último ajuste de avisos, TP CLIENT del menú de pausa abrió `ClientScreen`. Se corrigió el fondo blur que tapaba los textos personalizados; se observaron textos nítidos y controles separados a 856×512. Exploración seleccionó Coordenadas y se comprobaron los filtros Técnico y PvP. Todos/Ninguno se aplica al filtro visible. |
| Captura F9 | Se pulsó F9 en la partida de desarrollo real: apareció la notificación Saved Screenshot y se creó `2026-11-02_19.16.03.png`, de **332023 bytes**. En esa interacción se detectó que el aviso TP podía montarse sobre el toast Vanilla en una ventana de 856 píxeles; se corrigió compartiendo `HudLayout.notificationArea` entre el HUD y `Notices`, con cobertura automática nueva. No se atribuye una observación visual posterior a esa corrección. |
| Núcleo final | `cargo test --locked --quiet`: **135 aprobadas, 21 omitidas, cero fallidas** (156 total). Incluye el cierre real de un subproceso de prueba al liberar el Job Object de Windows. Las pruebas omitidas no se presentan como aprobadas. |
| Instalación/reversión revisada | `cargo test --quiet tp_client::tests`: **7 aprobadas**, cero fallidas. Incluye el bloqueo real de un archivo en Windows, preservación de config manual, rechazo de respaldo corrupto, integridad de JAR y consentimiento para sustituir Companion. |
| FFmpeg sintético revisado | `cargo test --quiet ffmpeg_synthetic_mp4_is_verified_without_audio -- --ignored`: **1 aprobada**. MP4 H.264 sintético verificado con ffprobe, sin audio y sin capturar pantalla, cámara, micrófono o partida. Una fuente 320 × 180 sigue en esa resolución; no se amplía artificialmente a 720p. |
| Typecheck final | `npm run check`: **cero errores y cero advertencias** tras los cambios de esta cohorte. |
| Catálogo de traducciones | `npm run check:ui`: 1468 textos tras añadir Explorar launcher, sin claves faltantes en sus diez idiomas principales. Esto no prueba que todas las cadenas Java/Svelte nuevas estén traducidas. |
| Editor de capas | El coordinador registró `node scripts/test-cape-ui.mjs` aprobado para pincel, deshacer/rehacer, guardar, filtros de compatibilidad y equipamiento con IPC simulado. Las validaciones reales de archivos viven en Rust; el test web no prueba un servicio de cosméticos remoto. |
| Centro de rendimiento | `cargo test --quiet performance_center::tests`: **8 aprobadas**, cero fallidas. Los presets no se ofrecen para Alpha/Beta/Classic/1.0–1.6 o formatos desconocidos; esto evita escribir opciones modernas en archivos incompatibles. |
| Backend comunitario de staging | Pase final de `npm test` dentro de `tp-api`: **29 aprobadas**, con HTTP/PNG y PostgreSQL/PGlite/RLS. No se desplegó ni se aplicó la migración al servicio remoto. Véase `docs/cloud-staging.md`. |
| Comentarios de código | Auditoría de **329 archivos**: cero comentarios ordinarios, dos avisos legales y seis archivos wrapper con licencias preservadas. No se eliminaron los avisos legales para satisfacer la limpieza de comentarios. |
| Secretos | Revisión de **998 textos, incluidos 542 blobs históricos**: cero coincidencias con los secretos locales comprobados y **dos alertas revisadas como falsos positivos** de fixtures de Crash Doctor. No se presenta como un resultado de «cero alertas» ni como garantía absoluta frente a cualquier secreto. |
| CI | Los jobs de interfaz/API en Ubuntu y núcleo/mod en Windows pasaron en [la revisión 2bb41ff](https://github.com/tomaap07/MinecraftTPS/actions/runs/41569718676). Ese resultado corresponde al código anterior al cambio posterior de Explorar launcher. |
| Explorar launcher, 3 de noviembre | Svelte volvió a terminar con cero errores/advertencias; `node scripts/test-launcher-browse-ui.mjs` pasó con IPC simulado: cerrar el asistente no inventa cuenta, licencia ni setup completo, y Jugar sigue bloqueado si falta verificación. La aplicación nativa de desarrollo se observó abierta en Jugar sin el asistente. |

## Escenarios negativos cubiertos en pruebas locales

- Un JAR cuya huella no coincide no se instala ni se elimina como propio.
- Una configuración corrupta se conserva; no se sustituye por defecto como parte de una instalación.
- Un archivo de configuración bloqueado provoca reversión del nuevo JAR y respaldo de Companion sin tocar archivos previos.
- La retirada con restauración no sobreescribe ediciones posteriores del jugador.
- Un respaldo de Companion alterado se rechaza antes de restaurar la configuración.
- Los PNG locales requieren dimensiones exactas, firma válida y límites de tamaño/asignación; el backend los vuelve a codificar y detecta alteraciones del archivo que intenten pasar por deduplicación. Las validaciones del servicio comunitario son un contrato independiente.
- Los presets de `options.txt` conservan bytes originales y detectan cambios manuales o corrupción del respaldo.
- Los diagnósticos devuelven categorías y pasos fijos en lugar del contenido privado del registro.
- Las muestras locales inválidas, viejas, sin foco/mundo o limitadas intencionadamente no generan un diagnóstico de FPS bajos. `PerformanceWarmupTest` comprueba que el calentamiento mida tiempo de mundo activo, sin sumar la permanencia en menús.
- El grabador exige consentimiento antes de iniciar procesos y valida sus opciones y los identificadores de archivo.

## Pruebas manuales todavía por registrar

Faltan los recorridos restantes: servidores, navegación completa por teclado, más valores de GUI Scale/resoluciones, herramientas y efectos con alturas variables, zoom mantenido, cambio de dimensión, shaders, cierre del launcher con el juego abierto, cosméticos locales y recuperación interactiva de configuración dañada. También falta revisar visualmente la última distribución de avisos después del arreglo detectado con F9. El intento automatizado de pulsar Shift derecho no abrió el menú; el mapeo predeterminado se confirmó en la configuración, pero no se da por verificado ese atajo en una interacción real. El audio del menú necesita escucha, volumen y confirmación de parada al entrar en el mundo; activar sus controles y observar el JSON no prueba reproducción audible.

La evidencia visual local está en `.codex-tmp/tp-client-qa/`: `title-menu.png`, `client-categories.png`, `hud-editor-small.png` y `hud-editor.png` (ventana maximizada de la revisión anterior). Esos archivos de QA no se incluyen automáticamente en la distribución. La compilación NSIS final y su publicación se registran por separado; este informe no afirma que ya se haya subido un instalador.

El acceso Microsoft real sigue bloqueado por `INVALID_APP_REGISTRATION` del servicio para el Client ID usado. No se ha verificado una sesión premium completa posterior a la notificación de aprobación. Las pruebas de desarrollo del mod no eliminan esa comprobación del launcher.

No se han probado NAT restrictivo, dos redes remotas, relay, voz, publicación comunitaria de producción, telemetría externa, rendimiento A/B, firma Authenticode o todas las versiones de Minecraft. Los bloqueos y partes locales se detallan en `docs/TP_LAUNCHER_IMPLEMENTATION.md`.
