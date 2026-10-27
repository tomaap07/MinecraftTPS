# Catálogo de servidores de TP LAUNCHER

## Descubrimiento y lista personal

- **Selección TP**: las 40 direcciones aportadas por el usuario están en
  `src/lib/data/server_catalog.json`. No son patrocinios ni se escriben automáticamente
  en Minecraft. Sus idiomas, regiones y modalidades son etiquetas orientativas.
- **Catálogo público**: MC Monitor, `https://www.mc-monitor.app/developers`, mediante
  su GET público `/api/servers`. No requiere credenciales. Se consulta desde Rust.
- **Mis servidores**: conserva el lector/escritor de `servers.dat`, sus revisiones,
  respaldos, orden, edición y detección LAN/e4mc. Añadir desde el catálogo usa el
  mismo guardado y no reemplaza las entradas existentes.

El directorio público tiene paginación numerada y salto a cualquier página, búsqueda,
modalidad, versión y orden. No hay un registro universal de todos los servidores.
Las cantidades son las publicadas por el proveedor, no una garantía de disponibilidad,
compatibilidad ni acceso. Los servidores marcados como exclusivamente Bedrock no
pueden iniciarse con este launcher Java.

Al abrir una página se consultan sus servidores Java directamente y se actualizan cada
15 segundos: jugadores, plazas, latencia, versión, icono y mensaje (MOTD) con colores.
Antes de recibir esa consulta, las tarjetas públicas identifican el dato del proveedor.
Se muestra la hora de la medición y un botón permite actualizar manualmente.
El mismo monitor se utiliza en Mis servidores. El icono recibido se conserva al añadir
un servidor a la lista personal.

Solo se consulta la página visible, con cuatro solicitudes simultáneas como máximo.
El monitor se detiene al cambiar de sección, pausa nuevas solicitudes si la ventana
queda oculta o se pierde la conexión y descarta la cola de las páginas abandonadas.
El backend comparte una caché de 10 segundos y agrupa solicitudes duplicadas.
Ante un fallo temporal se mantienen el último icono y mensaje, indicando su antigüedad;
un fallo de transporte no se presenta como una medición de cero jugadores.
No se escanean redes ni se consultan automáticamente todos los servidores del catálogo.

## Caché y límites

`server_directory.rs` valida las direcciones y normaliza los dos formatos de etiquetas
del proveedor (texto u objeto). Solo realiza consultas al endpoint HTTPS fijo, con
timeout, límite de 4 MiB por respuesta y un máximo local de 45 consultas por minuto.
Las páginas se reutilizan durante 60 segundos en memoria y disco. Si falla la conexión,
se recupera la página guardada, identificada como copia anterior; la selección local
permanece accesible. La búsqueda tiene debounce y descarta respuestas obsoletas.

## Espacios de patrocinio

`CommunitySpotlight.svelte` está colocado al final de Servidores, en el flujo normal:
sin superponer el juego, reproducción automática, popups ni scripts publicitarios.
Su configuración `src/lib/data/community_campaigns.json` está **vacía**. No se muestran
anuncios, no se hacen peticiones publicitarias y no se afirma tener socios comerciales.

Cuando exista una campaña acordada, puede añadirse una entrada de este formato:

```json
{
  "id": "identificador-de-campana",
  "active": false,
  "topic": "minecraft",
  "placement": "servers",
  "sponsor": "Nombre real del patrocinador",
  "title": "Título de la campaña",
  "description": "Descripción breve",
  "url": "https://example.com/",
  "ends_at": "2027-01-01T00:00:00Z"
}
```

Se muestra una campaña activa cada vez, siempre identificada como contenido patrocinado.
El jugador puede ocultarla durante la visita. Los enlaces se abren solo al pulsar el
botón y deben usar HTTPS. Las ubicaciones `home` y `community` se reservan para
integraciones futuras; actualmente solo está montado el espacio de Servidores.
