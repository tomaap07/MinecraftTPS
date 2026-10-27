# Instalador TP LAUNCHER

El asistente usa superficies oscuras, acentos violetas y el monograma TP vigente. Incluye bienvenida, licencia GPL, destino, carpeta del menú Inicio, resumen y opción de acceso directo al Escritorio, progreso con TP Explorer y pantalla final. El desinstalador comparte el tema y mantiene sus opciones nativas de datos. La carpeta `licenses` instalada conserva la licencia del proyecto, los avisos de terceros y la licencia MIT de la plantilla.

La versión de los textos NSIS procede de `${VERSION}`; la de los gráficos se lee de `src-tauri/tauri.conf.json`. Antes de compilar una versión nueva hay que regenerar el arte.

## Origen del arte

| Archivo | Origen |
| --- | --- |
| Monograma TP | `static/logos/tp-launcher.svg`, vector original basado en la referencia del autor del proyecto |
| `tp-explorer.svg` | Astronauta geométrico original definido mediante polígonos en `scripts/prepare-installer-assets.py` |
| `sidebar.bmp` | Composición original de monograma, TP Explorer y geometría dimensional; 164×314 RGB |
| `header.bmp` | Monograma y versión; 150×57 RGB |
| `skin-install.bmp` | Panel de copia de TP Explorer; 230×460 RGB; conserva el nombre histórico de la plantilla |
| `*-preview.png` | Previsualizaciones de los mismos gráficos, sin incluirse como recursos del programa |

El generador no consulta Minecraft, AppData, perfiles, skins de usuarios, URLs de imágenes ni claves. No usa texturas de Mojang. Tauri rasteriza SVG propio y Pillow convierte a BMP, ajusta al tamaño de NSIS y dibuja los textos. Segoe UI se utiliza desde Windows para rasterizar etiquetas; no se empaquetan archivos de fuente.

## Regenerar y compilar

Para regenerar solo los gráficos del asistente: ejecutar `scripts/prepare-installer-assets.py` con Python, Pillow, Node y el CLI local de Tauri disponibles.

Para regenerar también iconos de Windows, favicon y arte TP restante:

1. `npm run tauri -- icon static/logos/tp-launcher.svg --fit contain -p 512 -o .codex-tmp/brand-raster`
2. Ejecutar `scripts/prepare-original-art.py`, que delega el arte de NSIS al generador anterior.

Compilar con `npm run tauri build`. La ruta de salida de v0.1.1 es `src-tauri/target/release/bundle/nsis/TP LAUNCHER_0.1.1_x64-setup.exe`.

La plantilla conserva sus avisos y licencia upstream en `TAURI-LICENSE-MIT.txt`. Se mantienen instalación por usuario, WebView2, migración de accesos directos, modo silencioso, `/NS` y actualización de Tauri. No se ha configurado firma digital. Un binario anterior requiere reconstrucción para contener estos cambios.
