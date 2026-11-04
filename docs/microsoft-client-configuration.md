# Microsoft y Offline Mode tras verificar Minecraft Java

Guía de comportamiento vigente. Los informes de versiones anteriores conservan su fecha y documentan el comportamiento de aquellas compilaciones; véase el [índice de documentación](README.md).

## Investigación de compatibilidad del 4 de noviembre de 2026

Las capturas de Entra confirman el Client ID `6a4f8732-6bf7-49c3-a749-16607f97281d`, cuentas personales y la URI `http://localhost:8080` bajo aplicaciones móviles y de escritorio. No apareció un reemplazo del Client ID en `.env.local`, `.env` ni en el entorno del proceso de compilación inspeccionado. Esto no identifica por sí solo la aplicación incluida en una aprobación enviada por correo.

Se comparó el intercambio con el código público actual de [Prism](https://github.com/PrismLauncher/PrismLauncher/blob/develop/launcher/minecraft/auth/steps/LauncherLoginStep.cpp) y [Modrinth](https://github.com/modrinth/code/blob/main/packages/app-lib/src/state/minecraft_auth.rs). Ambos usan `POST https://api.minecraftservices.com/launcher/login` con `xtoken` y `platform: PC_LAUNCHER`. TP utilizaba `/authentication/login_with_xbox` con `identityToken`; esta revisión alinea ese paso con el formato de los clientes examinados. No se ha demostrado que el endpoint anterior sea la causa del rechazo ni que haya dejado de funcionar para todas las aplicaciones.

El diagnóstico nuevo indica `/launcher/login` y el Client ID efectivo. Microsoft OAuth con PKCE, los tokens de la cuenta obtenidos de Xbox/XSTS, la comprobación de licencia y el perfil oficial siguen siendo obligatorios. Una respuesta 401/403 termina el intercambio y no provoca intentos por otros endpoints ni cambios de identidad. Las pruebas HTTP locales verifican el contrato de solicitud y la propagación de rechazos; no verifican la aprobación real de Mojang. Hace falta un nuevo inicio de sesión del titular en una compilación con este cambio antes de declarar resuelto el problema del `.exe` instalado.

El titular probó la compilación nueva y compartió un HTTP 403 de `/launcher/login` que el diagnóstico inicial no reconoció. Las capturas también confirman cuentas personales, compatibilidad Live SDK y flujos de cliente público habilitados. El cambio de endpoint no resolvió ese intento; no se debe presentar como causa confirmada ni como reparación validada.

Se añadieron diagnósticos limitados al tipo de contenido (lista fija), forma del JSON, tamaño del cuerpo, presencia de campos conocidos y códigos de error de una lista cerrada. El texto libre, nombres de campos no reconocidos, tokens y cabeceras arbitrarias no se muestran. Se reconoce el motivo exacto de registro inválido también en `message`, `error`, `reason` y `details.reason`. La comprobación local de red verificó TLS de Minecraft Services con emisor Microsoft y no detectó entradas de hosts para los cuatro dominios de autenticación ni variables de proxy en el proceso. Esto no demuestra el estado de la lista de acceso ni descarta todos los problemas de red.

Validación local de esta revisión: 13 pruebas de autenticación y licencia relacionadas aprobadas (`cargo test microsoft_`), `npm run check` sin errores ni advertencias y auditoría de los dos archivos modificados sin coincidencias con secretos locales. El inicio de sesión real sigue rechazado y pendiente del nuevo diagnóstico.

## Rechazo de Minecraft Services después del login de Microsoft

En los registros de una compilación de desarrollo 0.1.2 se observó HTTP 403 al llamar a `authentication/login_with_xbox`, después de Microsoft y Xbox. Ese error impide obtener el token de Minecraft. El código actual conserva la corrección que propaga el fallo a Ajustes, Cuenta y el asistente, sin mostrar «Listo» ni una cuenta conectada cuando la autenticación no terminó.

Las aplicaciones nuevas deben solicitar el acceso a Minecraft Services. Registrar la aplicación en Entra y obtener un token de Microsoft son pasos diferentes de esta aprobación. El HTTP 403 es compatible con una aplicación pendiente de aprobación; por sí solo no prueba esa causa ni que falte la licencia del usuario. Si la aplicación ya está aprobada, hay que revisar el rechazo con el soporte del servicio.

El 2 de noviembre de 2026 el titular compartió una notificación de Mojang Enforcement fechada el 30 de octubre que comunica la aprobación de sus aplicaciones en el lote revisado. La notificación compartida no enumera el Client ID. Sigue pendiente contrastar el identificador enviado en el formulario con el efectivo del launcher y confirmar una sesión completa: el intento posterior aún muestra HTTP 403.

El diagnóstico anterior asignaba el mismo texto a todos los 403. Ahora distingue `INVALID_APP_REGISTRATION` cuando la respuesta contiene el mensaje reconocido de registro inválido, `ACCOUNT_SUSPENDED` cuando aparece ese motivo estructurado y el resto de 403 sin atribuirles una causa no comprobada. Informa del Client ID efectivo y, si existe, de una referencia UUID del servicio; no imprime cuerpos completos, tokens, nombres ni correos. No cambia scopes, identidad de aplicación ni comprobaciones de licencia para eludir el rechazo. Una autorización nueva debe completarse en el navegador de Microsoft; un token antiguo rechazado con `invalid_grant` no sirve para probar el acceso actual.

Formulario de revisión: https://aka.ms/mce-reviewappid. La biblioteca documenta el requisito y enlaza el formulario: https://minecraft-launcher-lib.readthedocs.io/en/stable/tutorial/microsoft_login.html#apply-for-permission. Si el servicio responde `INVALID_APP_REGISTRATION` para el mismo ID aprobado, corresponde facilitar la confirmación y el diagnóstico al soporte de Minecraft; el launcher no administra esa lista.

La página de retorno del navegador solo confirma la recepción de la autorización de Microsoft. El resultado definitivo está en el launcher, después de Xbox/XSTS, Minecraft Services, licencia, perfil y almacenamiento protegido. El progreso no incluye tokens ni respuestas privadas.

TP LAUNCHER usa el identificador público de aplicación que facilitó su autor:
`6a4f8732-6bf7-49c3-a749-16607f97281d`. Este es el valor predeterminado del
backend y se puede sustituir por otro registro propio con `VITE_MS_CLIENT_ID`.
No incorpora el Client ID de Minecraft Launcher, Prism,
Xbox ni de otra aplicación. Un Client ID público no es un secreto ni una
autorización para adoptar la identidad de otra aplicación.

## Configuración del desarrollador

1. Registra una aplicación **propia** en Microsoft Entra para cuentas personales
   de Microsoft y la plataforma de aplicaciones móviles y de escritorio.
2. Registra la redirección **`http://localhost:8080`** en la plataforma móvil
   y de escritorio (cliente público/nativo). El launcher escucha únicamente
   en `127.0.0.1:8080` mientras dura la solicitud. Usa exactamente la misma URI
   al pedir y canjear el código. Si el puerto está ocupado, informa del problema
   antes de abrir el navegador; no cambia silenciosamente de puerto.
3. Comprueba que tu aplicación tenga el acceso necesario a Xbox y Minecraft
   Services. Registrar una aplicación no garantiza que Minecraft Services la
   autorice; esa autorización depende de Microsoft/Mojang.
4. Copia su Application (client) ID en `.env.local` o en el entorno del proceso:

   ```dotenv
   VITE_MS_CLIENT_ID=6a4f8732-6bf7-49c3-a749-16607f97281d
   ```

   El valor anterior es el identificador público proporcionado para TP LAUNCHER.
   Los forks deben usar el registro de su propio autor. No añadas un client
   secret: una aplicación pública de escritorio no debe distribuirlo.
5. Reinicia `tauri dev` o recompila. `build.rs` lee exclusivamente esa variable
   de `.env.local`/`.env` y la incorpora al binario como identificador público.
   El resto de valores de esos archivos no se incorpora por este mecanismo.

El inicio de sesión usa el navegador del sistema, el endpoint oficial
`https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize`, el scope
**`XboxLive.signin offline_access`**, Authorization Code, PKCE S256 y un `state`
aleatorio. Microsoft conduce la autenticación de cuentas personales a su
página de inicio de sesión. El callback local valida método, Host, ruta y estado
antes de
intercambiar el código. Luego se completa la cadena Xbox Live → XSTS →
Minecraft Services. El launcher verifica la licencia Java mediante
`entitlements/mcstore` y obtiene el perfil oficial antes de guardar la sesión.
Si cambias de Client ID, los refresh tokens concedidos a otra aplicación pueden
dejar de servir: el usuario tendrá que volver a autorizar TP LAUNCHER.

## Estado del registro comprobado el 26 de octubre de 2026

Las capturas del registro confirmaron el Application (client) ID correcto:
`6a4f8732-6bf7-49c3-a749-16607f97281d`. El valor comunicado inicialmente tenía
`a740` en lugar de `a749`; ese carácter distinto causaba el rechazo anterior.

Una comparación directa sin credenciales mostró que el ID anterior devuelve
`unauthorized_client`, mientras que el ID correcto conduce desde el endpoint
`consumers` a la página oficial de Microsoft titulada «Sign in to your Microsoft
account», sin ese error. Se usaron la redirección `http://localhost:8080`, el scope
`XboxLive.signin offline_access` y PKCE S256 del launcher.

Las capturas muestran cuentas personales, plataforma de escritorio, la URI
correcta y los flujos de cliente público habilitados. No hace falta añadir las
otras URI predefinidas ni crear un secreto para este flujo. La prueba confirma
que se acepta el inicio de autorización; todavía no acredita un inicio de sesión
completo ni los canjes posteriores Xbox Live/XSTS/Minecraft, que requieren que
el propietario complete el acceso en Microsoft.

## Primer inicio y Offline Mode

El primer paso del asistente exige una cuenta Microsoft cuya autenticación complete Xbox/XSTS, Minecraft Services, la comprobación de licencia Java y la obtención del perfil. El idioma de Windows se detecta para presentar el asistente; la configuración de idioma y Java opcional viene después de ese requisito de cuenta.

El modo de creación y uso de perfiles locales queda disponible únicamente cuando la biblioteca de ese equipo contiene al menos una cuenta Microsoft con su token de renovación cifrado por DPAPI y descifrable por el usuario de Windows, más un registro de licencia Java verificada que corresponda a esa cuenta y siga vigente. Un nombre, un UUID escrito en el JSON o un antiguo perfil local no bastan. La comprobación se aplica en Rust a la elegibilidad, creación, activación y lanzamiento; deshabilitar un botón no es la única barrera.

Cuando falta esa evidencia se muestra: **«Para utilizar perfiles locales, debes iniciar sesión con una cuenta Microsoft al menos una vez en este equipo»**. La interfaz incluye un enlace a la [tienda oficial](https://www.minecraft.net/es-es/store/minecraft-deluxe-collection-pc?tabs=%7B%22details%22%3A0%7D) para quien necesite adquirir el juego.

El registro de verificación cifrado tiene una ventana de 30 días, una política del launcher para revalidar periódicamente con conexión. No se atribuye ese plazo a una norma específica de Microsoft. Si deja de existir una cuenta Microsoft apta o caduca la verificación, los perfiles locales vuelven a quedar bloqueados hasta completar la comprobación.

Después de cumplir el requisito, el propietario puede crear y alternar perfiles locales con nombres válidos en su armario. Un perfil sin conexión usa identidad local estable y no representa una sesión autenticada ante servidores oficiales. La vista previa de una skin por nombre de usuario no cambia la identidad de autenticación.

Sin red, Java, cliente, bibliotecas y assets deben estar previamente instalados. Las credenciales Microsoft se mantienen cifradas por separado para permitir volver a elegir la cuenta oficial. No se incluyen cuentas, tokens ni comprobantes privados en el repositorio o el instalador.

## OptiFine

El launcher detecta perfiles locales e importa el JAR original seleccionado por
el usuario en Instalaciones → OptiFine. No descarga OptiFine, no repara sus JAR
mediante mirrors y no incluye sus binarios en el repositorio o el instalador.
La validación del JAR comprueba su formato, nombre y clase de arranque; no es
una firma de autenticidad del autor. Los derechos del archivo siguen siendo de
su titular y la obtención del archivo corresponde al usuario.

## Referencias

- [Authorization Code con PKCE de Microsoft](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow)
- [Restricciones de redirección y loopback](https://learn.microsoft.com/en-us/entra/identity-platform/reply-url)
- [Autenticación Microsoft/Xbox y endpoint de consumidores](https://learn.microsoft.com/en-us/gaming/gdk/docs/services/fundamentals/s2s-auth-calls/service-authentication/live-website-authentication)
- [Copyright de OptiFine](https://optifine.net/copyright)

Estas medidas delimitan el comportamiento del programa; no constituyen una
garantía de cumplimiento jurídico de todos los recursos o integraciones.
