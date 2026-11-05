# TP Client

Mod cliente opcional de TP LABS para **Minecraft 1.21.1, Fabric y Java 21**. Se compila desde `tp-client/`. No es un cliente universal: este JAR no debe instalarse en Vanilla, OptiFine, Forge, NeoForge, otra versión de Fabric o una snapshot por analogía con el soporte del launcher.

## Instalación

En TP Launcher abre **Plugins → TP Client**, selecciona una instalación Fabric 1.21.1 completa o un modpack compatible y elige Mínimo o Completo. La descarga de Fabric API necesita autorización explícita si falta. La sustitución de TP Companion también pide una elección explícita y conserva una copia. Instalar el mod no convierte una instalación Vanilla.

La compilación de escritorio incluye el JAR propio en `clients/tp-client-0.1.0.jar`. El instalador por instancia compara su SHA-256 con el manifiesto incorporado en Rust y valida `fabric.mod.json`. Esta huella protege la integridad de ese artefacto local; no es una firma digital de un servicio de actualización remoto.

## Controles y módulos

- **Shift derecho:** menú TP Client. El atajo se puede reasignar en Controles de Minecraft.
- **F8, editor del HUD:** arrastrar widgets, ajustar su escala/opacidad y guardar posiciones en la instancia.
- **C:** zoom visual opcional; respeta las normas del servidor y puede desactivarse ante un conflicto de atajos.
- **F9:** captura local de Minecraft; F2 sigue disponible por el juego.

El menú organiza las funciones en HUD, PvP, Exploración, Técnico, Cosméticos, Notificaciones, Rendimiento y Ajustes. Solo se muestran métricas que el cliente puede obtener: FPS, latencia del servidor cuando existe, posición, orientación, entorno cargado, teclas/clics, durabilidad del equipo, efectos y estado del jugador. FPS no equivale a TPS. El modo individual no fabrica una latencia de servidor.

Los perfiles Supervivencia, PvP, Exploración, Técnico, Minimalista y Streaming son configuraciones locales. Si una resolución/escala GUI no permite colocar todos los widgets sin solapar zonas protegidas, se reduce la densidad o se informa del espacio insuficiente; no se promete que veinte widgets quepan a cualquier escala. La comprobación visual de cada resolución se registra por separado.

## Configuración, privacidad y cosméticos

Archivos relativos a la carpeta de juego de la instancia:

| Ruta | Contenido |
| --- | --- |
| `config/tp-client.json` | Configuración versionada de widgets, perfiles y preferencias |
| `config/tp-client-profiles/` | Perfiles JSON exportados por el usuario |
| `config/tp-client-import.json` | Perfil seleccionado para importar desde el menú |
| `config/tp-client-cosmetics.json` | Equipamiento local TP, independiente de capas oficiales |
| `config/tp-client-cosmetics/*.png` | Capas locales normalizadas de 64 × 32 |
| `config/tp-client/performance.json` | Última muestra de rendimiento local, solo si el usuario la activa |

Los tres cosméticos TP incluidos son **Nebulosa Infinita**, **Serafín Celestial** y **Eclipse Elemental**. Son recursos originales y locales; no desbloquean capas oficiales, no confieren vuelo, armadura o ventajas de combate y no son visibles para otros clientes Vanilla. Una galería o sincronización entre jugadores necesita el servicio comunitario autorizado y se documenta aparte.

El mod no recibe credenciales Microsoft, Xbox o Minecraft. Puede funcionar con el launcher cerrado. El informe de rendimiento es local y opcional: no contiene chat, IP o coordenadas ni se envía a un servidor externo. Los servicios sociales no se presentan como conectados si faltan identidad y backend.

## Retirar TP Client

Con Minecraft cerrado, usa **Desinstalar TP Client**. Se retira únicamente el JAR reconocido por su registro y huella; los mundos, mods ajenos y PNG propios se conservan. La restauración de una configuración anterior se bloquea si el archivo cambió después de la instalación: desmarca la restauración para conservar los ajustes nuevos. Fabric API permanece porque puede ser utilizada por otros mods. Un respaldo de Companion modificado o un destino ya ocupado no se sobrescribe.

## Compilar y verificar

```powershell
npm run build:client
cargo test --quiet --manifest-path src-tauri/Cargo.toml
```

Necesita JDK 21 con `javac`; el script lo busca en `JAVA_HOME`, `JDK_HOME`, PATH o la caché local de desarrollo. Gradle valida la huella de su distribución. El JAR de salida es `tp-client/build/libs/tp-client-0.1.0.jar`; el script actualiza `src-tauri/src/tp_client_release.json` con la huella de la compilación realizada.

Consulta [arquitectura](TP_CLIENT_ARCHITECTURE.md), [informe de pruebas](TP_CLIENT_TEST_REPORT.md) y [matriz completa de implementación](docs/TP_LAUNCHER_IMPLEMENTATION.md). Compilar no demuestra que todas las versiones, servidores o shaders sean compatibles.
