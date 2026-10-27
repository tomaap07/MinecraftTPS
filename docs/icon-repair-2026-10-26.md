# Iconos, portadas y accesorios originales

Cambios del 26 de octubre de 2026. El alcance excluye el menú lateral izquierdo
y el HUD In-Game. La comparación contra la instantánea previa de esta tarea
comprueba el bloque completo del lateral y los archivos del HUD protegidos.

## Presentación y procedencia

- Releases y Betas/Alphas conservan su cuadrícula SVG 16×16, con caras más claras,
  borde plata y energía violeta. Instaladas, Forge, Fabric, OptiFine y Snapshots
  conservan sus iconos anteriores.
- Comunidad utiliza `CommunityIcon.svelte`: favicons consultados por HTTPS en el
  servicio de Google y dibujos pixel-art propios cuando una imagen falla. No se
  descargaron ni añadieron esos favicons al repositorio. El proveedor recibe la
  petición de red; servir una marca remotamente no concede una licencia de marca.
- Noticias conserva el feed oficial y sus fechas. Resuelve rutas relativas de
  `playPageImage.url` y `newsPageImage.url` y prueba la imagen alternativa antes
  del respaldo SVG. Las imágenes permanecen en sus servidores; no se incluyen en
  el instalador. No se afirma que enlazar cualquier imagen sea siempre lícito.
- Los runtimes usan una taza pixel-art original de TP y un badge de versión,
  también en el asistente inicial. No se utiliza el dibujo de la marca Oracle Java.
- Los accesorios de Poser se generan en un canvas de 16×16. Los píxeles opacos
  forman mallas recortadas con grosor normalizado 1/16 y agarres por forma. Los
  bloques siguen siendo cubos volumétricos con patrones originales. Se eliminó
  la lectura de JAR y las descargas de texturas desde `get_poser_assets`.

## Biblioteca de contenido

El lector local conserva límites de ZIP, dimensiones y tamaño de imagen. Admite
logos declarados por Fabric, Forge/NeoForge y `mcmod.info`, rutas históricas y
logos convencionales del propio JAR. La caché de metadatos fue versionada para
volver a procesar resultados vacíos antiguos. Las consultas externas usan el
proveedor/proyecto conocido o una coincidencia exacta y no ambigua del nombre.
La vista muestra resultados progresivos y respaldo propio a color cuando no
existe una imagen disponible.

Auditoría real, de solo lectura, de RLCraft Dregora: 216 JAR examinados, 200 con
metadatos reconocidos, 67 con icono interno válido. Los 149 restantes necesitan
catálogo o respaldo; no se inventaron 149 logos oficiales ni se ejecutaron todas
esas búsquedas externas durante la prueba. RLCraft y RLCraft Dregora ya tenían
portada local antes de esta corrección: se reparó su presentación y recuperación.

## Cuentas y distribución

La configuración del OAuth propio y las restricciones de Offline/OptiFine se
describen en [Microsoft y perfiles desconectados](microsoft-client-configuration.md).
La gracia de 30 días es una política técnica del launcher, no un plazo de licencia
establecido por Mojang. No se registró una aplicación Microsoft ni se obtuvo una
autorización nueva en nombre del usuario. Falta proporcionar un Client ID propio
válido para probar el nuevo login de extremo a extremo.

Estas medidas reducen exposiciones concretas. No sustituyen los permisos de cada
titular ni certifican todos los recursos, integraciones, versiones del instalador
o términos jurídicos del proyecto. No se generó un instalador nuevo en esta tarea.
