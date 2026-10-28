# Datos para solicitar acceso de TP LAUNCHER a Minecraft Services

La solicitud la presenta el titular desde https://aka.ms/mce-reviewappid. El titular está preparando el formulario; este documento reúne datos técnicos para sus campos y no demuestra que se haya enviado o aprobado la solicitud.

- Nombre de aplicación: **TP LAUNCHER**.
- Contacto público autorizado: **tplabs.contacto@gmail.com**.
- Identificador propio de Microsoft Entra: **6a4f8732-6bf7-49c3-a749-16607f97281d**.
- Plataforma: aplicación de escritorio Windows con Rust, Tauri y Svelte.
- Autenticación: Authorization Code Flow de cliente público, con PKCE S256, navegador del sistema y redirección registrada `http://localhost:8080`.
- Permisos Microsoft: `XboxLive.signin offline_access`.
- Finalidad: organizar instalaciones de Minecraft Java, contenido e instancias, e iniciar el juego con el perfil autorizado de su propietario. La conexión premium consulta licencia, perfil, skin y capas.
- Primer inicio: requiere completar el inicio de sesión Microsoft y confirmar la licencia Minecraft Java antes de configurar idioma y Java opcional.
- Offline Mode: habilitado después de verificar una cuenta propietaria en ese equipo. Crear, activar o lanzar perfiles locales requiere una cuenta Microsoft guardada con credencial DPAPI descifrable y un comprobante cifrado de licencia Java vigente. La política de revalidación del launcher es de 30 días; no es un plazo que se atribuya a Microsoft.
- Cadena: Microsoft → Xbox Live → XSTS → Minecraft Services; no se utiliza el Client ID de otro launcher ni se distribuye un client secret.
- Contraseña: se introduce únicamente en la página de Microsoft, nunca en el launcher.
- Persistencia: token de renovación cifrado por Windows DPAPI en el equipo del usuario, identificador y nombre de perfil; token de acceso Minecraft en memoria. No se incluyen tokens ni contraseñas en GitHub o en el instalador.
- Repositorio del titular: https://github.com/tomaap07/MinecraftTPS. El titular autorizó hacerlo público después de revisar secretos y archivos personales. Comprobar su visibilidad efectiva antes de comunicarlo como enlace público al revisor. El README principal está en inglés y enlaza la versión española; no se necesita hacer públicos los datos locales de los usuarios.
- Diagnóstico observado: Microsoft y Xbox completan sus etapas, pero `authentication/login_with_xbox` devuelve HTTP 403. Se solicita revisar la autorización del identificador propio para Minecraft Services.

El contacto público disponible es [el perfil del mantenedor](https://github.com/tomaap07) y [las incidencias del proyecto](https://github.com/tomaap07/MinecraftTPS/issues). El nombre legal, correo del formulario, país, enlaces públicos, política de privacidad y cualquier dato comercial deben ser completados por el titular. No inventar datos ni publicar cuentas, configuración privada o secretos para completar la solicitud.

Referencia del requisito de revisión y del flujo: https://minecraft-launcher-lib.readthedocs.io/en/stable/tutorial/microsoft_login.html#apply-for-permission.
