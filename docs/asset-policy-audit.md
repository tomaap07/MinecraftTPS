# Auditoría visual y condiciones de distribución

Revisión: 25 de octubre de 2026. Alcance: recursos distribuidos en este árbol de
fuentes, interfaz del launcher y gráficos de su instalador. No es una certificación
jurídica del producto, de sus integraciones, ni de archivos aportados por usuarios.
Las políticas de los titulares pueden cambiar.

## Corrección de alcance solicitada

Los logos oficiales de Fabric, Forge, OptiFine, Modrinth y CurseForge están
restaurados como identificadores de sus integraciones. Forge muestra solo el
trazado oficial del yunque en un SVG cuadrado, sin el texto horizontal. La barra
«VERSIÓN» conserva sus cuatro SVG originales de TP y la cuadrícula de Instaladas.
Los componentes compartidos restablecen también los filtros de Mods y modpacks.

Esta corrección visual no afirma que todos los titulares concedan permiso general,
ni que las APIs exijan mostrar todos esos logos. Las reservas de derechos y fuentes
consultadas se conservan como documentación, sin presentar su disponibilidad pública
como prueba de licencia. No se cambió la autenticación ni la lógica de filtros.

## Criterio aplicado

Una URL pública, un favicon descargable o una licencia del código no bastan por
sí solos para atribuir permiso sobre una marca. Se conservan recursos con una base
documentada y uso descriptivo secundario; las decisiones de presentación vigentes siguen el alcance solicitado arriba,
manteniendo la identificación de cada integración y la separación de su marca
respecto al arte propio de TP. No se presenta a ningún proveedor como patrocinador.

## Mojang / Microsoft y arte de TP

Las [Usage Guidelines](https://www.minecraft.net/en-us/usage-guidelines) distinguen
las referencias descriptivas al juego del uso de sus activos como identidad de otro
producto. Por prudencia se retiraron las texturas de bloques, objetos y criaturas
de TODOS los iconos de la interfaz, no solamente del selector de versiones.

- `GameIcon` usa vectores originales. El lateral, logros, tarjetas y menús comparten
  esa implementación; ya no extraen imágenes de clientes instalados.
- Se eliminó el extractor `game_textures.rs` y sus comandos IPC, el atlas de iconos y
  los modelos de objetos que copiaban texturas/modelos del juego al launcher.
- Reloj: símbolos propios de sol y luna, conservando zona horaria y hora local.
- Poser: accesorios geométricos nuevos; las variantes de criaturas utilizan un
  centinela de cristal propio, sin reproducir sus siluetas del juego.
- Skins de reserva: astronauta TP original y variante de visor espectral. Se retiró
  Steve y la copia de Steve con ojos blancos. Los identificadores históricos de
  secretos y equipamiento siguen siendo compatibles con las preferencias guardadas.
- Logo: cristal dimensional dentro de un escudo; se retiraron los ojos que evocaban
  al Enderman, los JPG antiguos y el render de una skin personal del instalador.
  Los iconos de aplicación, favicon y BMP de NSIS se regeneraron con la nueva marca.
- Vista previa del HUD: paisaje vectorial propio. La captura aportada para la
  maqueta anterior no se redistribuye, al no haberse verificado su autoría.
- Noticias: se mantienen titulares, resúmenes y enlaces oficiales; las portadas
  de marketing remotas se sustituyen por ilustraciones propias de TP.
- Las imágenes subidas por el jugador, sus capturas F2, su skin/capa y las portadas
  recibidas de proveedores son contenido del usuario/proveedor, no arte de TP ni
  una licencia general para redistribuirlo. Sus funciones siguen disponibles.
- Los archivos que Minecraft necesita para ejecutarse se descargan por la ruta de
  instalación del juego. No se borraron instalaciones, mundos ni recursos del usuario.

Los nombres de Minecraft y versiones son referencias secundarias de compatibilidad.
Se conserva el aviso de no afiliación. El nombre dominante del producto es TP LAUNCHER.

## Logos conservados

| Recurso | Base comprobada y límites |
| --- | --- |
| Fabric | Se restauró el PNG original de [fabricmc.net/assets/logo.png](https://fabricmc.net/assets/logo.png), conservando su identidad visual. El [repositorio del sitio](https://github.com/FabricMC/fabricmc.net) distingue el código MIT del contenido CC BY-NC-SA 4.0; no se atribuye al PNG la licencia Apache del instalador de Fabric. |
| NeoForge | Equipo NeoForged. PNG sin modificar del sitio. El [repositorio del sitio](https://github.com/neoforged/websites) publica [CC BY 4.0](https://github.com/neoforged/websites/blob/main/LICENSE.txt); la imagen se encuentra en `static/img/authors/neoforged.png`, fuera de la excepción del tema Mainroad. Atribución visible y copia de licencia. |
| Quilt | QuiltMC/art, PNG/SVG originales del proyecto. La licencia específica de la carpeta [brand es CC0 1.0](https://github.com/QuiltMC/art/blob/master/brand/LICENSE). Se conserva `quilt_mini_icon_transparent.svg` sin alterar; no implica respaldo de Quilt. |
| Microsoft | Símbolo sin modificar para identificar la autenticación Microsoft, conforme a sus [pautas de inicio de sesión](https://learn.microsoft.com/en-us/entra/identity-platform/howto-add-branding-in-apps). No se usa como logo de TP. Estas pautas visuales no autorizan un Client ID OAuth ni certifican el flujo de autenticación. |
| GitHub | Símbolo secundario para enlazar al proyecto, uso contemplado en el apartado «Do these things» de su [guía de marca](https://brand.github.com/foundations/logo). Sin recolorear, deformar ni sugerir afiliación. |
| Discord | Símbolo original para identificar/enlazar Discord, subordinado al nombre del servicio y sin modificaciones. Uso limitado a referencia al servicio conforme a sus [pautas](https://discord.com/branding); no forma parte de la marca TP ni se utiliza en merchandising. |

El inventario con URL y SHA-256 está en `static/logos/sources.json`. Las decisiones
de visualización se centralizan en `src/lib/brand_assets.ts`. Los avisos y licencias
también pueden leerse en el pie de la aplicación.

## Logos restaurados y condiciones investigadas

| Recurso | Estado y condiciones |
| --- | --- |
| Modrinth | Restaurado desde el repositorio oficial para identificar el catálogo. Su [COPYING](https://github.com/modrinth/code/blob/main/COPYING.md) contiene una reserva de marca y exige permiso escrito. La restauración no equivale a afirmar que ese aviso haya desaparecido o que la API conceda por sí sola una licencia del logo. |
| CurseForge | Favicon oficial restaurado como identificador del proveedor. Los [términos de API](https://support.curseforge.com/support/solutions/articles/9000207405-curseforge-3rd-party-api-terms-and-conditions) y las condiciones de marca son independientes de que el archivo pueda descargarse públicamente. |
| Forge | Logo oficial restaurado; `forge-anvil.svg` contiene únicamente su trazado original de yunque y color, con viewBox cuadrado. No se utiliza un yunque genérico ni el texto horizontal en los filtros. |
| OptiFine | Favicon oficial restaurado como identificador de compatibilidad. Se conserva la referencia a su [copyright](https://optifine.net/copyright); no se redistribuyen instaladores ni JAR dentro de este proyecto. |
| NameMC, SkinMC, Chunk Base, MCStacker, Minecraft Wiki y Planet Minecraft | Las antiguas fuentes demostraban procedencia, no permisos suficientes del dibujo para esta distribución. Se retiran los favicons/logos, incluida la carga remota del favicon de Planet Minecraft. Se mantienen enlaces, créditos y nombres con vectores genéricos propios. |

## Clave de CurseForge

Los términos de su API restringen la divulgación de claves. Se quitó la constante
incrustada en Rust; no se envía al frontend ni se compila con `include_str!`.

Orden de lectura: variable de entorno `CURSEFORGE_API_KEY`; `.env` de desarrollo
(solo builds debug); `integrations.env` dentro de la carpeta de datos del dispositivo.
No se expanden `$`, ni se ejecuta PowerShell. Ambos archivos locales están ignorados
por Git y no figuran en recursos del bundle. Si no hay clave autorizada, se informa
del requisito; Modrinth y los enlaces oficiales continúan disponibles.

Las peticiones se identifican como `TP-LAUNCHER/0.1.0`, sin simular un navegador.
Esto no concede autorización para distribuir una clave, replicar el catálogo,
saltarse restricciones de descarga o usar la API fuera de su contrato.

La clave anterior apareció en esta conversación y pudo estar en ejecutables previos:
quitarla del código no la revoca. Antes de una distribución pública debe revocarse
y sustituirse desde el panel del titular; no se hizo una rotación sin acceso al panel.

## Límites y revisión antes de publicar

Esta limpieza no borra historial remoto de Git, ejecutables/instaladores antiguos,
ni archivos externos del usuario. Los nuevos builds usarán los recursos revisados.
Debe reconstruirse el instalador antes de distribuirlo.

El titular debe proporcionar identidad y contacto válidos para el producto. No se
inventó un correo ni se consideró suficiente un enlace de chat. También deben
revisarse antes de publicar o monetizar los acuerdos de cada API, las autorizaciones
OAuth de la aplicación, licencias de mods/modpacks y contenidos de publicidad.
No se declara que un cambio de iconos garantice que todo el launcher carezca de
riesgos contractuales o jurídicos.

Monocraft se conserva bajo SIL OFL 1.1 con atribución y licencia adjuntas; los
vectores propios no incluyen el archivo de fuente de Minecraft. Los créditos MIT
de skinview3d y Three.js se conservan: son herramientas, no permiso sobre skins ajenas.

## Verificación técnica

- `npm run check`: cero errores y cero advertencias.
- `npm run build` y `cargo check`: completados; el build avisa del tamaño de dos
  chunks existentes, sin impedir la compilación.
- Inventario de imágenes de `static/logos`, SHA-256 del arte de los proveedores,
  opacidad de las nuevas caras, ausencia de referencias al extractor antiguo y
  presencia de avisos legales en el build: comprobados.
- 112 combinaciones de accesorios/manos/anchos de brazo: geometría finita y
  anclaje en la palma; ambas variantes de centinela generan geometría.
- Consulta real de solo lectura a CurseForge, con clave local interpretada y el
  nuevo User-Agent: HTTP 200. No se mostró la clave en los resultados.
- `npm run check:ui`: la estructura de Inicio pasa, pero la comprobación de
  traducciones encuentra entradas incompletas en `pt-BR`, de apartados anteriores.
  No se modificaron los catálogos de idiomas ni se ocultó ese fallo.
- No se reconstruyó el instalador NSIS ni se lanzó Minecraft para estas comprobaciones.

## Ampliación de revisión

Blockbench restaurado tras confirmar permiso público para crédito/enlace. La captura
del diseñador puede elegirse del dispositivo o de F2 sin redistribuirse con el
launcher. Ver `docs/legal-review-2026-10-25.md` para la revisión completa de marcas,
autenticación, API, exportación, fuente y privacidad.
