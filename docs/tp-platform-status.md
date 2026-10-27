# TP LAUNCHER y TP API — estado verificado

25 de octubre de 2026, America/Santiago.

## Infraestructura activa

- TP API pública: https://tp-api-hazel.vercel.app/api/v1/servers
- Salud: https://tp-api-hazel.vercel.app/api/v1/health
- Supabase `tp-api`, organización tomalapala, región São Paulo.
- 523 servidores importados, sin destacados ni acuerdos comerciales ficticios.
- Catálogo de 12 entradas por página: filtros, MOTD con estilos, iconos y jugadores.
- Worker SLP remoto, leases, cuotas, claves hash y protección de direcciones privadas.
- Cron cada cinco minutos desde Supabase; respaldo diario desde Vercel Hobby.
- Variables privadas en Vercel y secreto de cron cifrado en Supabase Vault.
- RLS en todas las tablas; claves y contadores bloqueados a clientes públicos.

Los intervalos de cinco/treinta minutos son objetivos por servidor; con el lote
actual de 48 filas la cola puede retrasarlos. El sistema permite workers concurrentes
sin reclamar la misma fila. No se ha contratado una ampliación de pago.

## Launcher

- Identidad End, SVG original e iconos de Windows derivados del SVG; GPLv3 y aviso de independencia.
- Directorio TP API con respaldo local y mcsrvstat; publicación, ping TCP manual y entrada rápida.
- Asesor de memoria basado en archivos de mods activos; estimación, sin promesa de ausencia de lag.
- Enlaces oficiales de BisectHosting, Apex y HolyHosting. Los anuncios reales siguen configurándose
  en `src/lib/data/community_campaigns.json`; no hay afiliados ni descuentos TP activos.
- Perfil social local por identidad, capa original TP, alas y aura como previsualizaciones.
- Estado de juego por eventos nativos. Noticias, promociones, personaje animado, partículas y
  catálogo de servidores se desmontan durante preparación/partida; se cancelan consultas nativas.
- La consola y el monitor de proceso siguen funcionando. No se promete 0% de CPU/RAM del launcher.
- Discord Rich Presence opcional con versión y servidor cuando se configura una aplicación Discord.

## Plugins y complemento

Actualización posterior: el [HUD híbrido universal](universal-hud.md) comparte editor,
WASD/CPS, reloj y PresentMon entre cargadores/versiones en ventana o sin bordes Windows.
El overlay nativo ya no es solamente un panel de RAM: sigue al juego y permite editar
con Shift derecho/F8. Los datos internos exactos y efectos del motor conservan la
compatibilidad del adaptador Fabric 1.21.1 descrito a continuación.

El editor permite guardar 15 módulos y abrir el overlay nativo opcional de RAM/estado.
Los FPS no se inventan en ese overlay. TP Companion aporta los datos reales dentro
del juego para **Minecraft 1.21.1 + Fabric**: FPS, coordenadas, dirección, armadura,
teclas, ping, reloj, minimapa de superficie, fuente uniforme del HUD, menú transparente,
marcas horarias del chat y zoom con C. Se instala únicamente con consentimiento
mediante el interruptor de Plugins y al preparar una instancia compatible.

Los optimizadores Sodium/Lithium/FerriteCore se resuelven por versión y cargador
usando el instalador existente. No se inyectan archivos incompatibles por fuerza.
Los cosméticos compartidos dentro del juego requieren una integración posterior;
no se presentan como capas oficiales ni son visibles para otros jugadores todavía.

Compilar el instalador requiere JDK 21 o superior con `javac`. `npm run tauri build`
ejecuta `build:desktop`, genera el JAR con el wrapper verificado e incluye solo
nuestro JAR, no los binarios Minecraft descargados durante el desarrollo.

## Verificación y límites

- `npm run check`: cero errores y advertencias Svelte/TypeScript.
- `cargo check`: correcto sin advertencias de código tras retirar el helper DRP antiguo.
- `npm test --prefix tp-api`: siete pruebas correctas.
- Gradle: build, remapeo del JAR y resolución de mixins correctos con Java 21.
- API pública: salud, paginación, filtros PvP y ES/LATAM, MOTD e iconos, 401 para
  clave inválida y worker sin autorización, 422 para publicar una dirección privada.
- Worker remoto y cron: 200, lotes guardados correctamente. Sin errores de ejecución
  en los logs recientes consultados. Supabase solo informa RLS sin políticas en las
  dos tablas privadas, intencionadamente inaccesibles.
- Inicio nativo inspeccionado; se corrigió el ID legado `1.21.1 - Vanilla` que dejaba
  el selector vacío. Las demás pantallas y el HUD requieren una revisión visual en partida.
- Vite avisa de dos paquetes grandes; Gradle advierte sobre APIs obsoletas de sus
  plugins para una futura versión 10. Son avisos de empaquetado, no errores de compilación.
- `check:ui` comprueba correctamente la distribución del inicio, pero detecta textos
  sin traducción completa en idiomas adicionales. No se ocultó ni debilitó esa comprobación.
- No se ha generado un instalador NSIS nuevo ni se han subido cambios a GitHub en esta fase.
