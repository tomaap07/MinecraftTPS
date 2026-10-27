# Traducción local y contenido público

La interfaz utiliza catálogos locales y persiste el idioma elegido en los ajustes. En el primer inicio obtiene el idioma de Windows mediante GetUserDefaultLocaleName, sin PowerShell. La ausencia de una clave muestra el texto base y no bloquea la interfaz.

Las descripciones públicas en inglés de proyectos y noticias pueden traducirse con el botón Traducir. La traducción automática es opcional y está desactivada por defecto. Se indica MyMemory cuando una respuesta ha sido traducida. No se envían contraseñas, sesiones ni registros de juego a ese servicio.

La API gratuita tiene cuota: para acceso anónimo se presupuestan 5.000 caracteres diarios por dispositivo. El proveedor también puede limitar por IP. Se conservan hasta 500 traducciones durante 30 días en translation-cache.json, con claves hash del texto y del idioma. Solicitudes fallidas, lentas, sin conexión o sin cuota muestran el original. La caché no convierte contenido externo en una traducción oficial ni revisada por su autor.

Fuentes del proveedor: https://mymemory.translated.net/doc/spec.php y https://mymemory.translated.net/doc/usagelimits.php.

Los catálogos no traducen nombres de proyectos, texto arbitrario escrito por jugadores ni toda la salida de Minecraft; los mensajes externos se conservan para diagnóstico. La cobertura mide las claves del catálogo, no una certificación lingüística de cada contenido dinámico.

La expansión cubre los diez idiomas principales solicitados. Catalán y esperanto conservan sus catálogos previos; las claves nuevas sin revisión se muestran en español y su porcentaje refleja esa cobertura parcial. Pirate Speak y LOLCAT derivan las nuevas cadenas del catálogo inglés.

Las cadenas nuevas se prepararon con traducción asistida local y revisión de terminología y mensajes de privacidad. Las pruebas verifican las claves y los flujos de interfaz; no equivalen a una certificación de traducción profesional.
