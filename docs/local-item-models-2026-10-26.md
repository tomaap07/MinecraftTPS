# Modelos locales del personaje

## Origen

El IPC `get_poser_assets(version_id)` mantiene `textures` (iconos 16 px) y añade
`models` y `unavailable`. Lee exclusivamente recursos que ya existen en los JAR
de la instalación seleccionada y sus padres. No hace solicitudes HTTP ni copia
texturas/modelos de Minecraft al repositorio o al instalador.

Los modelos resuelven parents y variables de textura locales del namespace
minecraft. Los prefijos modernos `minecraft:item/...`, los nombres históricos
`items/...` y los display de cada mano se conservan. Ballestas resuelven su capa
`crossbow_standby`; el tridente usa `trident_in_hand` y su textura de entidad;
el escudo transmite la hoja completa `shield_base_nopattern` cuando existe.
Las definiciones modernas `items/*.json` pueden referenciar directamente un
modelo de bloque; se resuelve su rama predeterminada sin uso/carga y con el
mismo confinamiento. Se reconocen también las rutas anidadas de texturas de
escudo y tridente de 26.2.
Los bloques mantienen sus elementos, caras, UV, rotaciones y overlay de césped.
El color de césped sale del colormap local en el punto de temperatura/humedad
predeterminado (no se dibuja una textura de reemplazo).

## Contrato

Cada entrada de models tiene kind sprite, model, shield o trident; textures con
PNG data URL validados; elements con from/to/faces/rotation; display.right y
.left con rotation/translation/scale; source_model; y tint_color opcional.
Las coordenadas de elementos y las UV conservan las unidades 0–16 del archivo.
Las transformaciones de display aún no están espejadas por mano: eso corresponde
al renderer. Las texturas sprite usan el primer frame cuadrado a resolución
original; las hojas de entidades se conservan completas.

No se inventa un modelo si faltan recursos o el formato no está soportado.
En concreto, los atlas de versiones sin modelos JSON de objetos no se convierten
a accesorios aproximados. Los objetos inexistentes en una versión (por ejemplo,
netherite en 1.12.2) quedan ausentes. La UI puede usar otros recursos instalados
para sus iconos decorativos, pero los objetos del personaje usan los modelos de
su versión seleccionada.

## Límites y pruebas

Límite de 12 padres, 256 JSON y texturas por JAR; cada modelo JSON se limita a
64 KiB y 128 elementos. Los PNG se validan antes de decodificarse con límites de
bytes, dimensiones y memoria. Se rechazan namespaces externos, rutas relativas,
URLs, ciclos, transformaciones no finitas y UV/rotaciones fuera de límites.
La lectura de assets/objects conserva su comprobación SHA-1.

Hay fixtures propias para herencia por mano, UV/alias por cara, ciclos y rutas
externas, definiciones modernas y hojas de entidades completas. Las cuatro
pruebas de fixtures pasan. La lectura real de 26.2 devuelve 41 texturas y 30
modelos válidos; 1.21.1 devuelve 41/30 y 1.12.2 devuelve 25/22 (los objetos
modernos inexistentes quedan ausentes).
La prueba ignorada `installed_client_art_is_read_without_writing_game_or_repository_files`
acepta TP_LOCAL_ART_VERSION y comprueba los recursos de instalaciones reales.
La tarea principal ejecuta la compilación y registra sus resultados finales.

Extraer un archivo local evita incluirlo en la distribución; no constituye por
sí mismo una licencia adicional ni una garantía jurídica absoluta.

## Verificación final de esta modificación

- `npm run check`: 0 errores y 0 advertencias.
- `cargo test --lib`: 82 aprobadas, 0 fallidas y 15 pruebas de integración
  opcionales ignoradas.
- Lectura real sin escribir archivos del juego: 1.21.1 (41 texturas / 30 modelos),
  26.2 (41 / 30) y 1.12.2 (25 / 22; objetos posteriores ausentes).
- `node scripts/test-local-item-renderer.mjs`: 120 poses, 1440 contactos de mano
  classic/slim, caras y UV válidas; recursos ausentes no generan accesorios.
- Comprobación visual en la aplicación Tauri: espada de hierro y escudo,
  tablones de roble y arco desde el cliente 1.21.1 heredado por NeoForge.
- `npm run build:desktop`: compilación de TP Bridge, TP Companion y frontend
  correcta. Vite conserva su advertencia de chunks mayores de 500 kB.
- El chequeo adicional `npm run check:ui` detecta traducciones pt-BR incompletas
  anteriores a este cambio; no se oculta ni se cambia ese chequeo.

Esto no afirma que todos los recursos de todas las ediciones estén soportados.
Los clientes anteriores al sistema de modelos JSON no reciben accesorios
aproximados. Tampoco significa que se haya regenerado el instalador NSIS: el
ejecutable publicado anteriormente no se modifica al editar y compilar el código.
