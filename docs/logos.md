# Identidad visual de proveedores

Fabric, Forge, OptiFine, Modrinth y CurseForge conservan sus logos oficiales para identificar filtros e integraciones. Forge utiliza únicamente el trazado oficial del yunque sobre una caja cuadrada de 16x16 en la barra de versiones. `ServiceLogo`, `LoaderLogo` y `ProviderLogo` comparten los recursos de `brand_assets.ts`, incluidos los filtros de Mods y modpacks y Ambos · unificado.

Las cuatro texturas de Mojang del filtro VERSIÓN siguen sustituidas por `TpPixelIcon.svelte`. Los nombres, filtros y acciones no se alteran. Las fuentes, SHA-256 y modificaciones de encuadre/color están en `static/logos/sources.json`. Las condiciones investigadas y sus límites están en [asset-policy-audit.md](asset-policy-audit.md). El uso de una marca ajena no la convierte en arte propio ni implica afiliación.

# Metadatos de mods y portadas

Los mods instalados se leen localmente como archivos ZIP, sin ejecutar código Java. Se admite `fabric.mod.json`, `quilt.mod.json`, `META-INF/neoforge.mods.toml`, `META-INF/mods.toml` y `mcmod.info`. Se muestran nombre, versión, autores, descripción e icono publicado dentro del propio mod. Si no existe metadato o icono, se conserva el nombre de archivo y un símbolo neutro, sin inventar la identidad de un proyecto.

Cada entrada JSON/TOML tiene un límite de 1 MiB y las imágenes de 2 MiB. La decodificación tiene límites de dimensiones y memoria, y los iconos se convierten a PNG de hasta 96 px. El caché en `cache/mod-metadata` se invalida con ruta, tamaño y fecha de modificación del JAR. No se extraen archivos del JAR al disco.

Al instalar un modpack desde el catálogo se conserva su `icon_url` oficial en `tps-pack.json` (manifiesto de instancia ya existente) y una copia PNG de hasta 128 px en `icon.png`/`icon_data`. Las fuentes remotas se restringen a los CDN de Modrinth y ForgeCDN. La importación local conserva `icon_data`, `icon_url` o `icon.png` cuando el archivo los aporta. Para un `.mrpack` sin portada, una consulta pública opcional identifica el archivo por SHA-1 en Modrinth y obtiene la portada de su proyecto; la consulta tiene un plazo de 10 segundos. Los formatos estándar no garantizan una portada dentro del ZIP y un archivo desconocido permanece con el símbolo de su motor.

Blockbench: restaurado el logo oficial sin modificar para crédito y enlace conforme
a https://blockbench.net/wiki/blockbench/logos/. Fuente y hash en sources.json.
