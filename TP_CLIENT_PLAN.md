# Plan y matriz verificable de TP Client

La unidad de entrega es un JAR compilado para una combinación concreta de Minecraft/cargador/Java, no el catálogo de todas las versiones que TP Launcher puede iniciar. La auditoría y checklist globales están en `docs/TP_LAUNCHER_AUDIT.md` y `docs/TP_LAUNCHER_IMPLEMENTATION.md`.

## Matriz de construcción

| Entorno | Estado |
| --- | --- |
| Minecraft 1.21.1 + Fabric + Java 21 | Objetivo construido; Yarn `1.21.1+build.3`, Fabric Loader `0.20.1`, Fabric API `0.116.17+1.21.1` |
| Vanilla 1.21.1 | Sin inyección ni conversión automática de loader. El mod exige seleccionar/crear una instancia Fabric explícitamente. |
| Fabric 1.21.9 / 26.3 / snapshots | Este JAR no se ofrece como compatible. Un adaptador necesita compilarse y probarse por separado. |
| Forge / NeoForge / Quilt / OptiFine | Sin variante TP Client verificada; no se reutiliza un JAR Fabric incompatible. |
| Overlay nativo anterior | Controles retirados del panel nuevo y activación automática deshabilitada. No equivale al HUD nativo. |

## Contratos

- Configuración del mod: `config/tp-client.json`, JSON versión 1, rutas de perfiles confinadas y escritura temporal.
- Cosméticos: `config/tp-client-cosmetics.json` con `schemaVersion`, `equippedCape` como nombre local, `wings` y `crown`; PNG 64×32 dentro de `config/tp-client-cosmetics/`.
- Rendimiento: una muestra local limitada y opt-in en `config/tp-client/performance.json`; PID y carpeta deben coincidir con un proceso seguido. Sin credenciales, chat, IP o coordenadas en ese informe.
- Artefacto: `scripts/build-client.mjs` genera el JAR y la huella que Rust incorpora. Un manifest remoto firmado o kill switch necesita infraestructura y raíz de confianza independientes; no se simula con una huella sin firma.
- Backend comunitario: identidad TP independiente de Minecraft y operaciones autorizadas por servidor. Las tablas y políticas de staging no se consideran desplegadas por existir en Git.

## Riesgos y controles de aceptación

| Riesgo | Control y evidencia requerida |
| --- | --- |
| Dos HUDs o conflicto de mixins | Consentimiento al sustituir Companion, respaldo propio y no activar el overlay antiguo junto al mod |
| JAR incompatible o cambiado | Comparar versión/loader/Java/metadatos y SHA-256 antes de instalar o borrar |
| Pérdida de preferencias | Conservar config, detectar ediciones posteriores, prevalidar backup y probar un archivo bloqueado en Windows |
| Widgets solapados | Medición real, distribución con zonas protegidas y pruebas de geometría; QA visual en escalas/resoluciones distintas |
| PNG malicioso o remoto | Firma, dimensiones, tamaño y asignación acotados; re-encoding, nombres por hash y rutas locales |
| Métricas inventadas o privacidad | Solo APIs reales, disponibilidad explícita, sin tokens en mod/UI y sin telemetría remota implícita |
| Grabación inesperada | Default apagado, consentimiento en cada inicio, ventana seguida, sin audio, Job Object y MP4 verificado |
| Afirmar servicio inexistente | Estados de integración reales; probar roles, RLS, autorización y operaciones remotas antes de mostrar conectado |
| Material ajeno | Recursos TP originales, dependencias con licencias conservadas; ninguna marca, JAR o API privada de otro cliente |

## Secuencia de verificación

1. Compilar Java y ejecutar tests, regenerando el manifiesto de integridad.
2. Ejecutar tests Rust de instalación, configuración, PNG, opciones, diagnóstico y ciclo de vida de captura.
3. Comprobar Svelte con `npm run check`, flujos web con IPC simulado y el catálogo de traducciones sin confundir esos tests con un backend real.
4. Ejecutar el JAR en desarrollo aislado: menú principal, mundo, HUD, editor, cosméticos, controles, pantalla/GUI Scale y degradación sin launcher.
5. Preparar staging comunitario, políticas y pruebas multiusuario; desplegar únicamente cuando exista autorización y configuración adecuada.
6. Auditar secretos y recursos antes de empaquetar/publicar. Registrar por separado la firma digital pendiente y cualquier parte del alcance no verificada.

Los resultados concretos y limitaciones se actualizan en `TP_CLIENT_TEST_REPORT.md`. Este plan no da por terminadas las fases de social, P2P, voz, galería pública, recompensas certificadas o grabación instantánea por tener una interfaz o una migración local.
