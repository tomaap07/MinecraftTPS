# TP Client: menú principal dentro de Minecraft

Implementación original para **Fabric 1.21.1, Java 21**. Es un módulo del mod TP Client, separado del HUD y de la ventana de TP Launcher. No cambia OAuth, licencias, servidores, mundos ni las capas oficiales de la cuenta.

## Uso

Al abrir Minecraft con TP Client aparece el menú oscuro de TP. Los botones Un jugador, Multijugador y Realms ejecutan los botones originales de `TitleScreen`, conservando sus restricciones de cuenta. En una sesión demostración se conserva Jugar demostración; Multijugador y Realms siguen deshabilitados cuando Minecraft los deshabilita.

Cosméticos abre la pantalla del mod con capas locales, alas y corona. TP Client abre sus ajustes. Opciones, Idioma y Accesibilidad usan las pantallas originales del juego. Salir solicita el cierre normal de Minecraft. Tab y Enter usan la navegación de widgets nativa.

**Menú Minecraft** devuelve la pantalla original, incluidos los botones de otros mods. Este acceso es la alternativa de compatibilidad cuando otro mod modifica el inicio de una forma que TP no reproduce. En la pantalla original se incorpora un pequeño botón TP CLIENT para volver a la personalización. No se afirma compatibilidad visual con todos los mods de terceros.

## Apariencia y archivos por instancia

La personalización se guarda dentro de `config/tp-client-menu/`:

| Archivo | Contenido |
| --- | --- |
| `menu.json` | Esquema 1, activación, diseño, audio, partículas, parallax, personaje, movimiento reducido y bajo consumo |
| `background.png` | Fondo opcional importado por el usuario y normalizado a PNG |
| `logo.png` | Logo opcional importado, dibujado sin deformar su proporción |
| `*.previous-*.png` | Copias conservadas al restaurar fondo o logo |
| `menu.json.invalid-*` | Copia del archivo dañado antes de usar ajustes seguros |

Personalizar → Visual permite elegir un fondo o logo PNG/JPEG. Se comprueba el formato real, máximo 8 MiB, 4096 píxeles por lado y 8 megapíxeles; se normaliza a PNG sin conservar metadatos y se limita también el tamaño de salida. No se permiten URLs remotas ni enlaces simbólicos. El destino siempre queda en la carpeta de menú de la instancia. El selector nativo solo se abre por una acción del usuario.

El fondo predeterminado se dibuja con degradados, volúmenes geométricos y un máximo de 18 partículas pequeñas. Es arte procedural propio; no contiene texturas del Enderman ni ilustraciones de otros clientes. Movimiento reducido y Bajo consumo desactivan los movimientos. El fondo personalizado se oscurece para preservar el contraste. Las texturas se cargan una sola vez por cambio y se liberan al sustituirlas o cerrar el cliente.

La vista 3D usa `PlayerSkinWidget` y el proveedor de skins del cliente: conserva el modelo Classic/Slim y la skin o fallback que Minecraft tenga disponible, sin enviar credenciales al mod. Admite giro por arrastre y movimiento sutil del modelo al desplazar el puntero. La capa TP local se diferencia de la oficial. La previsualización de alas y corona en esa pantalla todavía no está implementada; sus renderizadores corresponden al personaje dentro del mundo.

## Música original

`assets/tp_client/sounds/menu/endless_dusk.ogg` contiene **Endless Dusk**, una composición ambiental procedural original de TP LABS, de 29.09 segundos, Vorbis estéreo a 22050 Hz. Su fuente reproducible es `tp-client/tools/compose_menu.py`; utiliza síntesis de osciladores, envolventes y ruido generado con semilla fija, sin grabaciones, samples, melodías o archivos de otras obras. El recurso y su fuente se distribuyen bajo GPL-3.0-only, igual que este proyecto.

La reproducción se integra con el motor de audio del juego, una única instancia y volumen de menú persistente con fundido. Respeta los volúmenes Music y Master de Minecraft. Al salir de las pantallas de menú pertinentes o entrar en un mundo se atenúa y detiene. No sigue sonando en la partida. Si se quiere sustituir el tema, hay que añadir una composición propia o autorizada en la misma ruta y reconstruir el JAR; no se cargan canciones desde URLs.

Para regenerar el OGG se necesitan NumPy y una instalación local de FFmpeg con `libvorbis`:

```powershell
python tp-client/tools/compose_menu.py
```

No se empaquetan esos programas dentro del mod. No se garantiza una valoración artística ni un benchmark de sobrecarga por haber generado el archivo; la escucha y la medición del resultado son verificaciones distintas.

## Verificación y límites

Se compila con el Gradle Wrapper de `tp-client/` y JDK 21. `MenuContractsTest` verifica seis casos: límites de todas las filas a varias escalas, configuración normalizada, esquema futuro rechazado, recuperación conservando configuración dañada, persistencia de accesibilidad/audio, validación real de imágenes y recursos de audio empaquetados. El informe XML de Gradle es la evidencia de ejecución; no sustituye la prueba interactiva de botones, sonido y piel en Minecraft.

El panel Social no se muestra porque su integración de identidad y presencia no está operativa en el mod. No hay notificaciones de actualizaciones inventadas. El sonido de clic de los botones conserva el comportamiento de UI de Minecraft y aún no dispone de un interruptor independiente. El modelo animado y las partículas no tienen un benchmark de consumo en esta entrega. Solo se declara la variante Fabric 1.21.1 compilada; no se supone soporte Forge, NeoForge ni snapshots.

The menu is an independent in-game Fabric 1.21.1 module. It preserves Minecraft's original account restrictions, uses original procedural visuals and music, and exposes the original Minecraft menu as a compatibility fallback. Automated checks do not imply that every interactive path, third-party mod or performance scenario was tested.
