# TP LAUNCHER: base para plugins del HUD

La pestaña **Plugins** permite diseñar una vista previa y guardar un perfil local.
Todavía no existe un complemento cliente ni se instala código dentro de Minecraft.
Los números de la vista previa son ejemplos, no telemetría de una partida.

## Próximas herramientas

El panel identifica como **Planificado** el futuro editor de menús e interfaz,
fuentes del juego, cosméticos, perfil social y galería de personajes predeterminados.
Las referencias visuales de otros clientes sirven como inspiración; no se integra
su código, marca ni servicio de cosméticos. No se enlazan redes sociales ni se cambian
pantallas del juego todavía. Estas funciones requieren desarrollar el complemento
cliente, su compatibilidad por versión y sus controles de visibilidad.

## Contrato del perfil

Archivo: `<carpeta de datos del launcher>/plugins/hud-profile.json`.
La ruta respeta la carpeta personalizada del usuario y la ubicación existente de
MinecraftTPS. Rust valida el tamaño del archivo, su versión y todos los módulos.
El guardado usa un archivo temporal único y confirmación por renombrado.

```json
{
  "schema_version": 1,
  "name": "Mi HUD",
  "scale": 1.0,
  "opacity": 0.9,
  "background": true,
  "modules": [
    {"id":"fps","enabled":true,"anchor":"top-left"},
    {"id":"coordinates","enabled":true,"anchor":"top-left"},
    {"id":"keystrokes","enabled":true,"anchor":"top-right"},
    {"id":"armor","enabled":true,"anchor":"bottom-right"},
    {"id":"clock","enabled":false,"anchor":"bottom-left"},
    {"id":"ping","enabled":false,"anchor":"top-right"}
  ]
}
```

Se requieren los seis identificadores únicos. Anclajes disponibles: `top-left`,
`top-center`, `top-right`, `bottom-left`, `bottom-center`, `bottom-right`.
Escala: 0.75–1.5. Opacidad: 0.35–1.0. Nombre: 1–60 caracteres sin controles.

## Puntos de integración

- `launcher_plugins.rs`: lectura, validación y guardado del perfil; contrato Tauri.
- `launcher_plugins.ts`: descriptores de los módulos y tipos del editor.
- `PluginsPanel.svelte`: editor y vista previa sin depender de un proceso del juego.
- Comandos: `get_launcher_plugins`, `save_launcher_plugins`, `open_launcher_plugins_folder`.

El futuro complemento deberá leer una copia del perfil para su instancia. Las
lecturas de FPS, posición, teclas, armadura y conexión pertenecen al cliente del
juego. Los adaptadores por versión/cargador deben mantener ese detalle fuera del
editor. Antes de instalar un complemento se deberá identificar explícitamente
la compatibilidad del cargador y la versión. `companion_available` permanece en
`false` hasta que exista esa integración; guardar un perfil no la activa.
