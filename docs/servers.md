# Servidores y LAN

El panel utiliza el `servers.dat` real de la instalación activa. Las instalaciones Vanilla conservan su carpeta histórica por jugador y cada perfil con mods o modpack utiliza su propia carpeta. Cambiar de instalación en Jugar cambia también la lista que administra este panel.

## Lista de Minecraft

- Lee y escribe NBT Java sin comprimir y con cadenas Java Modified UTF-8, incluidos nombres con caracteres suplementarios.
- Conserva los campos NBT desconocidos del documento y de cada servidor: iconos, preferencias de recursos, estado oculto y metadatos futuros.
- Añade, edita, elimina y reordena servidores. La interfaz pagina las listas largas.
- Detecta modificaciones externas mediante una revisión SHA-1. Ante un cambio de Minecraft, pide actualizar y conserva el archivo existente.
- Antes de reemplazar el archivo crea `servers.dat_old`. La escritura se realiza en un temporal exclusivo con sincronización y reemplazo por `rename`, sin borrar antes el archivo original.
- Si Minecraft ya tiene abierta la lista Multijugador, sal de esa pantalla y vuelve a entrar para que relea el archivo.

## Estado de los servidores

El ping utiliza el protocolo TCP de Minecraft Java, con resolución SRV para dominios sin puerto. Admite IPv4, IPv6 y dominios con puerto explícito. Aplica límites de tiempo y tamaño a las respuestas. Para servidores anteriores a 1.7 intenta el ping clásico después del protocolo moderno.

El MOTD se representa mediante fragmentos de texto, colores y estilos; nunca mediante HTML del servidor. Las consultas se limitan a cuatro servidores simultáneos de la página visible. Un servidor sin respuesta puede seguir seleccionándose para conexión directa.

## Compartir un mundo

1. Inicia la instalación activa, abre el mundo y selecciona **Abrir en LAN** en Minecraft.
2. El launcher detecta el puerto desde el registro del proceso o desde `logs/latest.log`. También permite introducirlo manualmente.
3. Comprueba que el puerto local siga escuchando antes de ofrecer las direcciones para compartir en la red local.
4. Para una dirección pública, instala e4mc desde este panel en una instalación Fabric, Forge, NeoForge o Quilt compatible. La descarga utiliza la edición publicada para la versión exacta del juego y resuelve sus dependencias mediante el instalador del estudio de mods.
5. Reinicia Minecraft y abre el mundo en LAN. La dirección pública solo aparece cuando e4mc la publica en su registro. No se inventa un dominio ni se presenta una descarga como un túnel ya conectado.

El anfitrión debe mantener el mundo abierto. Los amigos necesitan versiones y mods compatibles. La disponibilidad del relay público depende del servicio e4mc y de la conexión del juego. La detección del dominio en el registro no es una comprobación desde otra red.

Fuentes: [e4mc y sus creadores](https://e4mc.link/), [proyecto e4mc en Modrinth](https://modrinth.com/mod/e4mc), [API oficial de versiones de Modrinth](https://docs.modrinth.com/api/operations/getprojectversions/).

## Verificación automatizada

El módulo contiene pruebas de validación de direcciones, NBT completo con metadatos desconocidos, UTF-16 suplementario, backup/reordenamiento/conflicto externo, respuestas malformadas, MOTD seguro, detección LAN y un servidor TCP de prueba que completa el intercambio real de estado y ping/pong.
