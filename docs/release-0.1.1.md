# TP LAUNCHER 0.1.1 — versión de prueba

## Cambios de esta entrega

- Se bloquea la compilación concreta de Hit Color que provoca el fallo de shaders de Minecraft 26.3. La comprobación utiliza versión, cargador y huella; conserva las preferencias y pausa solo el archivo gestionado identificado.
- Los widgets del HUD permanecen dentro de la ventana al cambiar resolución o escala, sin sobrescribir la posición guardada.
- Los perfiles OptiFine antiguos recuperan LaunchWrapper cuando lo necesitan. La dependencia oficial se comprueba por tamaño y SHA-1 y no se duplica si el perfil ya tiene su propio wrapper.
- El instalador NSIS usa el monograma TP actual y el personaje original TP Explorer, bienvenida y progreso personalizados, selección de accesos directos y licencia GPL. Incluye los avisos de terceros en `licenses` y WebView2 offline. No incorpora cuentas, claves, mundos ni un cliente de Minecraft.
- Se retiraron 606 comentarios ordinarios con comprobaciones sintácticas. Se conservaron avisos de licencia y los wrappers de Gradle de terceros.
- Se completaron 106 entradas pendientes del catálogo de interfaz en diez idiomas y sus dos variantes derivadas. La revisión cubre 751 textos y 16 idiomas.

## Comprobaciones

| Comprobación | Resultado |
| --- | --- |
| `npm run check` | 0 errores, 0 advertencias |
| `npm run check:ui` | 751 textos, 16 idiomas |
| Rust, pruebas ordinarias | 98 aprobadas, 17 omitidas por requisitos externos |
| Scripts de pruebas de la aplicación | 10 suites aprobadas |
| TP API | 7 pruebas aprobadas |
| TP Bridge y TP Companion | Compilación aprobada |
| Arranques reales aislados | 10 combinaciones aprobadas tras corregir y repetir los casos señalados |
| Prueba real A/B del fallo 26.3 | Reproducido con el archivo afectado y resuelto al pausarlo |
| Catálogo oficial | 920 manifiestos y 1.844 endpoints comprobados |
| `npm audit --omit=dev` | 0 vulnerabilidades comunicadas |

La auditoría completa de npm informa tres avisos de severidad baja en herramientas de desarrollo de SvelteKit. La corrección propuesta requiere un cambio mayor de esa cadena; no se aplicó una migración de framework durante el empaquetado. La aplicación de escritorio usa salida estática.

Los arranques comprueban proceso, ventana, renderizado y audio inicializados, con un periodo de estabilidad de al menos 18 segundos. No acreditan gameplay, servidores autenticados ni todos los módulos en todas las versiones. La comprobación del catálogo no equivale a ejecutar sus 920 clientes. [Resultados y alcance](validation/README.md).

## Distribución

El instalador no está firmado digitalmente. La revisión visual recorre bienvenida, licencia, destino, menú Inicio y resumen; no ejecuta una instalación sobre el equipo del autor. El informe del contenido extraído y su SHA-256 están en [la validación del instalador](validation/installer-payload-0.1.1.json).

Por indicación del autor, esta entrega se publica como versión de prueba en el repositorio privado vinculado, con el alcance de validación descrito. No se afirma que cada cliente del catálogo haya sido ejecutado. El instalador se distribuye como asset de [la publicación v0.1.1](https://github.com/tomaap07/MinecraftTPS/releases/tag/v0.1.1), acompañado de sus sumas SHA-256; no se incluye como binario dentro del historial de fuentes.
