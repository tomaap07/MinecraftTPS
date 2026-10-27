# Validación de TP LAUNCHER 0.1.1

## Catálogo completo

La revisión de 2026-10-27T09:25:55.761Z verificó 920 manifiestos oficiales de Minecraft, sus identificadores, mainClass, integridad SHA-1 del JSON y 1844 endpoints únicos de cliente, bibliotecas, nativos e índices de recursos. Resultado: 920 metadatos aprobados, 0 fallos de endpoints. Plataforma: Windows x64.

[Detalle CSV](manifest-and-links-0.1.1.csv) · [Resumen JSON](catalog-0.1.1.json). Reproducir con node scripts/audit-version-catalog.mjs; el informe detallado y la caché se guardan en .codex-tmp/version-audit, ignorada por Git.

Esta revisión NO ejecuta los 920 clientes, NO descarga y verifica cada recurso individual y NO garantiza compatibilidad de todos los mods con cada versión. Los arranques reales se registran por separado. Una fila pass del CSV se refiere exclusivamente a metadatos y enlaces.

## Arranques reales

La prueba A/B de Minecraft 26.3 con Fabric y el mismo conjunto de mods reprodujo el error de shaders Sampler1 con Hit Color fabric-26.3-0.1 y cargó recursos/sonido sin ese único archivo, manteniendo el puente. Los perfiles de diagnóstico están aislados y no contienen los mundos del usuario. Véase la revisión de implementación para los detalles y límites.

La [matriz real de publicación](client-startups-0.1.1.json) verificó 10 combinaciones en perfiles aislados, sin copiar mundos ni mods del usuario: Vanilla 1.12.2, 1.21.1, 26.2 y 26.3; Fabric 1.21.1, 26.2 y 26.3; NeoForge 21.1.248; Forge 1.12.2-14.23.5.2860; OptiFine 1.12.2 G5. Se observaron motor de audio, renderizado, ventana y al menos 18 segundos de estabilidad. No se jugó en mundos ni se verificó multijugador autenticado.

La primera pasada identificó un LaunchWrapper ausente en perfiles OptiFine antiguos; se corrigió para Java 8 y se repitió el arranque de OptiFine con éxito. NeoForge escribe mensajes de audio en su archivo con marcador SOUNDS: se corrigió el lector incremental de la prueba y se repitió esa prueba con éxito. Los resultados iniciales se conservaron localmente y no se convirtieron en aprobados sin repetir el cliente.
