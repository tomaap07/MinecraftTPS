# TP LAUNCHER

Launcher de Minecraft Java para Windows con Tauri 2, Rust, Svelte 5 y TypeScript. Los clientes, runtimes, loaders y mods se descargan desde sus proveedores durante la instalación.

## Licencia y marca

El código original de TP LAUNCHER se distribuye bajo **GNU GPLv3**, identificada
como `GPL-3.0-only`. El texto íntegro está en [LICENSE](LICENSE), obtenido de la
[publicación oficial de GNU](https://www.gnu.org/licenses/gpl-3.0.txt).

La licencia del código no concede una licencia de marca para presentar una
distribución modificada como el producto oficial **TP LAUNCHER**. El nombre
comercial y la identidad visual —incluido el logotipo propio de
`static/logos/tp-launcher.svg`— quedan reservados a sus titulares. Las referencias
descriptivas y la atribución no deben sugerir aprobación o afiliación. Esta reserva
de marca no limita los derechos que la GPL concede sobre el código.

Las dependencias, fuentes, imágenes aportadas por usuarios, marcas y archivos de
terceros conservan sus propias licencias y créditos; no se relicencian como obras
originales del proyecto. El SVG de la nueva marca está construido con geometría
propia y no contiene texturas extraídas del juego. No se afirma una garantía de
ausencia de reclamaciones de terceros.

**TP LAUNCHER no es un producto oficial de Minecraft. No está aprobado por Mojang
ni asociado con Microsoft.**

## Desarrollo

Necesitas Node.js 22 o superior, Rust estable con MSVC, las herramientas de compilación C++ y el SDK de Windows de Tauri. Para empaquetar TP Companion configura JAVA_HOME con un JDK 21 o superior que incluya javac. El instalador suministra WebView2 offline.

```sh
npm ci
npm run tauri dev
```

```sh
npm run check
npm run check:ui
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

## Jugar y crear

- **Jugar:** catálogos Vanilla y loaders, personaje integrado en el fondo con una de las poses del catálogo elegida al iniciar, skin oficial o vista previa del nick offline y aspecto predeterminado obtenido de servicios públicos o recursos locales disponibles; Java automático, RAM y estadísticas por versión.
- **Mods y modpacks:** búsqueda Modrinth/CurseForge con filtros y páginas; fichas con descripción, galería, archivos y dependencias; instalación de un archivo concreto con sus dependencias obligatorias.
- **Modpack Studio:** instancias aisladas con Vanilla, Fabric, Forge, NeoForge, Quilt u OptiFine; gestión de mods, exportación ZIP/`.mrpack` e importación de formatos MinecraftTPS, Modrinth y CurseForge. Incluye progreso por etapas y archivo, avisos de éxito, descargas por streaming y reintentos con caché verificada. Las exportaciones excluyen OptiFine y binarios con permiso no verificable, muestran un informe de omisiones y pueden referenciar descargas Modrinth verificadas por hash. La instalación original permanece intacta.
- **Cuenta:** sesión Microsoft, skin y capas oficiales, armario local persistente, importación por nick y sincronización de una skin guardada.
- **Estudio 3D:** catálogo de poses, poses propias, rotación XYZ de siete articulaciones, posición del personaje, luz, cámara y segunda capa; renders PNG transparentes en HD y copia de imagen al portapapeles.
- **Biblioteca:** instalaciones locales, capturas de las distintas instancias, mundos y accesos a resourcepacks y shaderpacks. Los registros de consola se conservan al cerrar Minecraft.

## Integración de CurseForge

**Mods y modpacks** consulta CurseForge al seleccionar su proveedor. La clave se lee en ejecución desde `CURSEFORGE_API_KEY`, `.env` en desarrollo (debug) o `integrations.env` en la carpeta de datos local. No hay claves integradas en el código, frontend ni ejecutable; no se distribuye la clave del desarrollador. El parser conserva los signos `$` literalmente. Se usan `x-api-key`, `Accept: application/json` y un User-Agent que identifica a TP LAUNCHER.

El catálogo usa `gameId=432`, `classId=6` para mods y `classId=4471` para modpacks. El backend comunica los errores de autorización o de descarga del proveedor en español. Cuando un autor restringe la descarga externa de un archivo de un modpack, la instancia queda pendiente y muestra su página oficial y el botón para importar la versión exacta. Se comprueban tamaño y hash antes de aceptarla. [Detalles de instalación y el caso RLCraft](docs/modpack-installation.md).

Los archivos de credenciales locales están excluidos de Git y del bundle. El acceso y la distribución de esta integración están sujetos al acuerdo del titular con CurseForge; disponer de una clave no concede permiso general de marca ni permite compartirla. La clave que antes figuraba en el código debe revocarse antes de publicar otra distribución.

CurseForge limita cada consulta a 10.000 resultados accesibles. El launcher respeta este límite e informa cuando el proveedor restringe resultados o descargas; los filtros permiten explorar subconjuntos del catálogo.

## Archivos del usuario

Los datos existentes conservan `%APPDATA%/MinecraftTPS/`. Las instalaciones nuevas usan `%APPDATA%/LauncherTPS/`; Ajustes permite seleccionar otra carpeta. La configuración y la bóveda de cuentas permanecen en `%APPDATA%/LauncherTPS/`.

| Carpeta o archivo | Contenido |
| --- | --- |
| `versions`, `libraries`, `assets`, `runtime` | Archivos comunes de Minecraft y Java |
| `instances/<nombre>-<id>/tps-pack.json` | Configuración de cada modpack propio |
| `instances/<nombre>-<id>/{mods,config,saves,...}` | Contenido aislado de la instancia |
| `skins_library/skins.json` | Índice del armario y sus PNG |
| `custom_poses.json` | Poses creadas por el usuario |
| `renders` | Exportaciones PNG de Poser |
| `playtime.json` | Tiempo acumulado por jugador y versión |
| `logs` | Registro persistente del launcher y de Java |
| `cache/content`, `cache/modpacks` | Descargas verificadas y archivos de importación para reintentos |

Las instancias Vanilla existentes conservan sus directorios anteriores. El tiempo se mide mientras el proceso del juego está abierto, con guardado periódico y al terminar la sesión.

## Arquitectura y contribuciones

- `src/routes/+page.svelte`: navegación, lanzamiento y configuración general.
- `src/lib/ModStudio.svelte` y `src-tauri/src/mod_studio.rs`: proveedores, archivos, dependencias e instancias.
- `src/lib/AccountWardrobe.svelte`, `CapePreview.svelte` y `src-tauri/src/skin_library.rs`: cuenta y biblioteca de skins.
- `src/lib/poses_catalog.ts`, `HeroShowcase.svelte`, `PoseStudio.svelte`, `SkinModelViewer.svelte` y `src-tauri/src/poser.rs`: catálogo, visor y herramientas de posado.
- `src-tauri/src/launcher_tools.rs`: estadísticas, apariencias públicas y capturas.
- `src-tauri/src/lib.rs`: autenticación, descarga, runtimes y proceso de Minecraft.
- `src-tauri/src/studio_progress.rs`, `src/lib/ContentProgress.svelte` y `ContentNotifications.svelte`: operaciones de instalación y notificaciones.
- `src-tauri/src/servers.rs`, `src/lib/ServersPanel.svelte`: lista NBT, consulta de servidores y LAN/e4mc.
- `src-tauri/src/launcher_settings.rs`, `SettingsSuite.svelte` y `SetupWizard.svelte`: ajustes, multicuentas y bienvenida.
- `src/lib/i18n.ts` y `src/lib/locales`: 16 idiomas, traducción reactiva y cobertura comprobable de los catálogos.

Puedes añadir poses con nuevas rotaciones en `poses_catalog.ts` o guardar poses propias desde el estudio. Cada adaptación de proveedor devuelve los mismos tipos de proyecto y archivo para mantener independiente la interfaz.

El editor Poser de este launcher está construido con [skinview3d](https://github.com/bs-community/skinview3d) ([web y demostración](https://bs-community.github.io/skinview3d/)), creado por Kent Rasmussen, Haowei Wen, Sean Boult y sus colaboradores, y [Three.js](https://threejs.org/), de su equipo de autores y colaboradores. Ambos proyectos usan licencia MIT y sus créditos tienen enlaces directos dentro del estudio. El nombre del editor no implica que sea el programa MC Poser.

Las apariencias por nick se consultan en los servicios oficiales de Mojang/Minecraft, con respaldo de texturas de [MCHeads](https://mc-heads.net/) y [Minotar](https://minotar.net/) para identidades verificadas. La vista previa offline no cambia la identidad de autenticación del juego. La pose de bienvenida se conserva al cambiar de pestaña y se vuelve a elegir en el siguiente inicio.

Formatos y servicios: [Modrinth API](https://docs.modrinth.com/api/), [formato mrpack](https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack), [CurseForge API](https://docs.curseforge.com/rest-api/), [Fabric Meta](https://meta.fabricmc.net/), [Quilt Meta](https://meta.quiltmc.org/), [Forge](https://files.minecraftforge.net/) y [OptiFine](https://optifine.net/downloads).

El proyecto no incluye clientes, mods ni instaladores de OptiFine. Los archivos de terceros conservan las condiciones de sus autores.

## Plataforma TP

El catálogo propio de servidores está desplegado en [TP API](https://tp-api-hazel.vercel.app/api/v1/servers).
Consulta el [estado verificado de la plataforma](docs/tp-platform-status.md), la
[documentación del backend](tp-api/README.md) y la [compatibilidad de TP Companion](tp-companion/README.md).
El complemento es opcional y requiere Minecraft 1.21.1 con Fabric. Para empaquetarlo
desde un clon limpio configura `JAVA_HOME` con un JDK 21 o superior y ejecuta
`npm run tauri build`; se compila el JAR propio antes de construir el launcher.

El [HUD híbrido universal](docs/universal-hud.md) añade una ventana superpuesta para
Vanilla, OptiFine, Fabric y Forge en Windows, con teclas/CPS y editor por Shift derecho/F8.
Los datos internos siguen usando adaptadores compatibles. PresentMon aporta FPS externos
cuando Windows autoriza ETW; sus requisitos y el resultado real de verificación están documentados.

## Recursos visuales y revisión de derechos

La [auditoría de recursos](docs/asset-policy-audit.md) documenta fuentes, condiciones y sustituciones. La interfaz y el instalador usan arte propio de TP; algunos iconos y objetos pueden leer recursos de un JAR instalado localmente, con respaldo vectorial cuando faltan. Esa lectura no concede permisos de redistribución ni sustituye las condiciones de Mojang. Los logos de las integraciones tienen su procedencia en `static/logos/sources.json` y licencias en `static/legal/THIRD-PARTY-NOTICES.txt`, disponibles desde el pie de la aplicación. Las integraciones cuyos logos se retiraron conservan su nombre y funciones con símbolos propios. Esto no certifica jurídicamente el producto ni los contenidos de terceros.

### Datos del HUD y revisión de distribución

`npm run build:desktop` prepara TP Bridge (Java 8+), TP Companion y la interfaz.
Activa el HUD antes de iniciar Minecraft; un juego que ya estaba abierto debe
reiniciarse para cargar el puente. Consulta [HUD universal](docs/universal-hud.md)
y la [revisión actual de derechos e implementación](docs/implementation-review-2026-10-27.md).

## Preparación de la versión 0.1.1

- [Auditoría de secretos](docs/security-audit-2026-10-27.md): fuentes e historial, con hallazgos redactados; los Client IDs públicos no son secretos. Los archivos locales de credenciales nunca se incluyen.
- [Revisión funcional y de derechos](docs/implementation-review-2026-10-27.md): alcance, fuentes y permisos pendientes.
- [Catálogo y enlaces verificados](docs/validation/README.md): 920 manifiestos y 1.844 endpoints oficiales. La validación de metadatos no equivale a arrancar cada juego.
- [Asistente de instalación](src-tauri/installer/README.md): arte original TP Explorer, logo vigente y pasos de instalación por usuario.

Las compilaciones de Hit Color identificadas por huella que rompen los shaders de 26.3 se pausan únicamente cuando el launcher gestiona ese archivo; el resto del perfil se conserva. Las pruebas reales de arranque y sus límites figuran en el informe de validación de la versión.
