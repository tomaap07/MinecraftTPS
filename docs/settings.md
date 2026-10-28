# Ajustes, cuentas y actualizaciones

## Pantalla del launcher

**Ajustes → Pantalla** controla la ventana de TP LAUNCHER y el tamaño de toda su
interfaz. La escala de 80–200 % se aplica con el zoom nativo de WebView, encima de
la escala de Windows, y amplía también botones, menús, iconos y visores. Incluye
presets, control gradual, botones de aumentar/reducir y restauración al 100 %.
La vista previa se guarda automáticamente; al salir del panel se termina de guardar
el último cambio pendiente, o se restaura la escala guardada si falla la persistencia.

El tamaño de ventana permite presets, dimensiones personalizadas, ajuste al área
útil del monitor y maximizado. Se respeta la barra de tareas y se limita la ventana
al espacio disponible para mantener los controles accesibles. Las preferencias
`ui_scale_percent`, `launcher_width`, `launcher_height` y `launcher_maximized` se
restauran al iniciar. Los archivos anteriores reciben 100 % y 1280 × 820 como valores
predeterminados. La resolución y pantalla completa de **Minecraft** conservan sus
propios campos y no cambian al ajustar la ventana del launcher.

## Persistencia y compatibilidad

`AppData/Roaming/LauncherTPS/launcher-settings.json` guarda los ajustes globales y `custom-background.png` conserva la imagen de fondo procesada. La imagen no necesita almacenarse en `localStorage`: las APIs `get_custom_background` y `set_custom_background` devuelven su URL de datos para visualizarla y escriben el archivo local de forma atómica.

El fondo admite PNG, JPEG y WebP de hasta 16 MiB. El decodificador limita dimensiones y memoria, escala proporcionalmente a un máximo de 2560 px por lado y guarda PNG. Una cadena vacía elimina únicamente el fondo creado por el launcher.

Los datos del juego conservan `AppData/Roaming/MinecraftTPS` si ya existe. Las instalaciones nuevas utilizan `AppData/Roaming/LauncherTPS`. Cambiar el nombre visible no mueve clientes, bibliotecas, mundos ni juegos abiertos. Una carpeta personalizada seleccionada en Ajustes se usa en los siguientes accesos; no se trasladan automáticamente los archivos antiguos. La respuesta de ajustes muestra la ruta real aunque el archivo aún conserve la selección predeterminada vacía.

## Gestor de cuentas

El primer inicio presenta Microsoft antes del idioma y Java. La conexión debe confirmar una licencia Java de Minecraft y obtener el perfil oficial. El modo de perfiles locales solo se habilita con una cuenta Microsoft guardada cuya credencial DPAPI pueda descifrarse y que tenga un comprobante cifrado de licencia vigente. Un perfil antiguo o metadatos de cuenta por sí solos no satisfacen el requisito.

La política del launcher revalida esa licencia cada 30 días. Al faltar evidencia apta, la interfaz muestra **«Para utilizar perfiles locales, debes iniciar sesión con una cuenta Microsoft al menos una vez en este equipo»**, y Rust bloquea creación, activación y lanzamiento locales. Después de verificar, se permite gestionar los perfiles locales dentro del armario. La opción no produce una sesión válida en servidores que necesitan autenticación Microsoft.

El bloque de compra enlaza a la tienda oficial de Minecraft en verde. Los errores de Microsoft, Xbox o Minecraft Services se propagan al panel: una respuesta incompleta o HTTP 403 no debe presentarse como «Listo» ni como una cuenta conectada.

La biblioteca estable se guarda en `AppData/Roaming/LauncherTPS/accounts.json` y sus credenciales en `accounts/<id>.dat`, fuera de la carpeta de juego configurable. Cada `.dat` es una copia de la credencial Microsoft que el sistema existente protege mediante DPAPI de Windows. La biblioteca JSON contiene solamente identidad, estado y avatar: las APIs de cuentas no devuelven refresh tokens.

Si había una biblioteca anterior en la carpeta de datos, se copian únicamente su JSON y las pequeñas credenciales cifradas. Los originales permanecen disponibles y no se mueven archivos del juego. Los archivos de credenciales tienen un límite de 64 KiB; la biblioteca admite hasta 128 cuentas y 1 MiB de metadatos.

Las operaciones de activación y eliminación se coordinan con el mismo bloqueo asíncrono de renovación de Microsoft. La activación conserva una instantánea de sesión, archivo activo y selección de cuenta; si la renovación o confirmación falla, se restauran esos tres elementos y se informa cualquier fallo de restauración. La renovación puede actualizar el token rotado dentro de su bóveda antes de confirmar la selección.

`deactivate_account` elimina el marcador activo al cerrar sesión, preservando otras cuentas guardadas. `restore_active_account_token` recupera la credencial cifrada de la cuenta elegida cuando no existe `account.dat`, por ejemplo tras cambiar la carpeta de juego. Un perfil offline activo no restaura una cuenta Microsoft.

## Java y Discord

El diagnóstico de Java acepta una ruta absoluta al ejecutable o a la carpeta del JDK. En Windows acepta rutas con espacios y comillas exteriores, y utiliza `java.exe` cuando se seleccionó `javaw.exe` con ese binario disponible al lado. Ejecuta solamente `-version`, oculta la consola auxiliar y limita el sondeo a cuatro segundos; los resultados se reutilizan mientras el ejecutable no cambie.

El campo `gc_mode` conserva `g1gc` de forma predeterminada al leer ajustes anteriores. Los únicos valores admitidos son `g1gc` y `zgc`; el lanzamiento valida por separado la compatibilidad de Java. La RAM global admite entre 2 y 128 GiB.

Discord Rich Presence está desactivado por defecto. Requiere un identificador de aplicación Discord válido proporcionado por el usuario, además de Discord de escritorio abierto. No se utiliza una aplicación ficticia ni una clave privada. El trabajador IPC se reconecta, actualiza cada quince segundos y elimina su actividad cuando se desactiva. Durante el juego muestra el perfil y la versión activos; el resto del tiempo muestra el estado del launcher.

## Actualizaciones

El comprobador consulta la última versión pública de [GitHub Releases del proyecto](https://github.com/tomaap07/MinecraftTPS/releases), compara versiones con SemVer y presenta el instalador compatible para Windows. Si no hay una publicación, lo indica sin simular una actualización.

La publicación 0.1.0 de la política Microsoft-first es una renumeración. Los usuarios de las compilaciones de desarrollo 0.1.1–0.1.3 deben instalarla manualmente, porque una comparación SemVer no ofrece una versión inferior como actualización. La retirada de publicaciones obsoletas no altera el historial de Git.

La API de descarga obtiene nuevamente los metadatos oficiales y exige un instalador `.exe`/`.msi` con `digest` SHA-256 publicado por GitHub, URL HTTPS del repositorio y tamaño de hasta 512 MiB. Las redirecciones se restringen a los servidores oficiales de archivos de GitHub. Se escribe un archivo temporal, se comprueban tamaño y hash y solamente entonces se confirma en `LauncherTPS/updates`.

La instalación requiere una acción explícita en la interfaz. Antes de abrir el instalador, el backend vuelve a consultar la publicación y comprobar el hash del archivo local. Se bloquea la instalación si hay una sesión Minecraft abierta. Consultar o descargar una actualización nunca ejecuta el instalador.

Este mecanismo verifica integridad mediante el digest de GitHub; no pretende ser una firma digital del editor ni inventa claves del plugin Tauri Updater. Una publicación antigua sin digest no se ofrece como descarga verificada y conserva el acceso a su página de GitHub. Referencia: [API oficial de assets de GitHub Releases](https://docs.github.com/en/rest/releases/assets#get-a-release-asset).

## Comprobaciones

Las pruebas unitarias cubren compatibilidad de ajustes anteriores, validación de cuentas, archivos atómicos, fondos redimensionados, identificadores de Discord y rutas de actualizaciones. En Windows, otra prueba cubre rutas Java con espacios y selección `javaw.exe` → `java.exe` sin ejecutar un binario simulado.

La prueba ignorada `launcher_settings::tests::installed_java_8_reports_its_version` ejecuta `-version` sobre el Java 8 Portable local ya instalado; debe seleccionarse expresamente al verificar ese equipo. Las pruebas del actualizador no ejecutan instaladores.
