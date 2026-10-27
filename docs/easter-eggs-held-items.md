# Personaje, objetos y descubrimientos

## Objetos 3D

`held_items.ts` conserva los identificadores de equipamiento y genera accesorios geométricos originales de TP. No incorpora sprites, texturas ni modelos JSON extraídos del juego. Los objetos usan materiales lisos con detalles violetas y un punto de agarre local definido en la palma. Los nombres de compatibilidad no atribuyen a TP las marcas del juego.

Cada objeto se acopla como hijo del grupo del brazo correspondiente. Sigue las rotaciones de las articulaciones y aparece en la exportación PNG transparente. El Estudio 3D permite elegir cada mano de forma independiente, asociar objetos al tipo de pose y recordar el equipamiento de las poses personalizadas en las preferencias locales del editor.

El personaje del fondo de Jugar equipa automáticamente objetos adecuados para poses como guardia de espada, arco tensado, bloqueo con escudo, minería y construcción. Ese fondo mantiene su presentación sin controles visibles.

## Motor de secretos

`easter_eggs.ts` contiene 1089 entradas con identificadores y textos únicos:

- 1040 frases compuestas a partir de 52 referencias históricas y mecánicas de Minecraft y 20 plantillas editoriales.
- Ocho apariciones visuales: Herobrine, Entity 303, Null, Steve con corona, Dinnerbone, Grumm, Creeper cargado y Enderman.
- 24 mensajes de consola, doce eventos de calendario/hora y cinco descubrimientos por interacción.

El motor realiza una única tirada global al iniciar: 1,2 % de probabilidad total. Herobrine conserva exactamente 0,1 %. No ejecuta mil tiradas independientes. Los valores `chance` de las entradas son metadatos de rareza de contexto; la puerta global limita la frecuencia real.

Dinnerbone y Grumm invierten temporalmente al personaje. Las variantes históricas de criaturas muestran ahora un centinela flotante de cristal original, sin reproducir el modelo de un mob. Las apariciones afectan al fondo del launcher; no cambian la skin de Microsoft, el perfil de Minecraft ni los archivos de la cuenta.

`getWelcomeSession()` conserva una sola elección durante toda la sesión del documento, incluidos los cambios de pestaña. `getSecretState()` y `subscribeSecretState()` permiten mostrar el progreso. Los identificadores descubiertos se validan frente al catálogo y se guardan localmente; si el almacenamiento no está disponible, el contador sigue funcionando en memoria.

`startEggClock()` comprueba eventos por minuto local mientras el launcher está abierto, con una tirada máxima del 2 % y una sola oportunidad para cada evento durante esa sesión. Incluye las 03:33, medianoche, fechas señaladas y el aniversario de Minecraft. Los eventos solo generan presentación o mensajes de texto: no ejecutan comandos del juego.

## Verificaciones realizadas

Se comprobaron los 1089 identificadores y textos únicos, los límites de probabilidad, persistencia y deduplicación del contador, las doce geometrías y su movimiento al rotar el brazo. Un barrido de 100000 muestras obtuvo 1200 eventos de inicio y 100 apariciones de Herobrine, exactamente las frecuencias previstas.
