# Instalación de modpacks y diagnóstico

## Estado y progreso

Las operaciones de crear, importar, instalar un archivo concreto y reintentar publican `studio-progress`: identificador de operación, etapa, archivo actual, porcentaje, cantidad de archivos y bytes. La interfaz mantiene una notificación global y muestra el mismo avance dentro del diálogo del proyecto. Cambiar de pestaña no interrumpe la operación. `studio_current_operation` recupera su último estado dentro del proceso actual.

La notificación de éxito solo se emite después de validar los archivos y aplicar las carpetas de configuración. Siempre existe el aviso dentro del programa. El plugin oficial de notificaciones de Tauri permite además el aviso del sistema cuando hay permiso y Windows reconoce la aplicación instalada; su indisponibilidad no hace fallar una instalación.

Las instalaciones se serializan para evitar que dos instaladores Forge modifiquen simultáneamente las bibliotecas comunes. Los manifiestos se escriben mediante un archivo temporal sincronizado y reemplazo atómico. Los estados son `installing`, `incomplete` y `ready`. Los manifiestos anteriores sin ese campo se consideran completos por compatibilidad.

## Archivos grandes y reintentos

Los mods y recursos del manifiesto se descargan por streaming a un temporal, verificando tamaño, SHA-1 y, en `.mrpack`, SHA-512. Después se confirman en la caché y en la instancia. No se mantiene el JAR completo en memoria. Las descargas tienen hasta tres intentos; las consultas transitorias al proveedor también se reintentan, respetando `Retry-After` con una espera acotada.

`cache/content` contiene descargas verificadas; `cache/modpacks/<sha1>.zip` conserva el archivo original para reintentar. Un fallo no elimina la instancia ni los archivos descargados. El botón **Reintentar instalación** revisa los hashes antes de reutilizar la caché. Los metadatos de CurseForge se consultan por lotes de 50 archivos y se conserva una caché vinculada a los identificadores exactos de proyecto y archivo.

Límites: archivo ZIP o `.mrpack` de importación de hasta 512 MiB; archivo remoto individual del manifiesto de hasta 2 GiB; contenido total de hasta 16 GiB. Se rechazan rutas que salgan de la instancia, dispositivos reservados de Windows, enlaces simbólicos y reemplazos de metadatos del launcher. Las capas `client-overrides` se aplican después de `overrides`.

Los archivos CurseForge de recursos se guardan en `resourcepacks` y los shaders en `shaderpacks`, según el tipo del proyecto. Los JAR van a `mods`. Así los ZIP de recursos de un pack no se confunden con mods.

## Descargas que requieren la página del autor

Una respuesta sin `downloadUrl` no habilita construir enlaces CDN ni omitir el archivo obligatorio. La instancia muestra cada archivo exacto pendiente, su tamaño y su página oficial. El usuario puede descargarlos y pulsar **Importar archivos descargados**; el backend comprueba nombre, tamaño y SHA-1 antes de aceptarlos. Al importar el último archivo se reintenta automáticamente la instalación.

Caso verificado: RLCraft 2.9.3, archivo CurseForge `4612979`, requiere Minecraft 1.12.2 y Forge 14.23.5.2860. Se verificaron 185 de los 187 archivos del manifiesto y se aplicaron las configuraciones del autor. La API no ofrecía descarga externa para:

- [SRParasites-1.12.2v1.9.11.jar](https://www.curseforge.com/minecraft/mc-mods/scape-and-run-parasites/files/4436467).
- [RTG-1.12.2-6.1.0.0-snapshot.1.jar](https://www.curseforge.com/minecraft/mc-mods/realistic-terrain-generation/files/2782568).

El pack permanece pendiente hasta incorporar esas versiones verificadas. No se presenta como jugable ni se notifica un éxito incompleto.

## Arranque aislado

El proceso Java usa la carpeta de la instancia como directorio de trabajo, además de `--gameDir`. Esto mantiene `config`, `local`, registros y auxiliares de mods dentro del pack. Los archivos que versiones anteriores escribieron en la carpeta del proyecto no se borran.

En perfiles con BootstrapLauncher/ModLauncher, se añade el nombre real del JAR Vanilla heredado a `-DignoreList`. El nombre del perfil NeoForge difiere del nombre del JAR base: sin esa corrección SecureJarHandler puede detectar dos módulos Minecraft y fallar con `ResolutionException`. Los perfiles clásicos LaunchWrapper, Fabric y OptiFine conservan sus contratos.

La salida estándar y la de errores alimentan el diagnóstico reciente; los crash reports se incorporan tanto en cierres inmediatos como posteriores. Los registros persisten y el cierre de Minecraft no reinicia el launcher.

## Comprobaciones

`cargo test --lib` cubre integridad, reemplazos atómicos, importación de archivos manuales, capas del pack, rutas, transacciones de mods y argumentos de arranque. Las pruebas ignoradas de instalación real y arranque deben ejecutarse explícitamente: descargan contenido oficial y conservan sus instancias. Las de arranque usan una identidad offline de prueba, esperan inicialización de audio y recursos y cierran solo el proceso que iniciaron.

Se verificó el arranque real de Fabulously Optimized en Fabric/Minecraft 1.21.1 y de NeoForge 21.1.248 con FerriteCore. Esto comprueba esos casos; los conflictos entre mods añadidos por el usuario todavía requieren revisar el registro concreto.

`node scripts/check-ui-contracts.mjs` comprueba la estructura de Inicio y la cobertura de los catálogos de idiomas. No sustituye una revisión visual del WebView.
