# TP LAUNCHER: revisión de derechos, distribución e integraciones

Fecha: 25 de octubre de 2026. Revisión del código y recursos del workspace, con
consulta de fuentes primarias. No equivale a un dictamen jurídico ni garantiza
ausencia de reclamaciones. No se auditó el contenido de cada mod, skin o mundo de
terceros, todos los commits históricos ni la configuración efectiva de las cuentas
cloud. No se han aceptado contratos, solicitado permisos a titulares ni cambiado
los despliegues durante esta revisión.

## Resultado principal

El proyecto puede mantener un launcher original y herramientas compatibles con
Minecraft. No es necesario borrar la galería F2 ni prohibir todas las capturas.
La conclusión anterior de que cualquier imagen del juego está siempre prohibida
era demasiado amplia. También era incorrecto dar por hecho que todas las APIs
conceden o exigen, por sí solas, el uso de sus logos.

Hay asuntos pendientes antes de presentar la distribución pública como revisada:
registro OAuth propio, contrato de CurseForge, permisos de algunas marcas,
redistribución de mods, fuente correspondiente al binario y política de privacidad.
No se eliminaron nuevamente las cinco marcas de integración que el usuario pidió
conservar; esa decisión visual no constituye una licencia del titular.

## Minecraft: capturas, interfaz y archivos del juego

Las [Usage Guidelines](https://www.minecraft.net/en-us/usage-guidelines) permiten
crear, usar y compartir capturas de uno mismo jugando o usando Minecraft, dentro
de sus condiciones. Esto no concede derechos sobre una captura de otra persona,
su música, un pack o una skin ajena, ni convierte texturas extraídas del cliente en
recursos originales del launcher. Se exige evitar apariencia de producto oficial.

La [EULA](https://www.minecraft.net/en-us/eula) contempla herramientas y mods
originales para usuarios con licencia, distinguiéndolos del cliente modificado
completo, que no debe redistribuirse. Descargar el juego para un usuario y colocar
su JAR dentro del instalador público son operaciones distintas.

Comprobaciones del árbol de distribución:

- Los iconos laterales, los cuatro iconos de VERSIÓN y los accesorios de Poser usan
  dibujos o geometría propios; `GameIcon` ya no extrae texturas de los JAR.
- Logo de TP, iconos de aplicación y gráficos NSIS son originales. Los nombres
  internos antiguos MinecraftTPS/LauncherTPS se conservan para no romper rutas.
- Las skins predeterminadas distribuidas son TP Explorer y su variante espectral.
  Las skins de cuentas y aportadas por usuarios siguen siendo contenido ajeno.
- Monocraft lleva su licencia SIL OFL y procedencia en `static/fonts/SOURCES.md`;
  no se distribuye la fuente extraída de Minecraft.
- La galería lee PNG locales de las carpetas de capturas. No publica esos archivos
  ni los incorpora al código o al instalador. Leer esa ruta no es redistribuir el juego.
- El diseñador puede usar una captura personal elegida por el usuario o su última
  F2. Se guarda en `plugins/hud-preview.png` dentro de sus datos locales. La imagen
  aportada para esta tarea se restauró allí, sin añadirla a `static/` ni al bundle.
  Una distribución nueva conserva el paisaje SVG original como reserva.
- El pie muestra la no afiliación. Deben añadirse datos de contacto reales del
  responsable antes de publicar comercialmente; el nombre de un proyecto no
  verifica por sí solo identidad, contacto ni registro de marca.

Estos puntos reducen riesgos; no certifican todas las versiones del instalador.
El EXE distribuido anteriormente no cambia al editar los fuentes: hay que reconstruirlo.

## Logos de integraciones y comunidad

Un favicon público y una licencia de software no bastan para acreditar derechos
del dibujo o marca. El uso descriptivo puede tener excepciones legales según el
país, pero esta revisión no las convierte en permiso universal.

| Proyecto | Evidencia y decisión actual |
| --- | --- |
| Blockbench | Su [guía de logos](https://blockbench.net/wiki/blockbench/logos/) permite usarlos como crédito o enlace, sin aparentar asociación ni incorporarlos a la marca propia. **Restaurado** el icono oficial sin modificar, con atribución a JannisX11. |
| GitHub | Su [guía](https://brand.github.com/foundations/logo) contempla referencias/enlaces e integraciones; símbolo secundario sin sugerir respaldo. Conservado. |
| Discord | Revisada su [página de marca](https://discord.com/branding). Se identifica el servicio sin deformarlo ni presentarlo como patrocinador. Conservado como referencia, sujeto a sus condiciones. |
| Microsoft | Símbolo para inicio de sesión conforme a su [guía visual](https://learn.microsoft.com/en-us/entra/identity-platform/howto-add-branding-in-apps). Esto no autoriza un Client ID ajeno. |
| Quilt | Carpeta de marca con [CC0](https://github.com/QuiltMC/art/blob/master/brand/LICENSE). Conservado. |
| NeoForge | Logo del sitio del [equipo NeoForged](https://github.com/neoforged/websites), con [CC BY 4.0](https://github.com/neoforged/websites/blob/main/LICENSE.txt), atribución y copia de licencia. Conservado. |
| Fabric | PNG original restaurado. El [repositorio del sitio](https://github.com/FabricMC/fabricmc.net) distingue código MIT y contenido CC BY-NC-SA 4.0. La licencia Apache del instalador no prueba la del PNG del sitio. Revisar permiso comercial si se monetiza con anuncios. |
| Forge | Yunque oficial aislado en SVG cuadrado, sin el texto diminuto. Fuente en [MinecraftForge](https://files.minecraftforge.net/). No se encontró una licencia de marca general suficiente para certificar todo uso; conservar con carácter descriptivo no implica respaldo ni autorización comercial verificada. |
| OptiFine | Identificador oficial conservado. Su [copyright](https://optifine.net/copyright) restringe reproducción pública y redistribución del mod sin permiso escrito. No se incluye su JAR en el bundle. No debe afirmarse que cualquier instalación automatizada o uso de marca sea «100 % legal» solo por descargar desde su web. |
| Modrinth | Identificador conservado. Sus [términos](https://modrinth.com/legal/terms) y [COPYING](https://github.com/modrinth/code/blob/main/COPYING.md) reservan marcas y contemplan permiso escrito. No se encontró una regla general que convierta toda integración API en licencia automática del logo. Obtener confirmación aplicable al launcher público. |
| CurseForge | Identificador conservado. La autorización de API y la de marca deben verificarse conforme al acuerdo de la aplicación. Un HTTP 200 o un favicon disponible no acreditan ambas. |
| NameMC | [Términos del sitio principal](https://namemc.com/terms): derechos de gráficos reservados; no se encontró permiso general del logo. Se conservan nombre y enlace con símbolo propio. No se realiza scraping de NameMC. |
| Chunkbase | Revisadas sus [condiciones](https://www.chunkbase.com/terms-of-use); no se acreditó una licencia general del logo. Nombre y enlace conservados, símbolo propio. |
| MCStacker | Revisada la [web del autor](https://mcstacker.net/); no se encontró una autorización pública suficiente del logo. Se enlaza la herramienta; no se copia su editor. |
| Planet Minecraft | Revisados [términos](https://www.planetminecraft.com/terms_of_use) y [reglas](https://www.planetminecraft.com/rules). No se acreditó permiso general del logo; se mantiene enlace con símbolo propio. |
| Minecraft Wiki | La licencia del texto del [sitio](https://minecraft.wiki/w/Minecraft_Wiki_%28website%29) no resuelve automáticamente la del logo ni imágenes con licencias distintas. Pendiente comprobar el archivo concreto; enlace y símbolo propio. |
| SkinMC | La [web](https://skinmc.net/) impidió verificar sus términos durante la consulta. Permiso **no verificado**, no se afirma que el logo sea ilegal. Se mantiene referencia con símbolo propio. |

Procedencia, modificaciones y hashes: `static/logos/sources.json`. Resolución de
imágenes: `src/lib/brand_assets.ts`. Créditos visibles: `LegalNotice.svelte` y
`static/legal/THIRD-PARTY-NOTICES.txt`. No se envían solicitudes a titulares sin
autorización del usuario.

## Integraciones: hallazgos que no se solucionan cambiando iconos

### Autenticación Microsoft y modo sin conexión — prioridad alta

`src-tauri/src/lib.rs` utiliza el Client ID público `00000000402b5328`. No hay
evidencia de que pertenezca al desarrollador de TP. Que el flujo funcione no prueba
derecho a presentarse como esa aplicación. Preparar un registro autorizado propio
antes de distribuir. Microsoft explica la identidad de la aplicación en su
[documentación de registro](https://learn.microsoft.com/en-us/entra/identity-platform/quickstart-register-app).

La revisión del flujo actual también detecta ausencia de validación OAuth `state`
y PKCE. Son cuestiones de seguridad, separadas del copyright: deben corregirse con
el registro y flujo compatible, sin sustituir identificadores por valores inventados.
Los refresh tokens se guardan con DPAPI de Windows; no se empaqueta la cuenta del
usuario por guardar su sesión local. No se han incluido tokens en este informe.

El modo offline permite elegir un nick y lanzar sin una comprobación de titularidad
de licencia en ese camino. Puede tener uso legítimo sin conexión por un propietario;
no debe anunciarse como acceso premium gratis, evasión de compra ni acceso autenticado
a servidores con identidades ajenas. Revisar la política de acceso para publicación.
Consultar el perfil de Minecraft es una comprobación diferente a verificar todas
las modalidades y vigencias de propiedad/suscripción.

### CurseForge y Modrinth — prioridad alta

Los [términos de la API CurseForge](https://support.curseforge.com/support/solutions/articles/9000207405-curseforge-3rd-party-api-terms-and-conditions)
incluyen confidencialidad de clave, límites, restricciones de caché, competencia
y elusión. Debe confirmarse qué acuerdo/autorización cubre este launcher. La clave
local se lee sin expansión de `$`, no está en una constante ni en el frontend y
`.env` está ignorado. **No distribuir una misma clave privada en cada instalador**.
La clave compartida previamente debe rotarse desde el panel del titular antes de
una publicación; no se ha rotado ni mostrado en las salidas de esta revisión.

Las búsquedas conservan separación de proveedores. No debe ocultarse la procedencia,
ignorar archivos restringidos o convertir una descarga bloqueada en scraping de
otra fuente para eludirla. El launcher debe conservar los créditos, las condiciones
por proyecto y la identificación del cliente HTTP. Para Modrinth, revisar además
su [documentación de API](https://docs.modrinth.com/api/) y no confundir la licencia
de acceso al catálogo con licencia ilimitada sobre marcas o archivos de usuarios.

### ZIP de mods y packs — prioridad alta

`export_modpack_zip` en `src-tauri/src/lib.rs` copia los JAR activos a un ZIP. Eso
crea una copia redistribuible de todos esos mods. Que un archivo se pueda descargar
no implica que pueda volver a distribuirse, y OptiFine tiene restricciones expresas.
Antes de ofrecerlo públicamente como paquete compartible, verificar cada licencia
o exportar un manifiesto de referencias/descargas autorizadas y solo recursos con
permiso de redistribución. No se ha eliminado el exportador ni se han borrado mods.

### Código abierto y binarios — prioridad alta

El código propio declara GPL-3.0-only; existen LICENSE, lockfiles, scripts y avisos.
Distribuir un binario exige cumplir la [GPL](https://www.gnu.org/licenses/gpl.html),
incluida su fuente correspondiente y materiales necesarios para reconstruirlo.
En el workspace hay numerosos fuentes sin seguimiento: un enlace a GitHub no
demuestra que allí esté la revisión completa del EXE. Revisar el commit/tag de cada
release y las licencias de dependencias, mods, fuentes y arte por separado. No se
ha afirmado que la GPL otorgue derechos sobre recursos de terceros ni se ha publicado
el repositorio durante esta tarea.

## Privacidad, servicios y monetización

| Flujo observado | Destino / tratamiento |
| --- | --- |
| Login y perfil premium | Microsoft → Xbox Live/XSTS → Minecraft Services. Tokens privados en el dispositivo; los proveedores tienen sus propias políticas. |
| Nick offline y skin pública | Consultas a APIs de perfiles/skins, incluidas fuentes externas. El proveedor recibe IP de conexión y consulta de nick/UUID. No presentarlo como operación completamente offline. |
| Mods, shaders, recursos y Java | Proveedores y CDNs oficiales reciben las peticiones de búsqueda/descarga. Revisar licencias por archivo. |
| Directorio y estado de servidores | TP API y consultas de estado. Los nombres, MOTD, logos y muestras públicas de jugadores son contenido del servidor, no propiedad de TP. Documentar corrección/retirada, frecuencia y conservación. |
| TP API | Backend Vercel y Supabase. Código de `tp-api/lib/http.ts` deriva identificadores de límites mediante HMAC; no debe describirse como anonimato absoluto. La configuración de secretos es privada en el backend. |
| Capturas, mundos, HUD y TP Bridge | Archivos locales. El puente no hace peticiones de red ni lee cuentas; publica métricas para el PID del juego. No se suben capturas mediante estos comandos. |
| Comunidad y hosting | Enlaces externos; el sitio visitado aplica su política. La interfaz actual declara que no hay acuerdos de afiliación activos. |

La fuente SQL de Supabase habilita RLS y limita funciones/escrituras a roles
administrativos; la lectura pública del catálogo es deliberada. Su bridge verifica
token privado y restringe rutas. Esto es revisión del **código local**, no prueba de
que la base desplegada tenga exactamente esa configuración. Los controles deben
mantenerse conforme a la [guía de seguridad de Data API](https://supabase.com/docs/guides/api/securing-your-api).

Antes de abrir al público: publicar política de privacidad propia con responsable,
contacto, finalidad, destinos, conservación y ejercicio de derechos; revisar mercados
y edades a los que se dirige. Una futura publicidad o programa de afiliados exige
revisión adicional de contratos, identificación de patrocinios, rastreo/consentimiento
y condiciones de Mojang/proveedores. No se verificó un SDK publicitario activo.

También se encontró `csp: null` en la configuración Tauri. Endurecer la política
de contenido y revisar superficies remotas/IPC antes de distribución masiva. Es un
hallazgo técnico de seguridad, no una infracción de copyright demostrada.

## HUD y normas de servidores

El overlay propio y el puente de telemetría no contienen clases ni texturas de
Minecraft redistribuidas. Los símbolos necesarios se consultan durante el inicio
y se guardan localmente; no se incluye el índice completo en el instalador.
TP Bridge usa reflexión de lectura, sin transformadores, acceso a cuentas ni
adjuntarse a otros procesos Java. No evita las restricciones de depuración reducida.

Cada servidor puede prohibir overlays, minimapas o agentes Java mediante sus
reglas o anticheat. «Compatible» técnicamente no significa «permitido por todo
servidor»; no se garantiza ausencia de sanción y no se ha implementado evasión.

## Orden recomendado antes de una publicación general

1. Registro OAuth propio y endurecimiento del login; definir acceso offline con licencia.
2. Confirmación contractual de CurseForge y permisos de marca pendientes; rotar clave.
3. Exportación de packs respetando las licencias de cada archivo.
4. Fuente correspondiente al release, avisos y datos reales del responsable.
5. Privacidad, política de contenido Tauri y revisión de publicidad antes de monetizar.

El informe anterior de arte sigue en `docs/asset-policy-audit.md`; este documento
amplía su alcance y corrige la situación de Blockbench y de capturas personales.
