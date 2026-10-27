# TP API

API pública de directorio y monitor Minecraft Java para TP LAUNCHER. Node.js 22+
con TypeScript, Vercel Functions y Supabase Postgres. El runtime utiliza `fetch`,
`net` y `dns` de Node; no necesita una biblioteca Minecraft o cliente Supabase.

## Preparación

1. Vincular este subdirectorio a un proyecto Vercel y elegir el proyecto Supabase.
2. Configurar las variables privadas indicadas en `.env.example` en Vercel.
   Usar `SUPABASE_SERVICE_ROLE_KEY` **o** el puente privado de Edge:
   `SUPABASE_ANON_KEY`, `TP_DB_BRIDGE_URL` y `TP_DB_BRIDGE_TOKEN`.
3. Aplicar la migración de `../supabase/migrations/` al proyecto confirmado y
   comprobar sus políticas y funciones antes de sembrar datos.
4. `npm ci`, `npm run build`, `npm test`.
5. Para administración local, obtener las variables en `.env.local` mediante el
   mecanismo autorizado del proveedor. No imprimir ni subir el archivo.

Las claves de servicio, el token del puente, `TP_MASTER_SECRET` y `CRON_SECRET`
permanecen exclusivamente en el servidor. La salud pública solo muestra si la
configuración está completa; nunca devuelve valores privados.

## Contrato HTTP

### `GET /api/v1/servers`

Parámetros: `page` (1–10000), `limit` (1–48, predeterminado 12), `category`, `lang`,
`region` (`all` para no filtrar), `search`, `version`.

```json
{
  "servers": [], "total": 0, "page": 1, "limit": 12, "pages": 1,
  "source": "TP API"
}
```

Cada entrada expone únicamente los campos públicos: `id`, `name`, `ip`, `port`,
`category`, `region`, `language`, `version_text`, `is_online`, `players_online`,
`players_max`, `motd_clean`, `motd`, `favicon_url`, `latency_ms`, `protocol`,
`is_featured`, `featured_until`, `sponsor_url`, `last_pinged_at`, `created_at`.
`motd` contiene spans con `text`, `color`, `bold`, `italic`, `underlined` y
`strikethrough`. El favicon es un PNG 64×64 en una URL de datos, validado y limitado.
La latencia de esta respuesta se mide desde el worker, no desde el PC del jugador.

Orden: destacados con fecha vigente, en línea, jugadores e ID estable. Los
destacados expirados dejan de priorizarse incluso antes de la siguiente ejecución
del worker. Las entradas iniciales no se convierten en anuncios ni destacados.

La cabecera `X-Client-ID: tp-launcher-official` identifica al launcher; **no es una
credencial ni una excepción a límites**. Cada origen tiene un límite de consultas
por IP usando un HMAC privado. Las respuestas anónimas usan caché CDN de 60 s y
`stale-while-revalidate=300`; el límite de aplicación se ejecuta en misses del CDN.
Para imponer límites a cada hit de borde debe añadirse una regla de Vercel Firewall.
`Vary: X-API-Key` separa variantes; las peticiones con esa cabecera no se cachean y
consumen su cuota diaria de manera atómica. Una API Key opcional inválida devuelve
401, una cuota agotada 429; no se entrega privilegio de escritura por usar un tier.

### `POST /api/v1/servers/submit`

JSON `{ "name": "Mi servidor", "ip": "play.example.net", "port": 25565,
"category": "Survival", "region": "LATAM", "language": "ES" }`.
Antes de insertar se valida DNS público y se realiza una consulta SLP real. La
IP resuelta se fija al abrir el socket; se rechazan loopback, LAN, link-local,
redes reservadas y direcciones IPv6 mapeadas/túnel. Hay límites de tiempo y tamaño,
cinco intentos por minuto por IP y un formulario máximo de 4 KiB.

La respuesta es `{ "server": { ... }, "created": true }`, con HTTP 201. Una
dirección ya publicada devuelve su entrada existente, no permite que otra persona
cambie su nombre, metadatos, patrocinio o posición.

### `GET|POST /api/v1/worker/sync`

Requiere `Authorization: Bearer <TP_MASTER_SECRET o CRON_SECRET>`. Nunca colocar
esa credencial en el launcher. Expira destacados, limpia contadores antiguos y
reclama filas con `FOR UPDATE SKIP LOCKED` y leases de tres minutos. Los resultados
solo se aplican cuando coincide el token de la reclamación.

Planificador: VIP vigente o más de 50 jugadores, 5 minutos; activos, 30 minutos;
cinco fallos consecutivos, 12 horas. El worker reclama hasta 96 entradas, ejecuta
ocho sockets a la vez de forma predeterminada y deja pendientes los que excedan su
presupuesto de tiempo. Las filas pendientes quedan disponibles tras vencer el lease.

**Los intervalos son objetivos de planificación, no una garantía de frescura.** La
capacidad depende del número de workers, frecuencia, presupuesto y servidores lentos.
No se promete actualizar miles de hosts cada cinco minutos en un solo lote.

### `GET /api/v1/health`

Devuelve `configured: true|false` y versión. Un 200 confirma configuración, no
sustituye una consulta real a `/servers` ni una auditoría de la base de datos.

## Cron y plan de alojamiento

Despliegue activo: https://tp-api-hazel.vercel.app. Supabase: proyecto
`aucsjsaviekxhujwputl`, región São Paulo. El 25 de octubre de 2026 se importaron
523 direcciones únicas y se completó su primera sincronización SLP.

Se autorizó y activó Supabase Cron cada cinco minutos (`tp-api-smart-polling`),
con CRON_SECRET cifrado en Vault y un POST autenticado al worker de Vercel.
La migración solo contiene el nombre del secreto, nunca su valor. Se verificaron
respuestas 200 con 48 actualizaciones por ejecución después de aislar pg_net en
el esquema `extensions`. Los objetivos de frescura dependen de la cola y capacidad.

Como respaldo, `vercel.json` utiliza una ejecución diaria compatible con Hobby para evitar exigir
un plan de pago. **Ese cron diario no cumple objetivos de 5/30 minutos**. Tras
autorizar un plan compatible puede usarse `vercel.pro.example.json` como referencia
para ejecutar cada cinco minutos, o un programador externo autenticado con la
misma ruta privada. No se cambia la suscripción ni se activa facturación automática.

Vercel envía `CRON_SECRET` como Bearer para sus trabajos programados. Un cron que
se solape recibe otro lote gracias a los leases. La configuración privada debe
incluir un secreto aleatorio de al menos 24 caracteres; no usar el nombre público
del cliente como secreto.

## Semilla e importaciones

`npm run seed` carga las 40 direcciones aportadas para el launcher; no las marca
como verificadas en línea antes de obtener respuesta SLP ni crea patrocinios.

```text
npm run seed -- --sync --sync-batches=2
npm run seed -- --import-directory --sync --sync-batches=24
npm run seed -- --import-url=https://raw.githubusercontent.com/OWNER/REPO/main/servers.json
```

`--import-public` requiere indicar `--import-url` con un JSON de repositorio
público verificado. No se presupone una fuente de GitHub que pueda no existir.
`--import-directory` añade hasta 500 direcciones de la API pública documentada de
[MC Monitor](https://www.mc-monitor.app/developers). No se escanean rangos de IP ni
se scrapean páginas. Las fuentes externas varían en cantidad y disponibilidad.
Los archivos se limitan a 5 MiB/2000 entradas, solo HTTPS en orígenes conocidos,
sin redirecciones; las direcciones se deduplican y pasan la misma política SLP al
sincronizar. Los importadores no ejecutan contenido de las fuentes.

## API Keys para desarrolladores

```text
npm run keys:create -- --owner=Desarrollador --tier=free --limit=5000
```

Solo se guarda SHA-256 de la API Key en la base de datos. El valor original se
escribe una vez en `.local/api-keys/<id>.txt` y se anuncia la ruta, nunca el valor.
La carpeta está excluida de Git. Revocar una clave requiere desactivar su fila
desde un contexto administrativo autorizado.

## Comprobaciones incluidas

`npm test` comprueba direcciones/puertos, política SSRF IPv4/IPv6, VarInt, mensajes
de Minecraft y límites de favicon/contadores sin abrir conexiones externas. La
migración y las rutas desplegadas necesitan además verificarse contra el proyecto
real con los permisos correctos. Mantener habilitado RLS y sin grants públicos en
`tp_api_keys`, `tp_rate_limits` y las funciones administrativas.

Referencias: [RLS Supabase](https://supabase.com/docs/guides/database/postgres/row-level-security),
[Cron de Vercel](https://vercel.com/docs/cron-jobs/manage-cron-jobs),
[caché CDN y Vary](https://vercel.com/docs/caching/cdn-cache).
