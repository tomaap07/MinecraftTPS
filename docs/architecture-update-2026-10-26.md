# Actualización de identidad, autenticación y HUD

## Cambios implementados

- Marca: SVG del cubo TP basado en la referencia suministrada. Favicon, iconos Windows/macOS y gráficos NSIS regenerados. No se empaqueta la captura del teléfono.
- OAuth: Client ID propio `6a4f8732-6bf7-49c3-a749-16607f97281d`, navegador del sistema, redirección exacta `http://localhost:8080`, listener limitado a `127.0.0.1:8080`, PKCE S256, estado aleatorio y permisos `XboxLive.signin offline_access`. Tokens permanecen protegidos con DPAPI. El login oficial conserva su verificación de licencia. Los perfiles locales permiten pruebas con nick sin autenticar Microsoft; no se presentan como cuentas oficiales ni habilitan servidores con autenticación.
- Recursos: `get_poser_assets(version_id)` lee únicamente las rutas permitidas del JAR local de la instalación y sus padres. Si corresponde, usa índices y objetos ya instalados y verifica SHA-1. No hace peticiones HTTP, no descomprime archivos al disco y no copia imágenes del juego a `static/` ni al instalador. Límites de tamaño, dimensiones y memoria para JSON/ZIP/PNG; rutas confinadas a los datos del launcher.
- UI/Poser: utiliza los buffers de texturas locales y los actualiza al cambiar de instalación. Si no existen, conserva arte TP. Los atlas antiguos y modelos complejos sin sprite independiente conservan ese respaldo; no se simula haber extraído recursos inexistentes.
- HUD: se retiran 1% Low, frame time y contadores CPU/RAM/sistema. El preajuste mínimo contiene FPS y MS. Los perfiles previos se migran sin perder ubicación y preferencias individuales de los widgets conservados. Armadura y herramientas muestran iconos, orden por pieza y porcentaje a partir de `remaining/maximum`; sin datos no se inventa durabilidad. Zoom por C, sujeto a la capacidad real del adaptador.
- Se mantienen el manifiesto vanilla oficial (`https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`), descargas locales desde las URLs de sus metadatos, importación manual de OptiFine y captura de stdout/stderr del proceso Java en la consola del launcher.

## Client ID corregido con las capturas del registro

El ID facilitado inicialmente contenía `a740`, mientras que el registro muestra
`a749`. Tras corregirlo, la prueba sin credenciales alcanza la página de inicio
de sesión oficial de Microsoft sin `unauthorized_client`. No se cambiaron los
tipos de cuenta ni se usó un identificador ajeno. Falta que el propietario
complete el acceso para validar también Xbox Live/XSTS/Minecraft Services.
Véase [configuración Microsoft](microsoft-client-configuration.md).

## Alcance jurídico

El historial pegado no es una fuente normativa. La EULA no convierte Minecraft en código abierto; el parámetro `online-mode` es del servidor. Ni una URL pública, ni la extracción local ni un Client ID garantizan permiso o ausencia de reclamaciones. El launcher se identifica como proyecto independiente y conserva avisos de terceros. Las marcas y recursos siguen perteneciendo a sus titulares. La apariencia de una skin pública no prueba propiedad del juego. Esta actualización reduce redistribución y confusiones de afiliación; no certifica legalidad absoluta.

Fuentes primarias revisadas:

- [EULA de Minecraft](https://www.minecraft.net/en-us/eula)
- [Minecraft Usage Guidelines](https://www.minecraft.net/en-us/usage-guidelines)
- [Copyright de OptiFine](https://optifine.net/copyright)
- [Prism: instalación de OptiFine](https://prismlauncher.org/wiki/getting-started/installing-optifine/)
- [Microsoft: Authorization Code y PKCE](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow)
- [Microsoft: redirecciones loopback](https://learn.microsoft.com/en-us/entra/identity-platform/reply-url)

## Verificación

- `npm run check`: 0 errores y 0 advertencias.
- `cargo check`: correcto; `cargo test --lib`: 77 correctas, 14 pruebas explícitamente ignoradas.
- `npm run build`, `build:bridge` y `build:companion`: correctos. El build web mantiene el aviso previo de chunks grandes.
- Pruebas de HUD esencial, métricas, objetos originales y skins: correctas.
- Prueba real de extracción local: 40 texturas válidas en 1.21.1 y otras 40 en 26.2.
- Auditoría de los dos JAR TP: cero clases `net/minecraft/` y cero entradas `assets/minecraft/` empaquetadas.
- [Validación del puente y límites](hud-essentials-validation-2026-10-26.md).

No se consideran comprobadas todas las versiones reales por pasar fixtures. El chequeo heredado `check:ui` sigue detectando traducciones anteriores incompletas de pt-BR. No se han rellenado con español para ocultar ese fallo.
