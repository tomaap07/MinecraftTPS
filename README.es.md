# TP LAUNCHER · TP LABS

Contacto: [tplabs.contacto@gmail.com](mailto:tplabs.contacto@gmail.com).

[English / Microsoft review summary](README.md) · [Código fuente](https://github.com/tomaap07/MinecraftTPS) · [Contacto y soporte](https://github.com/tomaap07/MinecraftTPS/issues)

TP LAUNCHER es un launcher independiente de **Minecraft: Java Edition** para Windows, desarrollado con Rust, Tauri 2, Svelte 5 y TypeScript. Gestiona instalaciones, contenido compatible, cuentas y herramientas de creación. La versión actual de la aplicación es **0.1.0**.

## Inicio de sesión y juego sin conexión

El primer inicio requiere conectar una cuenta Microsoft y verificar su licencia de Minecraft Java antes de continuar con el idioma y la instalación opcional de Java. El login usa el navegador del sistema, **OAuth 2.0 v2.0 Authorization Code con PKCE S256**, un estado aleatorio y la redirección registrada `http://localhost:8080`. La contraseña se introduce únicamente en la página de Microsoft.

La cadena Microsoft → Xbox Live → XSTS → Minecraft Services debe completarse, confirmar la licencia Java y obtener el perfil oficial. Los tokens de renovación se guardan cifrados con **Windows DPAPI** para el usuario actual de Windows; los tokens de acceso de Minecraft permanecen en memoria. Se utiliza el registro propio `6a4f8732-6bf7-49c3-a749-16607f97281d`, sin distribuir un client secret ni usar el Client ID de otro launcher.

El **Offline Mode está disponible después de verificar una cuenta Microsoft con licencia de Minecraft** en ese equipo. La biblioteca debe conservar sus credenciales protegidas y un registro de verificación vigente. La política actual del launcher mantiene la verificación durante **30 días**; después se necesita una nueva comprobación con conexión. No se presenta ese intervalo como un requisito específico de Microsoft. Crear o activar perfiles locales, incluidos los ya existentes, exige esta validación en Rust y en la interfaz.

Sin una cuenta apta, la opción permanece deshabilitada con el aviso: **«Para utilizar perfiles locales, debes iniciar sesión con una cuenta Microsoft al menos una vez en este equipo»**. El modo sin conexión no proporciona una sesión válida para servidores con autenticación oficial. La vista previa de skins por nombre de usuario tampoco autentica ese nombre.

¿Necesitas adquirir Minecraft? Visita la [tienda oficial de Minecraft](https://www.minecraft.net/es-es/store/minecraft-deluxe-collection-pc?tabs=%7B%22details%22%3A0%7D).

**El titular recibió una notificación de aprobación de la lista de acceso de AppID con fecha 30 de octubre de 2026.** Un intento posterior todavía devuelve HTTP 403 con `INVALID_APP_REGISTRATION` en Minecraft Services; aún no se ha confirmado un inicio de sesión completo. El launcher informa del Client ID efectivo y distingue los rechazos reconocidos sin exponer credenciales. Consulta la [guía de autenticación](docs/microsoft-client-configuration.md) y los [datos técnicos de la solicitud](docs/minecraft-services-request.md). Aprobar el acceso a la API no implica avalar el launcher como producto.

**TP LAUNCHER no es un producto oficial de Minecraft. No está aprobado por Mojang ni asociado con Microsoft.**

## Funciones

- **Jugar e instalaciones:** catálogos Vanilla y de loaders, Java compatible, RAM, instalaciones recientes y tiempo de juego. El personaje utiliza la skin conectada o una vista previa de skins por nombre de usuario.
- **Mods, modpacks, shaders y recursos:** búsqueda en Modrinth y CurseForge, categorías, imágenes, archivos y dependencias; instalación de archivos recientes compatibles y selección de una instalación descargada compatible cuando corresponde.
- **Instancias aisladas:** directorios separados, importación/exportación, progreso, streaming, hashes y caché de reintentos. Los contenidos mantienen las condiciones de sus autores y proveedores.
- **Cuenta y skins:** perfil Microsoft, skins y capas oficiales, biblioteca local de skins y vistas previas por nombre de usuario.
- **Estudio 3D:** poses, cámara, iluminación y PNG transparentes, construido con [skinview3d](https://github.com/bs-community/skinview3d) y [Three.js](https://threejs.org/), con sus avisos MIT conservados.
- **TP Client:** mod nativo opcional para **Fabric / Minecraft 1.21.1 / Java 21**, con instalación explícita en una versión descargada o modpack compatible. Su HUD oscuro con acentos neón ofrece veinte widgets, datos reales del cliente, categorías PvP/Exploración/Técnico, ajustes individuales y editor por arrastre. La distribución reserva espacio para chat, hotbar y notificaciones; si no hay espacio suficiente, informa qué widgets quedan ocultos. Este JAR no admite otras versiones ni loaders. [Guía del cliente](TP_CLIENT_README.md).
- **Menú principal dentro del juego:** menú oscuro original, vista 3D con la skin disponible de la sesión, fondo y logo locales personalizables y música procedural original. Conserva las acciones originales de Minecraft y sus restricciones de cuenta/demostración, con acceso al menú original. [Guía y procedencia de recursos](docs/TP_CLIENT_MENU.md).
- **Cosméticos TP locales:** editor de capas con pintura, deshacer/rehacer, importación/exportación PNG y vista 3D; renderizadores opcionales de capa, alas y corona originales en instancias compatibles de TP Client. Son independientes de las capas oficiales, no desbloquean derechos oficiales y no son visibles en clientes Vanilla sin modificar.
- **Centro de rendimiento:** vista previa y confirmación de presets reversibles de `options.txt`, recomendación prudente de RAM y clasificación local de síntomas de fallos. El diagnóstico de FPS puede usar muestras locales opcionales de TP Client tras un calentamiento dentro del mundo. Los presets excluyen formatos legacy no admitidos; no se afirma una mejora medida ni un benchmark A/B.
- **Biblioteca y clips:** capturas, mundos, carpetas de recursos/shaders y consola persistente. La grabación opcional de Windows captura los próximos 15/30/60/120 segundos de la ventana de Minecraft seguida por el launcher, usando FFmpeg/ffprobe instalados por separado. No graba audio ni tiene búfer de repetición de segundos anteriores. La captura real de una partida y su coste aún necesitan verificación manual.
- **Idiomas:** diccionarios locales para diez idiomas principales y variantes con cobertura indicada. MyMemory es opcional para resúmenes públicos; [detalles](docs/translations.md).

Clientes, bibliotecas, runtimes Java, loaders y contenidos se obtienen de sus proveedores al instalarlos. El repositorio y el instalador del launcher no incluyen clientes de Minecraft ni instaladores de OptiFine. Los archivos de terceros conservan sus condiciones.

## Estado de implementación

El trabajo actual está registrado en la [matriz de implementación](docs/TP_LAUNCHER_IMPLEMENTATION.md) y el [informe de verificación](TP_CLIENT_TEST_REPORT.md). Se probaron el menú principal nativo, el acceso a Opciones originales, el HUD con datos de una partida, el arrastre/restablecimiento del editor, los filtros de categorías y el acceso desde pausa en una sesión de demostración Fabric 1.21.1 aislada. El cliente final también mantuvo libre el pie del editor en una ventana pequeña. Esto no acredita compatibilidad con multijugador, shaders, todas las resoluciones o todas las versiones. Las cadenas nuevas también necesitan completar su localización.

La galería comunitaria, identidad TP, amistades, invitaciones y contratos de misiones tienen pruebas locales HTTP y SQL/RLS. **TP Cloud está apagado por defecto y no se ha desplegado.** Su interfaz de producción, productor confiable de eventos de juego, voz y transporte de salas remotas no están operativos; no se anuncia una red social ni un servicio P2P conectado. Consulta [staging de la nube](docs/cloud-staging.md). El HUD de escritorio anterior ya no es el catálogo principal de módulos; las fuentes conservadas de bridge/Companion son material de compatibilidad, no evidencia de que sus widgets anteriores funcionen en todas las versiones.

## Desarrollo

Se necesitan Node.js 22 o superior, Rust estable con MSVC, herramientas C++ de Visual Studio y Windows SDK. TP Client requiere **JDK 21**, seleccionado mediante `JAVA_HOME`; TP Companion permanece como proyecto de compatibilidad separado. El instalador incluye el instalador offline de WebView2.

```sh
npm ci
npm run tauri dev
```

```sh
npm run check
npm run check:ui
npm run build:client
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

Las comprobaciones independientes de la API se ejecutan con `npm ci` y `npm test` dentro de `tp-api/`. El informe de verificación registra resultados y límites; aprobar pruebas locales no equivale a un despliegue de producción ni a una firma confiable.

Los forks deben configurar su propio registro de Microsoft según la [guía de autenticación](docs/microsoft-client-configuration.md). Un Client ID es público y no demuestra aprobación del servicio.

CurseForge lee las credenciales en ejecución desde el entorno, `.env` de desarrollo o `integrations.env` local. Se excluyen de Git y del bundle y no se distribuye la clave del desarrollador. Los errores y restricciones de descarga se comunican; cuando el autor exige descarga manual se solicita el archivo exacto en su página oficial y se verifica tamaño y hash. [Instalación de contenido](docs/modpack-installation.md).

## Archivos del usuario

Configuración y biblioteca de cuentas permanecen en `%APPDATA%/LauncherTPS/`. Las instalaciones nuevas usan ese directorio para los datos de juego; se conserva `%APPDATA%/MinecraftTPS/` si ya existe. Ajustes permite seleccionar otra carpeta de juego.

| Ubicación | Contenido |
| --- | --- |
| Configuración: `launcher-settings.json` | Preferencias, idioma y ruta de datos |
| Configuración: `accounts.json` | Nombres, identificadores y selección; sin tokens en texto plano |
| Configuración: `accounts/*.dat` | Tokens de renovación Microsoft cifrados con DPAPI |
| Configuración: `minecraft-license-cache.dat` | Verificaciones de Minecraft Java cifradas con DPAPI |
| Configuración: `translation-cache.json` | Traducciones de descripciones públicas |
| Datos de juego: `account.dat` | Token de renovación activo cifrado con DPAPI |
| Datos de juego: `versions`, `libraries`, `assets`, `runtime` | Dependencias descargadas y Java |
| Datos de juego: `instances/...` | Mods, configuración, mundos y registros por instancia |
| Datos de juego: `skins_library` | PNG de skins e índice de biblioteca |
| Datos de juego: `cosmetics_library` | PNG de capas TP locales validados y metadatos |
| Datos de juego: `custom_poses.json`, `renders` | Poses propias y renders PNG |
| Datos de juego: `clips` | Clips MP4 grabados manualmente en local |
| Datos de juego: `playtime.json`, `logs` | Tiempo de juego y diagnósticos |
| Datos de juego: `cache/content`, `cache/modpacks` | Cachés verificadas de descargas e importación |
| Por instancia: `config/tp-client*.json`, `config/tp-client-menu/` | Ajustes del cliente nativo, cosméticos locales y menú principal |
| Por instancia: `config/tp-client/performance.json` | Muestra opcional de rendimiento local; sin subida remota |

Las credenciales, cuentas, cachés de licencias, mundos y configuración privada no deben añadirse al repositorio ni al instalador.

## Licencia, recursos y distribución

El código original se distribuye bajo **GPL-3.0-only**; [LICENSE](LICENSE). Esta licencia no concede permiso de marca para presentar una modificación como el producto oficial TP LAUNCHER. Dependencias, marcas, fuentes e imágenes aportadas por usuarios conservan sus licencias y créditos en los [avisos de terceros](static/legal/THIRD-PARTY-NOTICES.txt).

Las imágenes del menú, música y cosméticos TP incluidos en TP Client son recursos originales del proyecto. No se incluyen código, arte, JAR ni servicios de Millida. Los avisos de las nuevas dependencias del backend están en [tp-api/THIRD-PARTY-NOTICES.txt](tp-api/THIRD-PARTY-NOTICES.txt). Esto documenta procedencia y licencias; no certifica jurídicamente todas las distribuciones o importaciones posibles.

Algunos recursos visuales opcionales pueden leerse de un JAR instalado por el usuario. Esa lectura no concede permiso de redistribución ni convierte los recursos en arte original de TP. [Política de recursos](docs/asset-policy-audit.md).

La compilación de Windows **no tiene identidad de firma Authenticode configurada**. La compilación firmada requiere un certificado válido. La autorización del servicio y la firma del ejecutable son requisitos distintos; [distribución Windows](docs/distribution-windows.md). Las pruebas automatizadas no certifican todas las versiones ni todos los contenidos.

