# TP Cloud: staging de comunidad, identidad y autorización

Implementación local comprobada el 2 de noviembre de 2026. **No se aplicó esta migración al proyecto remoto, no se desplegó el servicio y no hay una red social ni un túnel P2P operativos en producción.** El proyecto remoto inspeccionado conserva únicamente su directorio de servidores. Este trabajo prepara un backend independiente y verificable; no sustituye la autenticación Microsoft ni recibe sus tokens.

## Qué está implementado y probado

- Migración idempotente `supabase/migrations/20261102181706_tp_cloud_staging.sql`, creada mediante la CLI de Supabase. Esquema privado para roles, validaciones de PNG, auditoría, eventos de misiones y códigos de sala. Tablas públicas con RLS, permisos explícitos y operaciones de escritura mediante un contrato limitado.
- Identidad TP: cuenta Supabase Auth con correo confirmado y sin sesión anónima. El ID del actor se obtiene de la sesión verificada, no del formulario. Un nombre, UUID de Minecraft o `user_metadata.role` no concede autenticación o administración.
- Solicitudes de amistad, aceptación exclusiva del destinatario, rechazo, bloqueo bidireccional y cancelación de invitaciones al bloquear. No hay chat privado, voz, presencia en tiempo real o notificaciones remotas implementadas todavía.
- Subidas de capas: PNG real con CRC, límite de 1 MiB, dimensiones 64×32, 128×64, 256×128 o 512×256; se decodifica y se vuelve a codificar RGBA, retirando metadatos. La carga entra en un bucket **privado**. Solo el servidor emite el comprobante de validación, asociado al autor, de un uso y vigente una hora. Un fallo al registrar el comprobante intenta eliminar el archivo huérfano.
- Cola de moderación: máximo tres pendientes por autor; únicamente el rol `owner` registrado por un administrador de base de datos puede aprobar, rechazar o retirar, con motivo e historial. Un moderador declarado no obtiene facultades automáticamente. El propietario puede previsualizar imágenes privadas; otros usuarios no ven pendientes ajenos. Una retirada impide nuevas lecturas mediante las políticas. Las copias que un usuario ya haya descargado no pueden retirarse de su equipo.
- Tres derechos de bienvenida gratuitos e idempotentes por perfil TP. Las capas oficiales de Minecraft no se alteran. Los identificadores de recompensas de misiones existen como datos; no implican que todos sus modelos finales estén producidos ni conectados al renderizador del mod.
- Claims de misiones únicos por usuario y misión. Amistad cuenta relaciones aceptadas distintas. La misión de artista requiere una presentación de PNG validada; no basta un evento local del editor. Tiempo activo y sesiones requieren eventos del servidor, con ID de evento único. **El productor de eventos de juego verificados no está implementado ni habilitado**: la API pública no acepta `complete=true` ni expone la función que registra esos eventos.
- Salas privadas con propietario, versión/cargador, caducidad, códigos aleatorios almacenados por hash, destinatario concreto, uso único y revocación. Crear o aceptar una sala devuelve explícitamente `transport: "not_configured"`; autorizar una invitación **no establece una conexión de Minecraft**.
- Partners: alta o cambio solo por el propietario, URL HTTPS y evidencia privada obligatoria. El catálogo público no expone el documento de autorización. Ningún proveedor ha sido registrado como socio real por esta tarea y no existe atribución de conversiones o cobro.

## Contratos disponibles

`TP_CLOUD_ENABLED` es `false` por defecto; si falta configuración, los endpoints responden 503 y no aparentan éxito. Las variables se documentan sin valores secretos en `tp-api/.env.example`. La clave de servicio solo se utiliza dentro del servidor para validar subidas. Los comandos de usuario utilizan su propio bearer y la clave publicable, sujetos a RLS.

### HTTP

- `POST /api/v1/community/capes`: `Authorization: Bearer <sesión TP>` y JSON `{ "pngBase64": "..." }`. Devuelve un comprobante `private_validated`, no una publicación. El endpoint verifica la sesión en `/auth/v1/user`; no confía en un JWT simplemente decodificado.
- `POST /api/v1/community/commands`: el mismo bearer y JSON `{ "action": "...", "args": {} }`, hasta 4 KiB. Acciones permitidas: `profile`, `capabilities`, `friend_request`, `friend_response`, `block`, `submit_cape`, `moderate`, `report`, `claim`, `partner`, `room_create`, `room_invite`, `room_join`, `room_close`.
- `GET /api/v1/community/commands?resource=gallery&page=1`: catálogo de aprobadas no retiradas. También existe `partners`, limitado a proveedores activos. No necesita una cuenta TP para esos dos recursos.
- Los recursos `profiles`, `friends`, `submissions`, `reports`, `inventory`, `quests`, `rooms` y `members` necesitan una sesión TP verificada. Devuelven columnas explícitas y filas autorizadas por RLS, páginas de 24 elementos y `hasMore`. No hay lecturas arbitrarias de tablas o columnas privadas.

Los orígenes de navegador requieren inclusión explícita en `TP_CLOUD_ALLOWED_ORIGINS`. No se utiliza autenticación por cookies. Las respuestas privadas no tienen caché; los errores no devuelven tokens, claves de servicio ni cuerpos privados del proveedor. Las consultas públicas y la verificación Auth todavía necesitan límites de borde configurados para un despliegue público; los límites SQL de amistad, salas y comprobantes no sustituyen un control global de abuso.

### Ejemplos de RPC autenticados

```json
{"action":"profile","args":{"handle":"mi_nombre_tp","displayName":"Mi nombre"}}
{"action":"friend_request","args":{"recipient":"UUID de la cuenta TP destinataria"}}
{"action":"friend_response","args":{"request":"UUID de la solicitud","accept":true}}
{"action":"submit_cape","args":{"ticket":"UUID del comprobante privado","title":"Mi capa original","rightsAccepted":true}}
{"action":"moderate","args":{"submission":"UUID de la presentación","decision":"approved","reason":"Procedencia y diseño revisados"}}
{"action":"room_create","args":{"game":"1.21.1","loader":"fabric"}}
{"action":"room_invite","args":{"room":"UUID de sala","recipient":"UUID de amigo aceptado"}}
{"action":"room_join","args":{"code":"código privado de un uso","game":"1.21.1","loader":"fabric"}}
```

No enviar códigos de sala a incidencias o logs públicos. La ausencia de versión/cargador o una discrepancia al aceptar se rechazan; no se permite que `null` eluda el chequeo de compatibilidad.

## Activación de staging: acciones pendientes

1. Autorizar y seleccionar un proyecto **de staging**, independiente del directorio público, y configurar Supabase Auth con confirmación de correo, política de privacidad, retención, cuentas de prueba y límites de registro. Falta la interfaz de identidad TP en el launcher; la sesión Microsoft no debe reutilizarse como sesión TP.
2. Aplicar la migración solo al proyecto seleccionado, revisar los asesores y las políticas de Storage, probar el bucket privado y verificar usuarios reales A/B/propietario en ese entorno. Los tests PGlite usan PostgreSQL con roles y RLS, pero una emulación mínima de `auth.users` y Storage: no son una prueba del servicio remoto S3 ni del envío de correo.
3. Con el UUID **de la cuenta TP confirmada** del propietario, un administrador de base de datos debe insertar su fila `owner` en `tp_private.admin_roles`. No concederlo comparando emails en JS ni abriendo una API que permita autoasignar roles. No hay propietario precargado en la migración.
4. Configurar las variables privadas del servidor en el entorno aprobado, desplegar allí los endpoints y fijar orígenes y límites de borde. No poner `TP_CLOUD_SERVICE_ROLE_KEY` en Tauri, en `VITE_*`, en el mod ni en GitHub. Dar lectura al esquema público en Data API y mantener `tp_private` fuera de los esquemas expuestos.
5. Conectar la galería/admin y el login TP del launcher a esos contratos. El visor 3D y el editor local de capas siguen siendo funciones independientes: no publican nada por guardar un PNG.
6. Antes de habilitar salas entre redes, implementar señalización autenticada y un transporte TCP cifrado, consentimiento, límites de relay con presupuesto inicial cero y pruebas NAT entre dos redes. La base de datos de invitaciones no equivale al túnel.
7. Antes de habilitar recompensas de tiempo/sesiones, definir el productor confiable de eventos. Nunca otorgar la clave de servicio al cliente para que firme su propio progreso.

No se han creado claves privadas nuevas ni se han modificado las variables o los permisos de producción.

## Pruebas ejecutadas

En `tp-api`, `npm test` ejecutó TypeScript y Node Test: **29 pruebas aprobadas**, incluidas las subpruebas de PostgreSQL/PGlite. Comprueban acceso cruzado, sesión no confirmada/anónima, falsificación de rol, privacidad de pendientes, previsualización del propietario, retirada, comprobantes privados y de un uso, replay de eventos, claim idempotente, restricción de destinatario, versión obligatoria, cierre/bloqueo de salas, documentos privados de partners, segunda aplicación de la migración y eliminación de cuenta sin bloqueo por claves foráneas de autoría/moderación. Esta última es una prueba de integridad de datos, no una interfaz de borrado o una política completa de retención. Las pruebas HTTP usan fixtures de red únicamente dentro de los tests; las funciones de producción realizan peticiones reales cuando se configuran.

El primer pase detectó incompatibilidad de tipos en la codificación PNG; se corrigió usando una instancia `PNG` real. También se corrigieron una falta de política de previsualización para el propietario y una comparación SQL con `NULL` que permitía aceptar una invitación sin declarar versión/cargador.

## Recuperación y rollback

- La migración es aditiva respecto del catálogo de servidores y no lo borra ni modifica sus datos. Se recomienda snapshot del staging antes de aplicarla. La segunda aplicación es parte de los tests.
- Para detener la superficie HTTP, poner `TP_CLOUD_ENABLED=false` y retirar el despliegue de staging. Esto conserva los datos para investigación y exportación. No eliminar tablas, cuentas o imágenes automáticamente como "rollback".
- Antes de retirar definitivamente el esquema, exportar por separado las tablas `tp_cloud_*`, el esquema `tp_private` y el bucket privado, y revisar las referencias. La eliminación permanente requiere autorización explícita; no se proporciona una instrucción destructiva automática.
- El bucket permanece privado. Las políticas no otorgan subida, cambio de estado, entitlements o roles directamente a `anon`/`authenticated`.

## Fuentes técnicas

- [Supabase: Row Level Security](https://supabase.com/docs/guides/database/postgres/row-level-security)
- [Supabase: buckets privados y públicos](https://supabase.com/docs/guides/storage/buckets/fundamentals)
- [Supabase: securizar Data API](https://supabase.com/docs/guides/api/securing-your-api)
- [Changelog de Supabase](https://supabase.com/changelog): se revisó la retirada anunciada de cifrados antiguos TLS 1.2 para el 10 de noviembre de 2026; este servidor usa Node 22+ y TLS moderno.

Esta cohorte no constituye un servicio de producción terminado ni una garantía de cumplimiento legal. Mantiene separados los cosméticos TP y los derechos oficiales de Minecraft.
