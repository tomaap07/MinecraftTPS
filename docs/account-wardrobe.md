# Cuenta y armario de skins

El apartado **Cuenta** reúne el perfil oficial, el visor 3D, la biblioteca local y las capas que pertenecen a la cuenta Microsoft conectada.

## Añadir y conservar skins

- Arrastra un PNG a la zona de importación o pulsa **Subir archivo PNG**. Las dimensiones admitidas son 64 × 64 y 64 × 32, con un máximo de 2 MB.
- Selecciona **Clásico (4 px)** o **Slim (3 px)** antes de importar el PNG. Este dato se conserva con la skin.
- Para importar una skin pública, escribe un nick premium y pulsa **Importar**. Rust resuelve primero la identidad de Mojang y consulta su textura pública. Si falla la descarga de la textura, prueba MCHeads y Minotar usando el UUID ya resuelto.
- **Guardar skin actual en el armario** conserva el diseño del perfil activo; en modo offline guarda la textura que se muestra para el nick o el personaje predeterminado.

La colección se guarda en `%APPDATA%\MinecraftTPS\skins_library\`. Cada PNG tiene un UUID como nombre de archivo. `skins.json` contiene el nombre visible, el modelo y la fecha; el índice no contiene credenciales. Los cambios del índice se escriben primero a un archivo temporal y se reemplazan al terminar.

## Previsualizar y sincronizar

Selecciona una tarjeta para ver el personaje completo en el visor transparente. La cuadrícula utiliza miniaturas de canvas 2D para evitar crear un contexto WebGL por skin.

Con una cuenta Microsoft conectada, **Usar y sincronizar** renueva la sesión si hace falta y sube el PNG a Minecraft Services. El perfil devuelto por Minecraft actualiza el visor. El launcher no necesita pedir tokens manualmente.

En modo offline, **Ver skin** cambia la vista previa local. Para que una skin aparezca en servidores oficiales debe aplicarse a una cuenta Minecraft autenticada. Renombrar y eliminar solo modifican la biblioteca local.

## Capas y élitros

Las tarjetas muestran únicamente las capas devueltas por el perfil oficial. Cada tarjeta permite alternar entre el diseño de capa y el recorte de élitros; **Equipar capa** y **Ocultar capa** actualizan la cuenta autenticada.

Los recortes se dibujan sin interpolación en canvas: capa `(1,1,10,16)` y élitros `(36,2,10,20)`, multiplicados por `ancho_original / 64`. Las vistas no muestran el atlas de textura completo.

## Integración de componentes

`AccountWardrobe.svelte` recibe los datos del perfil y callbacks para iniciar/cerrar sesión, actualizar el perfil y abrir Poser. El callback opcional `onpreview(url, model)` comparte una selección offline con las vistas previas; `model` es `default` o `slim`.

Poser mantiene sus poses personalizadas en `custom_poses.json`, independientes del personaje ambiental de Jugar, cuya pose se elige al iniciar el launcher. El editor muestra créditos y enlaces directos a [skinview3d](https://bs-community.github.io/skinview3d/) y [Three.js](https://threejs.org/), los proyectos que utiliza para el visor 3D. skinview3d acredita a Kent Rasmussen, Haowei Wen, Sean Boult y colaboradores bajo [licencia MIT](https://github.com/bs-community/skinview3d/blob/master/LICENSE); Three.js también utiliza [licencia MIT](https://github.com/mrdoob/three.js/blob/dev/LICENSE).

Los comandos Tauri se encuentran en `src-tauri/src/skin_library.rs`: `list_saved_skins`, `save_library_skin`, `save_current_skin`, `import_library_skin`, `rename_library_skin`, `delete_library_skin` y `apply_library_skin`.

Los PNG se validan antes de guardarlos y los archivos dañados aparecen como entradas reparables o eliminables. Las descargas remotas aceptan únicamente los hosts de texturas de Minecraft, MCHeads y Minotar mediante HTTPS.
