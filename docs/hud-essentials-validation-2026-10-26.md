# HUD esencial: correcciones y validación

Fecha: 26 de octubre de 2026.

## Cambios aplicados

- El catálogo y los perfiles conservan FPS y latencia como indicadores de rendimiento. Se retiran FPS mínimos/1% Low, tiempo entre cuadros, CPU, memoria, recolector de basura y estadísticas redundantes. La migración conserva las posiciones y ajustes de los módulos que siguen disponibles.
- TP Bridge deja de consultar CPU/memoria y de calcular percentiles de fotogramas. Sigue contando presentaciones reales mediante las envolturas Java de LWJGL cuando el contador del cliente no está disponible. No modifica código de Minecraft ni funciones nativas.
- El equipamiento transmite `item_id`, `slot`, `remaining` y `maximum`. Los slots modernos se resuelven por la enumeración del juego, incluyendo casco, pechera, pantalones, botas y ambas manos. Se añaden símbolos legacy para objetos, nombre de objeto y mano principal. Las traducciones conocidas de herramientas/armaduras antiguas se convierten a identificadores de textura locales.
- El HUD nativo muestra el icono local y el porcentaje de durabilidad; TP Companion también dibuja armadura y manos con los iconos del propio juego y porcentaje. Los objetos sin desgaste no reciben un porcentaje inventado.
- Se corrige la lectura de luz: `getBrightness(LightLayer, BlockPos)` pertenece a `BlockAndTintGetter`; el parser anterior descartaba sus dos argumentos y los canales BLOCK/SKY podían estar ofuscados.
- Se admite `Window.handle()` de 1.21.9, además del getter anterior. Los archivos de bindings se reconstruyen en cada lanzamiento a partir de los mapas disponibles: no es necesario borrar las carpetas del usuario para aplicar los nuevos roles.
- TP Companion deja de solicitar estadísticas masivas y de recorrer entidades para módulos retirados. Conserva los datos de jugador que usa el catálogo esencial.

## Zoom con C

TP Bridge consulta la tecla y el foco desde el ejecutor del juego. En versiones que exponen las APIs necesarias aplica un FOV temporal y restaura el original al soltar C, perder el foco, abrir una pantalla del juego o desactivar el módulo/HUD. No llama a `saveOptions` para ese cambio.

El acercamiento genérico respeta un FOV mínimo de 30, por lo que su aumento es más conservador que el zoom de cámara de TP Companion. Si se detectan TP Companion, OptiFine, Zoomify u OkZoomer, el puente delega el zoom y evita sumar otro cambio de FOV encima. La integración de Companion conserva su tecla C.

No se publica la capacidad `optical_zoom` del puente si faltan el ejecutor, una opción FOV numérica o una API de foco/tecla compatible. La pantalla completa sin bordes se aplica únicamente cuando está activada su opción correspondiente; una ventana externa no se garantiza sobre pantalla completa exclusiva.

Referencias técnicas: [consulta de teclas de GLFW](https://www.glfw.org/docs/latest/group__input.html), [foco y atributos de ventana de GLFW](https://www.glfw.org/docs/latest/window_guide.html#window_attribs).

## Pruebas ejecutadas

```text
npm run build:bridge
node scripts/test-bridge.mjs
```

Ambos comandos terminaron correctamente. El segundo levanta procesos JVM reales con fixtures originales que imitan APIs del cliente y de LWJGL; no incluye código ni recursos de Minecraft. Verifica:

- FPS, XYZ, ping y equipamiento con nombres públicos, campos ofuscados y coordenadas legacy.
- Iconos identificados como `minecraft:diamond_helmet` y `minecraft:diamond_sword`, slots de cabeza/mano y durabilidad real de las fixtures.
- Niveles de luz de bloque/cielo de la fixture, sin valores de relleno.
- C con FOV float e integer: acercamiento, retorno al soltar, retorno al perder foco, retorno al abrir menú y retorno al desactivar; el contador de guardado de opciones no cambia.
- Ausencia de CPU y 1% Low en la telemetría nueva.
- Transporte UDP autenticado local, PID/timestamps, restricciones de depuración del servidor, menú principal sin mundo inventado, desactivación y reanudación.
- Días desde guardados NBT válidos y rechazo de datos incompletos/descompresión excesiva.

Se añadieron pruebas Rust de migración de perfiles y de resolución de `Window.handle` y luz con argumentos tipados. La comprobación y compilación Rust/Companion final se ejecutan en la tarea principal.

## Límites de la validación

Las pruebas JVM no sustituyen una partida interactiva en cada release, snapshot, modpack o controlador gráfico. No se ha verificado visualmente cada versión de Minecraft. Las ediciones sin símbolos o métodos compatibles deben mostrar indisponibilidad en lugar de métricas falsas.

El puente respeta `reducedDebugInfo`; no publica coordenadas ni información técnica restringida cuando el servidor la desactiva. El minimapa y las funciones profundas dependen de un adaptador compatible. Los identificadores de texturas de mods ajenos que no resuelva el extractor conservan el icono de respaldo. No se promete compatibilidad universal de todas las funciones con versiones futuras.
