# Datos para solicitar acceso de TP LAUNCHER a Minecraft Services

La solicitud la presenta el titular desde https://aka.ms/mce-reviewappid. Este documento es un borrador de datos técnicos para copiar en los campos que realmente solicite el formulario; no se ha enviado ni se afirma que haya aprobación.

- Nombre de aplicación: **TP LAUNCHER**.
- Identificador propio de Microsoft Entra: **6a4f8732-6bf7-49c3-a749-16607f97281d**.
- Plataforma: aplicación de escritorio Windows con Rust, Tauri y Svelte.
- Autenticación: Authorization Code Flow de cliente público, con PKCE S256, navegador del sistema y redirección registrada `http://localhost:8080`.
- Permisos Microsoft: `XboxLive.signin offline_access`.
- Finalidad: organizar instalaciones de Minecraft Java, contenido e instancias, e iniciar el juego con el perfil autorizado de su propietario. La conexión premium consulta licencia, perfil, skin y capas.
- Cadena: Microsoft → Xbox Live → XSTS → Minecraft Services; no se utiliza el Client ID de otro launcher ni se distribuye un client secret.
- Contraseña: se introduce únicamente en la página de Microsoft, nunca en el launcher.
- Persistencia: token de renovación cifrado por Windows DPAPI en el equipo del usuario, identificador y nombre de perfil; token de acceso Minecraft en memoria. No se incluyen tokens ni contraseñas en GitHub o en el instalador.
- Repositorio del titular: https://github.com/tomaap07/MinecraftTPS. Actualmente **privado**. No indicar que el código o las descargas son públicos si el formulario solicita acceso público; el titular debe decidir qué documentación y enlaces públicos proporcionar.
- Diagnóstico observado: Microsoft y Xbox completan sus etapas, pero `authentication/login_with_xbox` devuelve HTTP 403. Se solicita revisar la autorización del identificador propio para Minecraft Services.

El nombre legal, contacto, país, enlaces públicos, política de privacidad y cualquier dato comercial deben ser completados y revisados por el titular. No inventar datos ni compartir el repositorio privado, sus archivos de configuración o secretos para rellenar el formulario.

Referencia del requisito de revisión y del flujo: https://minecraft-launcher-lib.readthedocs.io/en/stable/tutorial/microsoft_login.html#apply-for-permission.
