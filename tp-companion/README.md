# TP Companion 0.1.0

Complemento cliente real y opcional para **Minecraft 1.21.1 + Fabric + Java 21**.
Código propio bajo GPL-3.0-only. No contiene binarios, clases, texturas ni fuentes de
Minecraft; Gradle descarga las dependencias de desarrollo en su caché local.

## Funciones implementadas

| Módulo | Fuente real de datos / comportamiento |
| --- | --- |
| FPS | Contador del cliente Minecraft |
| Coordenadas / dirección | Posición y orientación del jugador |
| Ping | Latencia del jugador en la conexión multijugador |
| Armadura | ItemStacks y durabilidad del inventario |
| Keystrokes | Keybindings de movimiento/ataque/uso activas |
| Reloj | Hora local del equipo |
| Minimapa | Superficie de 96 × 96 bloques, norte arriba, usando solo chunks recibidos; se actualiza una vez por segundo |
| Chat mejorado | Hora local en los nuevos mensajes; conserva componentes de texto y metadatos de firma |
| Fuente del HUD | Fuente uniforme ya incluida en el juego |
| Menú transparente | Omite el fondo del menú cuando hay un mundo abierto |
| Zoom óptico | Mantener C; reasignable en Controles. Solo cambia el FOV mientras está pulsado. Se desactiva si se detecta Zoomify, Ok Zoomer o WI Zoom. |

El minimapa no carga chunks nuevos, no consulta servicios externos y no muestra
entidades ocultas ni cuevas. Los cosméticos TP siguen siendo vistas previas del
launcher; este complemento no modifica skins, capas oficiales ni autenticación.

## Perfil compartido

El launcher añade el argumento JVM:

```text
-Dtp.launcher.profile=<carpeta de datos>/plugins/hud-profile.json
```

También se admite `TP_LAUNCHER_PROFILE`. Se valida el esquema 1, las posiciones, la
escala y el tamaño máximo de 64 KiB. El perfil se comprueba cada 200 ms y se relee solo si
su fecha cambió. Un archivo inválido conserva la última configuración válida.
No se realizan consultas HTTP ni se envía telemetría fuera del equipo. Se publica
telemetría local para el overlay y se solicita al servidor de Minecraft su paquete
de estadísticas una vez por mundo.

## Compilación

Se usan versiones fijadas: Gradle 9.6.1, Loom 1.17.21, Yarn 1.21.1+build.3,
Fabric Loader 0.20.1 y Fabric API 0.116.17+1.21.1. El loader mínimo del juego es 0.16.0.
Las versiones se consultaron en los metadatos oficiales de Fabric antes de compilar.

```powershell
$env:JAVA_HOME = '<ruta del JDK 21>'
.\gradlew.bat --no-daemon build
```

Salida: `build/libs/tp-companion-0.1.0.jar` (remapeada para distribución).
El wrapper de Gradle, cuando está generado, conserva su propia licencia Apache-2.0.

## Instalación desde el launcher

1. Activa **Instalar el complemento TP** en Plugins y guarda el perfil.
2. Selecciona una instalación **Fabric 1.21.1**.
3. Al preparar el inicio, el launcher instala una Fabric API compatible usando su
   gestor de contenido y copia únicamente el JAR TP verificado de su recurso local.
4. Ningún archivo ajeno al complemento se elimina. Otras versiones/cargadores se
   omiten con un mensaje de compatibilidad.

Para empaquetar el launcher desde un clon limpio hay que compilar este proyecto
antes de incluir `build/libs/tp-companion-0.1.0.jar` como recurso de Tauri.
La compilación no ejecuta Minecraft; la comprobación visual dentro de una partida
debe realizarse por separado.

## Ampliación del HUD

El mod incorpora datos de inventario y estadísticas, hitmarkers confirmados por el
servidor, combo, control de niebla, físicas visuales de ítems y comandos acotados
para packs/opciones/shaders. El menú TP conserva los textos legales de Vanilla e
incluye galería F2, selector de tema y fondo privado. Véase ../docs/universal-hud.md
para la matriz de compatibilidad y los límites. Esto no extiende el binario a otras
versiones de Fabric.
