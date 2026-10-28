# Microsoft y perfiles desconectados

## Rechazo de Minecraft Services después del login de Microsoft

En los registros de la versión 0.1.2 se observó un HTTP 403 al llamar a `authentication/login_with_xbox`, después de Microsoft y Xbox. Un error de esta etapa impide obtener el token de Minecraft y no debe mostrarse como una cuenta conectada. Ajustes mostraba «Listo» porque la interfaz capturaba el error sin rechazar la operación; esto se corrige en 0.1.3 para Ajustes, Cuenta y el asistente inicial.

Las aplicaciones nuevas deben solicitar el acceso a Minecraft Services. Registrar la aplicación en Entra y obtener un token de Microsoft son pasos diferentes de esta aprobación. El HTTP 403 es compatible con una aplicación pendiente de aprobación; por sí solo no prueba esa causa ni que falte la licencia del usuario. Si la aplicación ya está aprobada, hay que revisar el rechazo con el soporte del servicio.

Formulario de revisión: https://aka.ms/mce-reviewappid. El titular debe indicar TP LAUNCHER, el Client ID propio registrado y la información del proyecto que solicite el formulario. No se ha enviado esta solicitud en nombre del titular ni se considera concedido el acceso. La biblioteca documenta el requisito y enlaza el formulario: https://minecraft-launcher-lib.readthedocs.io/en/stable/tutorial/microsoft_login.html#apply-for-permission.

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

## Perfiles Offline / locales

Los perfiles locales se crean con un nick válido sin conectar Microsoft. No
requieren una licencia cacheada de 30 días ni se convierten en cuentas oficiales.
El botón Modo offline crea o selecciona el perfil local usando el nick actual.
Al seleccionarlo, restaurar el launcher o pulsar Jugar no intenta renovar una
cuenta Microsoft antigua. Las credenciales Microsoft guardadas se conservan por
separado para que su propietario pueda volver a elegirlas.

El juego recibe un UUID local estable, `accessToken=0` y `userType=legacy`.
Se mantiene el algoritmo anterior del UUID para conservar los mundos existentes.
No habilita servidores que exigen autenticación Microsoft. La skin pública en el
visor no cambia estas limitaciones. Usa una copia del juego que tengas derecho
a utilizar.

Sin red, Java, cliente, bibliotecas y assets deben estar ya instalados. Un perfil
local no hace disponibles archivos que faltan ni catálogos online. El acceso
oficial Microsoft conserva PKCE, tokens protegidos y comprobación de propiedad.

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
