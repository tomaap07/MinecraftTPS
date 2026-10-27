# Steve/Alex y perfiles públicos

La apariencia predeterminada del launcher vuelve a ser Steve clásico. Alex se
utiliza cuando se solicita el modelo slim. La descarga de estas texturas se hace
en ejecución; no se añadieron sus PNG al repositorio ni al instalador.

Las plantillas clásicas se consultan primero en Mojang. Para perfiles de jugador,
se resuelve el nombre a UUID y se consulta la textura pública de Mojang. Crafatar
es un servicio independiente de respaldo, no una API oficial de Mojang. Su API
acepta UUID y rechaza nombres de usuario, según su
[documentación](https://crafatar.com/).

Los defaults descargados y validados se conservan en la caché privada de datos
del equipo. Con conexión se pueden actualizar; sin conexión se reutiliza la
última copia válida. Si el primer arranque ocurre sin red y sin esa copia, se
utiliza el personaje original local como último recurso: no es posible obtener
una textura remota en un equipo completamente desconectado.

El visor mantiene el modelo clásico/slim correcto, superficies interiores
opacas y ropa con transparencia recortada. Una sustitución por un default
genérico no conserva una capa residual de otra apariencia.

Estas consultas únicamente cambian vistas previas y avatares. No crean cuentas
Microsoft, no acreditan una licencia, no autentican al usuario como el titular
del UUID y no modifican las restricciones del modo Offline. La existencia de un
endpoint público tampoco concede por sí sola permisos sobre una skin o marca.
