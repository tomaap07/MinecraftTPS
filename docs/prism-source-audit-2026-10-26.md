# Prism Launcher: investigación de código, recursos y distribución

Revisión de fuentes primarias: 26 de octubre de 2026. Repositorio público analizado: `PrismLauncher/PrismLauncher`, commit **`390a240abaefe389d1ab8510cc0ba5c2692a65be`**, fechado el 25 de octubre de 2026. La copia de consulta está fuera del producto, en `.codex-tmp/prism-source-audit`, una ruta ignorada por Git. No se han incorporado código, iconos ni binarios de Prism a TP LAUNCHER.

Esta investigación distingue hechos observables de conclusiones jurídicas. La existencia de una función en Prism no prueba que otro producto tenga los mismos permisos, contratos o circunstancias; tampoco permite calcular un porcentaje fiable de riesgo legal.

## 1. Código abierto, marca e ilustraciones son permisos diferentes

Prism declara GPL versión 3 para su programa y conserva avisos de componentes e incorporaciones anteriores. Su marca tiene otro documento: `program_info/LICENSE` limita expresamente CC BY-SA 4.0 a los logos y branding de esa carpeta. No extiende esa licencia a Minecraft ni a todos los recursos de terceros. Fuentes: [COPYING.md, líneas 1–33](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/COPYING.md#L1-L33), [licencia de branding, líneas 1–3](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/program_info/LICENSE#L1-L3).

Si se adaptase código cubierto de Prism, habría que cumplir sus condiciones y conservar los avisos pertinentes. TP ya declara `GPL-3.0-only` para su propio código; eso no concede derechos sobre imágenes ajenas. Al entregar ejecutables GPL, debe facilitarse el código fuente correspondiente mediante una modalidad admitida por la licencia. Un repositorio privado no es, por sí solo, incumplimiento: importa el acceso de los destinatarios al código correspondiente. Véase [GPL, sección 6](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/LICENSE#L247-L294).

## 2. Prism no extrae todos sus iconos del JAR

Su archivo de recursos registra varias imágenes `minecraft.png` dentro del propio programa. También incluye SVG de bloques, objetos y personajes; un comentario atribuye los iconos planos de instancia a Santiago Cézar bajo CC BY-SA 4.0. Algunos SVG contienen, además, metadatos de autores y licencia. El archivo QRC forma parte de la compilación. Evidencias: [PNG incluidos, líneas 46–50](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/resources/multimc/multimc.qrc#L46-L50), [atribución y SVG, líneas 248–269](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/resources/multimc/multimc.qrc#L248-L269), [metadatos de grass.svg](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/resources/multimc/scalable/instances/grass.svg#L61-L64), [registro en CMake](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/CMakeLists.txt#L1195).

Por tanto, **un dibujo propio que representa un bloque, una textura oficial extraída y un logo son casos distintos**. La atribución de un SVG no certifica derechos sobre el archivo oficial de Mojang que pudiera representar. No se encontró en los documentos revisados una autorización general de Mojang que TP pueda heredar por imitar a Prism.

Para TP, leer una textura de una instalación local evita incluir esa copia en Git o en el instalador. Es una diferencia técnica relevante, pero no transforma el recurso en dominio público ni concede automáticamente permiso para usarlo como identidad del launcher. Mantener la marca TP original y separar el caché local de los recursos distribuidos sigue siendo la opción prudente.

## 3. Descargas de Minecraft y bibliotecas

Prism usa su propio servicio de metadatos, configurable, además de los proveedores del juego. No es exacto describirlo como un programa que solo consulta un manifiesto de Mojang: [CMakeLists, línea 205](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/CMakeLists.txt#L205), [documentación de APIs](https://prismlauncher.org/wiki/help-pages/apis/).

Los assets se almacenan por hash y se verifican durante la descarga; sus rutas derivan del índice. Las bibliotecas usan las URL y sumas proporcionadas en sus metadatos. También separa los clasificadores nativos del artefacto Java y omite un artefacto inexistente: [AssetsUtils, líneas 269–296](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/minecraft/AssetsUtils.cpp#L269-L296), [Library.cpp, líneas 163–198](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/minecraft/Library.cpp#L163-L198).

Estos mecanismos explican instalación y compatibilidad; no demuestran que las texturas descargadas alimenten los iconos de su interfaz. TP debe conservar su manifiesto actual de `piston-meta.mojang.com`, comprobar integridad y evitar redistribuir clientes o instalaciones modificadas completas.

## 4. Microsoft y perfiles locales: diferencias reales

Prism tiene un Client ID configurable, y su configuración pide cambiar las claves al cambiar la identidad del producto. Utiliza endpoints de cuentas personales de Microsoft y permisos Xbox; su wiki permite configurar un ID propio. Evidencias: [CMakeLists, líneas 265–277](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/CMakeLists.txt#L265-L277), [MSAStep, líneas 135–138](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/minecraft/auth/steps/MSAStep.cpp#L135-L138), [wiki APIs](https://prismlauncher.org/wiki/help-pages/apis/).

No hay razón para sustituir el ID propio ya corregido de TP (`6a4f8732-6bf7-49c3-a749-16607f97281d`) por el de Prism o el de un cliente oficial. Un Client ID identifica una aplicación pública; no equivale a una contraseña ni garantiza acceso a todos los servicios posteriores.

Prism exige que exista una cuenta que conste como propietaria de Minecraft tanto al añadir perfiles offline como al iniciar. Esa comprobación consulta los datos de titularidad guardados; estos archivos no documentan una regla universal de renovación cada 30 días. Fuentes: [AccountListPage, líneas 141–156](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/ui/pages/global/AccountListPage.cpp#L141-L156), [LaunchController, líneas 96–114](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/LaunchController.cpp#L96-L114), [AccountList, líneas 572–580](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/minecraft/auth/AccountList.cpp#L572-L580).

TP permite ahora perfiles locales de pruebas con nick por solicitud expresa del usuario. Es una política diferente, no una reproducción de la de Prism. Ese perfil no acredita una compra, una sesión premium ni autorización para servidores autenticados; no debe anunciarse como sustituto gratuito de una licencia. Esta investigación no modifica esa función.

## 5. OptiFine, mods y exportación de packs

La guía oficial de Prism dirige al usuario a descargar OptiFine desde su web e importar el JAR; distingue instalación con Forge, Fabric/OptiFabric y variantes independientes. No promete compatibilidad uniforme: [guía de instalación](https://prismlauncher.org/wiki/getting-started/installing-optifine/). El [copyright de OptiFine](https://optifine.net/copyright) restringe su distribución pública sin permiso escrito. Mantener la importación local y no alojar sus binarios evita confundir descarga personal con redistribución.

Para `.mrpack`, Prism identifica archivos mediante hashes, escribe sus URL en el índice y excluye del ZIP los archivos resueltos; otros archivos pueden quedar en `overrides`. Su exportación CurseForge genera referencias de proyecto y archivo más una carpeta de overrides. Véanse [ModrinthPackExportTask, líneas 199–206](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/modplatform/modrinth/ModrinthPackExportTask.cpp#L199-L206), [URL y hashes, líneas 302–311](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/modplatform/modrinth/ModrinthPackExportTask.cpp#L302-L311), [FlamePackExportTask, líneas 337–340](https://github.com/PrismLauncher/PrismLauncher/blob/390a240abaefe389d1ab8510cc0ba5c2692a65be/launcher/modplatform/flame/FlamePackExportTask.cpp#L337-L340).

Tener un exportador no licencia todos sus archivos. Para compartir packs de TP conviene priorizar referencias oficiales y revisar los archivos incluidos directamente, especialmente OptiFine, recursos del juego y mods con redistribución restringida.

## 6. Condiciones oficiales y logos de comunidad

La [EULA de Minecraft](https://www.minecraft.net/en-us/eula) contempla herramientas y mods originales, reserva derechos sobre contenido del juego y distingue esos mods de redistribuir el cliente modificado. La compra y los permisos aplicables siguen siendo relevantes; GPL no sustituye esas condiciones.

Las [Usage Guidelines](https://www.minecraft.net/en-us/usage-guidelines) permiten capturas de la propia partida y exigen evitar apariencia de producto oficial. Distinguen referencias descriptivas del uso de recursos como marca. Una galería F2 local y una captura propia para previsualizar el HUD tienen una base distinta de empaquetar texturas oficiales. Hay que verificar la autoría de una captura ajena antes de distribuirla, y mantener responsable, contacto y aviso de no afiliación. Extraer un archivo del JAR no es una autorización general de marca.

Sobre los proveedores, no se confirmó la afirmación anterior de que todas sus APIs exigen sus logos o permiten cualquier uso:

- [Modrinth: términos, sección Trademarks](https://modrinth.com/legal/terms) y [COPYING del repositorio oficial](https://github.com/modrinth/code/blob/main/COPYING.md) reservan su marca y requieren permiso escrito. El permiso de usar la API y el del logo no son equivalentes.
- [CurseForge: términos de API de terceros](https://support.curseforge.com/support/solutions/articles/9000207405-curseforge-3rd-party-api-terms-and-conditions) regulan acceso, claves y propiedad intelectual; no proporcionan una autorización general implícita de marcas. No debe publicarse una clave privada ni utilizar la de Prism por comodidad.
- Para Fabric, Forge, OptiFine y los enlaces de comunidad, esta investigación de Prism no sustituye una revisión del titular y archivo concreto. Conservar un nombre descriptivo o enlazar su sitio tampoco acredita permiso del dibujo de su logo.

La recomendación para TP es conservar un inventario por recurso con titular, procedencia, permiso, atribución y alcance; usar dibujos propios cuando no se haya establecido una base suficiente y no presentar ninguna integración como patrocinio. No se borran ni restauran logos desde esta auditoría.

## 7. Correcciones explícitas al prompt recibido

Estas precisiones corrigen conceptos del texto de referencia; no atribuyen conductas a otros launchers:

- **`online-mode` es una propiedad del servidor**, no un permiso legal ni un modo que un launcher pueda cambiar en un servidor ajeno. Controla la comprobación de cuentas de quienes se conectan. Un nick local no se convierte por ello en una identidad autenticada. Fuente técnica primaria: [documentación de server.properties de PaperMC](https://docs.papermc.io/paper/reference/server-properties/#online-mode).
- **Poder abrir un JAR, leer sus recursos o descompilar clases no convierte Minecraft en código abierto.** JAR es un formato de archivo basado en ZIP; las autorizaciones dependen de la licencia, no de que sus bytes sean accesibles. La definición de código abierto incluye derechos de redistribución y modificación, además de disponibilidad del código. Fuentes: [Oracle: fundamentos de JAR](https://docs.oracle.com/javase/tutorial/deployment/jar/basicsindex.html), [Open Source Initiative: definición](https://opensource.org/osd).
- **`https://mojang.com` no es el manifiesto de versiones.** TP utiliza [version_manifest_v2.json de piston-meta](https://piston-meta.mojang.com/mc/game/version_manifest_v2.json). Se consultó este endpoint durante la revisión y respondió con un objeto JSON que contiene `latest` y `versions`; sustituirlo por la página genérica rompería esa lectura. La fuente técnica de TP está en `src-tauri/src/lib.rs`, función `load_version_manifest`.
- **Crafatar es un servicio comunitario independiente, no la API oficial de Mojang ni un fallback oficial que otorgue nuevos derechos.** Su [repositorio de los mantenedores](https://github.com/crafatar/crafatar) explica su función de servir avatares para aplicaciones externas y licencia su código bajo MIT. Esa licencia de software no constituye una licencia universal de las skins o texturas que entrega. Consumirlo cambia el proveedor técnico, no la titularidad de la imagen; las condiciones de Mojang y del autor de la skin siguen siendo cuestiones separadas.
- **La extracción local no es un blindaje legal.** Evita empaquetar una copia de la textura y puede reducir el alcance de la distribución, pero no concede dominio público, afiliación oficial ni autorización general para convertirla en logo o marca. Debe evaluarse el uso concreto con la [EULA](https://www.minecraft.net/en-us/eula) y las [Usage Guidelines](https://www.minecraft.net/en-us/usage-guidelines), como se detalla arriba.

## Resultado de esta revisión

Prism aporta patrones técnicos útiles y evidencia verificable de cómo separa recursos, identidad, cuentas y catálogos. No ofrece un certificado legal transferible a TP. Antes de distribuir otra versión conviene revisar los archivos que realmente entran al instalador y los packs exportados, el acceso al código GPL correspondiente y los permisos específicos de marca. Las políticas enlazadas pueden cambiar; estas conclusiones corresponden a la fecha indicada.
