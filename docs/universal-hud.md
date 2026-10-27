# HUD híbrido de TP LAUNCHER

El overlay de Windows comparte perfil y editor entre Minecraft Vanilla, OptiFine,
Fabric y Forge. Esa independencia de ventana no convierte las funciones internas
ni el mod Fabric 1.21.1 en universales. Las capacidades de cada adaptador determinan
qué datos y controles aparecen; no se simulan FPS, coordenadas ni impactos.

## Uso

1. En Plugins activa el HUD antes de iniciar Minecraft desde TP.
2. F8 o Shift derecho abre el editor. No se envía Escape al juego.
3. Arrastra la barra superior para mover el menú. «Mover widgets» permite arrastrar
   las cajas, alinear con guías y ajustar con las flechas del teclado.
4. Ajustes y colores ofrece transparencia de 10–95%, desenfoque, temas, colores
   de texto/fondo/borde y Chroma. Los cambios se guardan en hud-profile.json.
5. El perfil conserva posiciones normalizadas; cambiar un preajuste no desactiva
   los optimizadores ni las preferencias de motor ajenas a los widgets.
6. La opción sin bordes conserva y restaura el estilo anterior. F11 exclusivo
   requiere volver al modo ventana; Windows no garantiza overlays externos en
   pantalla exclusiva o sobre procesos elevados.

El catálogo tiene **160 módulos**, incluidos 122 indicadores numéricos/calculados
con registro único. Se pagina en el launcher y permite buscar por categoría. Las
métricas nuevas están desactivadas por defecto. Esa cantidad no significa que las
115 funciones visuales propuestas estén implementadas ni que cada edición exponga
147 datos: se han añadido también estadísticas del cliente y del proceso Java.

## Datos y límites

| Función | TP Bridge + overlay | TP Companion Fabric 1.21.1 |
| --- | --- | --- |
| WASD, CPS, reloj, diseño y mira SVG | Ventana de Minecraft lanzada por TP | Mismo diseño nativo |
| FPS / frame time / 1% low | Contador del cliente o llamadas reales LWJGL 2 / GLFW | Contador interno y puente |
| Días / posición / bioma / armadura | Getters resueltos por versión y capacidades | API interna del cliente |
| Inventario, efectos, estadísticas | Solo los valores resueltos; no se rellenan con cero | Lectura interna; estadísticas tras respuesta del servidor |
| RAM / CPU / GC / hilos | JVM Management API, sin requerir administrador | Se combinan con el puente |
| Impacto confirmado y combo | Requiere adaptador compatible | Paquete de daño cuyo atacante es el jugador |
| Ocultar mira original | Requiere adaptador compatible | Mixin del HUD, solo si el overlay está visible |
| Niebla | Wrappers Java GL11 / RenderSystem compatibles | BackgroundRenderer, dimensiones y fluidos |
| Packs, opciones y shaders | Allowlist y confirmación por edición; Iris/OptiFine detectados | Controles internos; shaders necesitan Iris |
| Menú TP, galería F2 y físicas visuales de objetos | Requieren adaptador del motor | Implementados en el mod compatible |

No hay PresentMon, ETW, elevación de permisos ni estimación con DWM. La frecuencia
compositora o del WebView no se presenta como FPS del juego. LWJGL 2 y GLFW tienen
adaptadores diferentes; un método nativo no tiene bytecode Java instrumentable.
El contador hookea los wrappers Java de presentación. ASM 9.8 está aislado bajo
un paquete propio, conserva su licencia BSD 3-Clause y no suplanta el ASM de mods.

El agente publica cada 200 ms por UDP autenticado en 127.0.0.1, con archivo atómico
por PID como reserva. Rust verifica PID lanzado por TP, token, tamaño, antigüedad,
valores finitos, capacidades y límites. Los contadores no acceden a cuentas o
contraseñas ni envían telemetría fuera del equipo. Las restricciones reduced debug
info eliminan posición, orientación, bioma y datos técnicos relacionados.

Día = floor(DayTime / 24000) + 1, más el ajuste manual del perfil. Si falta el getter,
solo se consulta level.dat del mundo individual identificado por su servidor
integrado. Nunca se elige el mundo más reciente por su nombre/fecha. Ese respaldo
se rotula «último guardado», admite hasta 15 minutos de antigüedad y limita tanto
el archivo como la descompresión y profundidad NBT. No equivale a tiempo en vivo.

Singleplayer confirmado muestra 0 ms · Local. En servidores se mide el handshake
TCP cada cinco segundos, excluyendo DNS/SRV, con límite de tiempo; se rotula TCP.
Mientras no hay muestra puede usarse la latencia que expone el cliente. No se
promete que el handshake TCP equivalga al ping del protocolo o al del servidor.

## Mira, menú y controles

La mira tiene siete estilos, lienzo 15/17 píxeles, grosor, separación, punto, borde,
opacidad, colores y Chroma. El movimiento amplía la geometría. La distancia usa la
intersección real con la entidad apuntada; el servidor decide el alcance válido.
El hitmarker responde a daño confirmado, no a hacer clic. El combo cuenta impactos
sobre el mismo objetivo separados por menos de dos segundos y se reinicia al
recibir daño. Una recarga de 100% no garantiza un crítico.

Los comandos del juego son una allowlist de datos locales por PID. Se ejecutan
en su hilo principal y esperan confirmación. No admiten scripts o reflexión libre
proporcionada por el usuario. Las recargas esperan su resultado asíncrono. Los
atajos F3+B/G/D/H/T devuelven el foco al juego y algunas ediciones antiguas no los
implementan. pauseOnLostFocus:false se conserva antes de iniciar y se aplica en
caliente mediante el ejecutor del cliente, sin enviar Escape.

La galería F2 del mod usa capturas de su instancia, miniaturas acotadas y páginas.
Permite ampliar, copiar y usar de fondo. El tema TP preserva los textos legales y
acciones del menú Vanilla, ofrece vuelta al tema original y fondos personales.
PNG/JPG/GIF pueden elegirse en el mod; el panel del overlay también convierte WebP.
Las animaciones se guardan como imagen fija. Las imágenes nunca se suben.

Los shaders que implementan una niebla propia pueden omitir los hooks normales.
Las físicas de objetos cambian su orientación visual, no la simulación del servidor.
No se inventan temperatura GPU, TPS de servidores ajenos ni radar de sonidos.
Skin Layers 3D, Wavey Capes, animaciones clásicas, Freelook, borde de bloque,
motion blur, fuego bajo, Clean View, Hit Color, partículas, nametag, ragdolls y
TNT Timer se preparan como integraciones externas de Modrinth con edición exacta
y crédito al proyecto. No son implementaciones propias del overlay ni switches
universales en caliente. Sus controles internos pertenecen al mod correspondiente.

## Optimización y arte

Sodium, Lithium, FerriteCore, EntityCulling, ImmediatelyFast, Dynamic FPS e Iris se
resuelven con la versión y cargador exactos en Modrinth, con hashes y dependencias.
Si se seleccionan optimizadores para una Release Vanilla, se prepara una instalación
Fabric independiente y se devuelve al selector su versión de lanzamiento. El perfil
Vanilla permanece disponible. Si no existe un Fabric compatible, se informa y se
conserva el lanzamiento original; no se pretende ejecutar mods sin cargador. Los complementos gestionados se registran con nombre y SHA-1: se desactivan
reversiblemente como .jar.disabled al siguiente inicio. Los archivos manuales o
modificados por el usuario no se reclaman ni se eliminan. Los ajustes G1GC son optativos; no se promete un porcentaje de FPS y una JVM
con DisableExplicitGC puede ignorar el botón de recolección de memoria.

Comunidad carga los favicons solicitados con fallback propio. Noticias muestra las
portadas del feed. Los JAR de mods aportan sus iconos, con caché de API si faltan.
Poser usa texturas de instalaciones locales o caché privada verificada de Mojang,
con pixel-art propio de reserva: no se incluyen texturas de Minecraft en el repo o
instalador. Cargar una imagen por URL no concede derechos; siguen vigentes los
avisos de terceros y la revisión de docs/legal-review-2026-10-25.md. No se ofrece
una garantía jurídica de «100% legal».

## Verificación reproducible

- npm run check y npm run build.
- npm run build:bridge y npm run build:companion.
- npm run test:hud: registro/unidades/restricciones, procesos JVM, datos UDP,
  menús, preferencias del hilo del juego, opciones modernas, audio, niebla y NBT.
- cargo test --manifest-path src-tauri/Cargo.toml --lib.
- Las pruebas ignoradas de mapas reales y rutas Windows se ejecutan con
  TP_HUD_REFERENCE_MAPS y TP_HUD_TEST_JAVAS, respectivamente.

Las pruebas de procesos usan fixtures propios, no sesiones de todas las versiones
de Minecraft. No demuestran compatibilidad de cada mod, snapshot, anticheat o GPU.
La revisión visual adicional del navegador local no se realizó por permiso
denegado. El instalador previo no se actualiza hasta reconstruir su bundle.

### Corrección de inicio del agente en Windows

El registro aportado el 25 de octubre mostró cierres de 26.2/26.3 antes del método
principal: `Unexpected error (103) returned by AddToSystemClassLoaderSearch` y
`processing of -javaagent failed`. Se reprodujo con Java 25: el mismo JAR falla
con `\\?\C:\...` y carga con `C:\...`. Las pruebas anteriores habían usado rutas
convencionales y no cubrían la ruta canónica del directorio de recursos Tauri.

`hud_bridge.rs` normaliza las rutas del agente/configuración (incluidas UNC),
conservando espacios y Unicode. Rechaza rutas de dispositivo o volúmenes que no
puedan representarse de manera convencional. No modifica las rutas de recursos
ni las instalaciones del jugador.

Antes de añadir el agente al juego se ejecuta una JVM oculta y separada con el
mismo Java, argumento `-javaagent` y `-version`. Si falla o excede seis segundos,
se omite TP Bridge durante ese lanzamiento y se explica en la consola. El editor,
teclas y fuentes alternativas siguen disponibles. Una excepción nativa previa a
`premain` no puede ser capturada por el try/catch del agente; por eso esta prueba
ocurre antes de lanzar Minecraft. No se inicia sesión ni se abre un mundo en ella.

Verificación: reproducción del fallo original, normalización de disco/UNC,
rechazo de rutas ambiguas y arranque real del agente con rutas canónicas y carpetas
con espacios en Java 8, 17, 21 y 25; rechazo seguro de JAR ausente. Prueba optativa
`hud_startup_windows_verbatim_paths_with_real_jvms`, mediante `TP_HUD_TEST_JAVAS`.
El diagnóstico español reconoce también el error de carga de agentes.

## Resultado de las comprobaciones de esta tanda

- Rust: 57 pruebas ordinarias aprobadas; 11 pruebas de integración se excluyen por
  defecto. Se ejecutaron además las dos de mappings y arranque normalizado.
- El puente y sus fixtures pasaron con Java 8, 17, 21 y 25 reales, sin abrir sesiones
  de Minecraft ni usar credenciales.
- 122 indicadores tienen pruebas de unidades, ausencia de dato frente a cero,
  chunks negativos, restricciones y separación de menú/mundo. Los 13 registros
  de integraciones visuales se comprueban también sin descargar binarios.
- Consulta pública real de versiones de los 13 efectos: 5 compatibles con Forge
  1.8.9, 11 con Fabric 1.21.1, 9 con Fabric 1.21.9 y 11 con Fabric 26.2. Estas cifras
  son una foto del catálogo, no una promesa futura ni una prueba de juego.
- npm run check:ui conserva una falla preexistente por traducciones incompletas de
  pt-BR; no se desactivó la aserción. El build conserva avisos de tamaño de chunks.
- La vista de prueba temporal fue eliminada. No se abrió otro navegador para
  eludir el rechazo de permisos de la revisión visual local.
