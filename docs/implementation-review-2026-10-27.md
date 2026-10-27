# TP LAUNCHER — revisión de implementación y atribuciones

Fecha: 27 de octubre de 2026. Alcance: cambios de interfaz, directorio y textos informativos de esta tarea, más lectura de los puntos de integración indicados abajo.

Este documento distingue una implementación comprobable de una autorización jurídica. No certifica que toda la aplicación esté libre de reclamaciones, no calcula una probabilidad de demanda y no sustituye la verificación de permisos, contratos y normativa aplicables a una distribución concreta. La mera inclusión de créditos no subsana un uso que requiera autorización.

## Implementación revisada

### Términos y contacto

- `src/lib/TermsConditions.svelte` incluye seis secciones: uso, cuenta Microsoft/perfiles locales, privacidad, créditos, financiación y contacto.
- `src/routes/+page.svelte` importa y muestra el componente en la pestaña `legal`. La entrada lateral está debajo de Servidores, dentro de TU AVENTURA, con un símbolo vectorial propio.
- `LegalNotice` se encuentra después del bloque de vistas y recibe `onterms={() => activeTab="legal"}`. La llamada ya no excluye Ajustes. Cambiar `activeTab` reinicia el desplazamiento del panel de contenido.
- El aviso indica que TP LAUNCHER es un proyecto independiente de TP Studios, sin aprobación ni asociación con Mojang, Microsoft o las comunidades enlazadas.
- Contactos publicados por solicitud del propietario del proyecto: TP Studios; TP LABS en [Instagram](https://www.instagram.com/tp_labs07/); [tomaap07 en GitHub](https://github.com/tomaap07); `tomignacio60@gmail.com`, con apertura del cliente de correo y botón para copiar; `xZaneXad / Fariiuh`, Lead Dev · TP Studios. No se presenta TP Studios como una sociedad inscrita ni se inventa un domicilio o registro mercantil.

Los enlaces a servicios externos usan el plugin de apertura del sistema y muestran un mensaje si falla la apertura. Los iconos de contacto de la página son SVG simples creados para esta interfaz; no se anuncian como material de marca licenciado por Instagram o GitHub.

### Comunidad: 14 herramientas

Se conservan las nueve entradas existentes y se agregan cinco. El contador se deriva del mismo array que utiliza el buscador; no existe un número fijo independiente del contenido.

| Herramienta | Destino |
| --- | --- |
| NameMC | https://namemc.com/minecraft-skins |
| SkinMC | https://skinmc.net/ |
| Blockbench | https://www.blockbench.net/ |
| MCStacker | https://mcstacker.net/ |
| Chunkbase | https://www.chunkbase.com/apps/seed-map |
| Planet Minecraft | https://www.planetminecraft.com/ |
| Minecraft Wiki | https://minecraft.wiki/ |
| Modrinth | https://modrinth.com/ |
| CurseForge | https://www.curseforge.com/minecraft |
| Misode | https://misode.github.io/ |
| Minecraft Tools | https://minecraft.tools/ |
| GamerGeeks | https://www.gamergeeks.net/apps/minecraft |
| Modpack Index | https://www.modpackindex.com/ |
| McSrvStat | https://mcsrvstat.us/ |

Los cinco destinos nuevos se abrieron en sus sitios oficiales durante esta tarea y sus descripciones se contrastaron con sus funciones. Utilizan símbolos vectoriales de la interfaz, sin copiar sus logos. Los nueve servicios previos conservan `CommunityIcon`: este puede consultar los favicons de Google S2 y dispone de ilustraciones propias de respaldo. Recibir un favicon remoto no concede automáticamente derechos sobre la marca o la imagen; la licencia de cada uso debe evaluarse por separado. La página de privacidad informa sobre estas conexiones.

### Hosting y patrocinio

- `HostingOptions` se reutiliza en Comunidad y Servidores. Consulta `src/lib/data/hosting_providers.json` en lugar de una lista duplicada dentro del componente.
- Los tres destinos actuales son las páginas oficiales de BisectHosting, Apex Hosting y HolyHosting. Se comprobaron sus destinos. No se fijan precios, disponibilidad geográfica, descuentos ni promesas de rendimiento.
- Los tres registros tienen `affiliateActive: false` y ningún enlace de afiliado. En ese estado se muestran como enlaces informativos; no se inventa una colaboración o comisión activa.
- La estructura permite definir un enlace de afiliado explícito HTTPS y marcarlo como tal. Eso es soporte técnico de configuración, no prueba de un contrato. Antes de activarlo se necesitan el acuerdo real, las condiciones y las divulgaciones correspondientes.
- `community_campaigns.json` está vacío en el estado revisado. `CommunitySpotlight` solo representa campañas activas configuradas y las identifica como contenido patrocinado. Esta tarea no contrató publicidad ni envió datos a redes publicitarias nuevas.
- Los avisos de financiación no otorgan permiso de monetización sobre gráficos o marcas de terceros. Los términos del proveedor, la normativa de publicidad y las pautas de Minecraft deben seguir revisándose para cada campaña.

### Directorio TP API

- Se retiró el selector «Selección TP» de `ServerDirectory.svelte`.
- Tanto el componente como `src-tauri/src/server_directory.rs` usan solamente TP API para el catálogo. El parámetro IPC opcional `local` se conserva para compatibilidad con frontends anteriores, pero no activa una fuente alternativa.
- Se eliminó el fallback que consultaba mcsrvstat.us y la lista incorporada como sustitutos de TP API. McSrvStat sigue disponible como enlace externo independiente en Comunidad.
- Si TP API falla, Rust solo admite una página guardada que se identifique como `TP API`, cuyos registros sean de nube e IDs `tp:`. Marca esa página como antigua, desactiva destacados y conserva el mensaje de error. Sin una página válida devuelve el error y el usuario puede reintentar.
- La caché no convierte un dato anterior en un dato en vivo. La interfaz conserva las fechas de consulta y diferencia una medición TCP desde el equipo del usuario de las cifras consultadas en la nube.
- «Consulta gratuita» significa consulta sin cobro en esta integración. No significa disponibilidad ilimitada, una licencia para reutilizar todas las imágenes o acceso a todos los servidores existentes. El servicio mantiene sus límites.
- No se modificó la lista personal `servers.dat`. Añadir un servidor a Mis servidores sigue dependiendo de una acción expresa del usuario.
- No se desplegó TP API ni se cambiaron Supabase, Vercel o su configuración remota en este trabajo.

## Fuentes jurídicas y alcance de las atribuciones

### Minecraft y recursos locales

La [EULA oficial](https://www.minecraft.net/en-us/eula) y las [pautas de uso](https://www.minecraft.net/en-us/usage-guidelines) son las fuentes aplicables consultadas. Se mantiene una marca propia de TP y un aviso de independencia. Los términos explican que el launcher no concede una licencia de Minecraft.

Extraer una textura del JAR que ya está en el equipo no cambia su titularidad. Tampoco convierte automáticamente en autorizados todos los usos públicos, comerciales o publicitarios. La galería consulta capturas locales; visualizar o enlazar una carpeta no equivale a redistribuir el juego. Compartir una captura o utilizarla en una campaña sigue requiriendo atender a su contenido y condiciones. No se afirma que todas las capturas estén prohibidas ni que todas estén libres de derechos.

El perfil con nick permite pruebas locales, pero no demuestra una compra o una identidad premium y no autentica frente a servidores que exigen cuenta oficial. La obtención de una skin pública tampoco acredita propiedad de esa cuenta.

### NeoForge: fuente exacta de arte

- Archivo utilizado: `static/logos/neoforge.png`.
- Fuente registrada: [neoforged.net/img/authors/neoforged.png](https://neoforged.net/img/authors/neoforged.png).
- Correspondencia del repositorio: [neoforged/websites/static/img/authors/neoforged.png](https://raw.githubusercontent.com/neoforged/websites/main/static/img/authors/neoforged.png).
- El [README oficial](https://github.com/neoforged/websites/blob/main/README.md) declara el contenido general del repositorio bajo CC BY 4.0, con una excepción específica para `themes/mainroad`, que no corresponde a este archivo.
- La [licencia completa](https://github.com/neoforged/websites/blob/main/LICENSE.txt) requiere atribución y contempla la indicación de cambios. El manifiesto de fuentes registra este archivo como no modificado; los términos atribuyen el material a NeoForged y colaboradores y enlazan su licencia.
- CC BY 4.0 excluye derechos de patente y de marca en la sección 2(b)(2) y no autoriza a insinuar aprobación. Esta atribución se refiere a derechos de autor del archivo; no certifica cualquier uso de la marca NeoForge.

### Quilt: fuente exacta de arte

- Archivo utilizado: `static/logos/quilt.svg`.
- Fuente registrada: [QuiltMC/art/brand/svg/quilt_mini_icon_transparent.svg](https://raw.githubusercontent.com/QuiltMC/art/master/brand/svg/quilt_mini_icon_transparent.svg), sin modificar según el manifiesto local.
- El [README oficial](https://github.com/QuiltMC/art/blob/master/README.md) diferencia licencias por carpetas y atribuye la marca principalmente a [Prospector](https://github.com/Prospector/).
- Se consultó específicamente [brand/LICENSE](https://github.com/QuiltMC/art/blob/master/brand/LICENSE), que contiene CC0 1.0. No se extendió la licencia de esta carpeta a todas las comisiones o ilustraciones del repositorio.
- Su cláusula 4(a) excluye derechos de marca y patentes. Los términos de TP distinguen esa exclusión de la situación de derechos de autor del icono.

### Otros proyectos

| Proyecto o material | Situación documentada en esta tarea |
| --- | --- |
| Forge, Fabric y sus marcas | Identificación del proyecto y enlaces. No se afirma que la licencia del cargador sea una licencia general de su logo. La existencia de un uso similar en otro launcher no es una autorización por sí sola. |
| OptiFine | La [página oficial de copyright](https://optifine.net/copyright) restringe la redistribución pública. La interfaz actual dirige a su sitio y permite detectar/importar el JAR local. La política de exportación revisada identifica OptiFine por nombre y contenido de clases y excluye el binario. No se acredita aquí un permiso general de su logo. |
| Modrinth | Sus [términos](https://modrinth.com/legal/terms) reservan las marcas y exigen permiso previo por escrito. El uso de su API no prueba dicho permiso. Los proyectos descargados tienen licencias separadas. |
| CurseForge | El [acuerdo oficial de API](https://support.curseforge.com/support/solutions/articles/9000207405-curseforge-3rd-party-api-terms-and-conditions) regula la API y la confidencialidad de claves. No se certifica una licencia general de todos sus recursos por disponer de una clave. |
| Blockbench | Su [guía oficial de logos](https://blockbench.net/wiki/blockbench/logos/) permite crédito y enlaces, con límites contra la confusión de asociación y contra incorporarlo a la marca propia. Se enlaza y atribuye a sus creadores. |
| NameMC, SkinMC, MCStacker, Chunkbase, Planet Minecraft, Minecraft Wiki y demás herramientas | Enlaces descriptivos a sitios independientes. Las imágenes, skins y marcas conservan derechos de sus titulares. No se verificó una autorización individual general para todas sus marcas. |
| Monocraft | Atribución a Idrees Hassan y colaboradores, SIL OFL 1.1; [repositorio oficial](https://github.com/IdreesInc/Monocraft). |
| skinview3d y Three.js | Atribución y MIT según [skinview3d](https://github.com/bs-community/skinview3d) y [Three.js](https://github.com/mrdoob/three.js/blob/dev/LICENSE). Usar la biblioteca no implica una relación con otras aplicaciones de poses. |

El archivo local `static/logos/sources.json` conserva origen, hashes y transformaciones de los recursos registrados. Un hash comprueba identidad del archivo, no su autorización. La página puede abrir `static/legal/THIRD-PARTY-NOTICES.txt`. El código propio conserva GPL-3.0-only según `package.json` y el archivo LICENSE; esa licencia no relicencia contenidos de terceros.

Se compararon además los bytes locales con las fuentes primarias de los dos recursos nuevos, sin modificarlos:

| Archivo | SHA-256 de la fuente primaria, coincidente con el local |
| --- | --- |
| NeoForge | `884d0637dda196443b5b277fc1bb607678f813923fb703273ed05aab89076226` |
| Quilt | `406dd9dd5c6c378c8af93e15694d820ec45568f0521fcfa3d654db2ff618af1a` |

## Privacidad contrastada con la implementación

- OAuth utiliza el Client ID propio configurado y un navegador del sistema, redirección local, estado y PKCE; el launcher no solicita la contraseña en su formulario.
- La persistencia de tokens en Windows emplea DPAPI mediante `CryptProtectData` para el usuario del sistema. Esto no evita que alguien con acceso a esa sesión de Windows pueda utilizarla.
- Se describen datos locales bajo AppData, registros, cachés, poses y capturas, sin prometer plazos de borrado automáticos no implementados.
- Se informa de consultas a Microsoft/Xbox/Minecraft, proveedores de skins y catálogos; sus servicios pueden observar la IP y los identificadores solicitados.
- Se informa de la publicación deliberada del formulario de servidores y del posible contenido sensible de los registros antes de compartirlos.
- No se afirma que «ningún dato sale del equipo» ni se inventa una política completa de los proveedores externos. Las retenciones y obligaciones de TP API deben completarse con las de su operación real.

## Verificación y límites

Se compilaron en memoria con `svelte/compiler` los seis componentes modificados: TermsConditions, LegalNotice, CommunityTools, HostingOptions, CommunitySpotlight y ServerDirectory. No produjeron advertencias. Sus scripts pasaron la comprobación de sintaxis TypeScript. Se revisaron por lectura la integración de navegación/footer, la política de caché y la retirada del fallback.

Esta revisión parcial no ejecutó la suite global de npm/cargo, no probó una compra, una comisión, una sesión Microsoft real, todos los clics en un ejecutable instalado o una interrupción de producción de TP API. La coordinación principal debe registrar por separado el resultado de sus comprobaciones globales. Tampoco se certificó el contenido de un instalador antiguo ni que GitHub y todas las distribuciones ya publicadas coincidan con estos fuentes. Para distribuir una nueva compilación deben incluirse los avisos pertinentes y cumplirse las obligaciones de las licencias del código y dependencias.

Siguen siendo comprobaciones de publicación: permisos de marca que no consten documentados, contratos reales de API/afiliación, origen de fondos y skins personalizados, obligaciones de código fuente y avisos de la compilación que se distribuya, y la adecuación de privacidad/publicidad a la operación y jurisdicción reales. Este documento no elimina esos pendientes.

## Integración funcional de ambos pedidos

- El lateral utiliza SVG propios y mantiene el logo de TP. Los seis accesos rápidos tienen un solo icono semántico por botón; Capturas ya no duplica símbolos y Shaders usa un sol vectorial.
- Los 57 módulos de plugins comparten un registro de iconos entre launcher y menú del juego. Se aprovechan texturas adecuadas de un cliente local cuando están disponibles; si faltan o fallan, se muestra un SVG semántico. No se incorporaron PNG de Minecraft al repositorio para estas funciones.
- Los logos existentes de los cargadores y proveedores se mantienen. El icono propio de Java permanece intacto.
- NeoForge y Quilt aparecen en Jugar, Nueva instalación y la barra inferior. Sus catálogos consultan los endpoints oficiales, cruzan IDs exactos del manifiesto de Mojang y conservan una caché validada. La instalación utiliza el gestor compartido de progreso y perfiles.
- OptiFine abre la web oficial por una acción explícita cuando falta una instalación. El detector consulta versiones, mods, instancias y la carpeta Descargas real de Windows, sin ejecutar JAR al escanear. Usa caché por metadatos y límites de tiempo/archivos/bytes. La importación posterior vuelve a validar contenido y huella. Los ZIP históricos sin el Tweaker esperado no forman parte del autodetector.
- Las exportaciones ZIP y mrpack comparten una política conservadora: archivos Modrinth rastreados pueden referenciarse con sus hashes; OptiFine, recursos oficiales reconocidos y binarios con permiso no verificable se excluyen. El diálogo y `export-report.json` enumeran nombre, versión declarada, archivo y motivo. La instalación original no se borra ni modifica por exportar. La importación rápida admite las referencias verificadas del nuevo ZIP.
- La consola limita su vista a las últimas 1.500 líneas. Una sesión nueva inicia un buffer visual nuevo. Tras 12 minutos sin logs, interacción ni partida activa, la vista se vacía. Estas limpiezas y el botón manual conservan el archivo persistente de diagnóstico.
- Comunidad contiene 14 herramientas; los espacios de hosting/afiliación están preparados y desactivados como acuerdos comerciales. Servidores usa TP API o su caché auténtica; se retiró Selección TP y el fallback que mezclaba otras fuentes.

## Comprobación global de la integración

- `npm run check`: cero errores y cero advertencias.
- `cargo test --quiet`: 91 pruebas aprobadas, cero fallidas y 15 omitidas que requieren red, instalaciones o ejecución de clientes reales. Incluye fixtures de OptiFine, política de exportación, catálogos nuevos y recursos de iconos.
- `node scripts/test-plugin-icons.mjs`: cobertura de los 57 módulos, rutas de recursos locales y fallbacks SVG; seis accesos con un icono.
- `node scripts/test-console-buffer.mjs`: límite, retención del último error, preservación del archivo de origen y expiración con guardas de actividad.
- Se abrió la aplicación de desarrollo y se comprobó la vista Términos y condiciones, su posición en el lateral y el uso de iconos propios. Los endpoints de ambos cargadores generaron sus cachés válidas en el dispositivo.

Estas comprobaciones no equivalen a probar el arranque de todas las versiones de Minecraft con todos los cargadores, ni a acreditar permisos jurídicos pendientes. No se generó ni publicó un instalador nuevo en esta tarea.

## Incidencia posterior: pantalla negra en Minecraft 26.3

El registro real señaló fallos de compilación en `armor_cutout_no_cull` y `armor_decal_cutout_no_cull`, con `Unable to find shader defined uniform (Sampler1)`. La inspección del bytecode de Hit Color `fabric-26.3-0.1.jar` mostró que su mixin suprime el define `NO_OVERLAY` precisamente en esos pipelines. El SHA-1 de esa compilación es `ebfe5a8f1aec0d0f96f5b1a3176eff74bc70992d`.

Se ejecutó una prueba A/B real con el mismo Minecraft/Fabric/Java y todos los demás mods, en perfiles aislados `TPHudBad263` y `TPHudGood263`, sin copiar mundos ni alterar la instalación original. El primero reprodujo Sampler1; el segundo, omitiendo solo ese JAR, cargó sonido y recursos, permaneció abierto y no produjo ese error. Ambos conservaron la preparación del puente de telemetría. La prueba cerró exclusivamente los PIDs registrados para esos perfiles de prueba.

La protección añadida se aplica a esa combinación exacta de juego, cargador, módulo y huella. La selección automática omite la compilación identificada y puede aceptar otra futura. Un archivo gestionado por TP se pausa mediante `.jar.disabled`, sin borrarlo ni cambiar la preferencia global del usuario. Un archivo ajeno o alterado no se renombra: se muestra un error concreto. Si un respaldo impide la pausa, se evita el arranque inseguro y se explica el conflicto. El launcher muestra el aviso de compatibilidad al preparar la partida.

También se corrigió el recorte de widgets con posiciones guardadas al reducir la ventana o aumentar su escala. Su posición visible se calcula con el tamaño real del widget y el viewport; las coordenadas guardadas permanecen intactas para restaurar su colocación al ampliar la ventana. Se conserva el movimiento con ratón y teclado.

Verificación posterior: `npm run check` mantuvo cero errores y cero advertencias; `cargo test --quiet` aprobó 94 pruebas con 16 omitidas por requisitos externos. Además se ejecutó expresamente y aprobó `hud_compat_smoke::hit_color_263_real_client_ab` (156 segundos). `node scripts/test-hud-widget-bounds.mjs` comprobó resolución, escalas global e individual, crecimiento del contenido, restauración sin mutar el perfil y límites de los bordes. Este diagnóstico no atribuye otros posibles fallos a todos los mods de renderizado ni garantiza cada módulo en cada versión.
