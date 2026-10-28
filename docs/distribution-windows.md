# Distribución Windows y SmartScreen

«Windows protegió su PC» y «aplicación desconocida» son avisos de SmartScreen sobre confianza/reputación. No equivalen por sí solos a una detección de malware. Un análisis sin amenazas tampoco establece reputación SmartScreen.

Los artefactos 0.1.2 se generan sin un certificado Authenticode de editor configurado. No se utiliza firma autofirmada para aparentar confianza y no se instalan certificados raíz ni exclusiones de Defender. No se garantiza eliminar el aviso recompilando o cambiando el nombre del archivo.

Para futuras publicaciones, TP LABS debe obtener una identidad de firma confiable y configurar la firma de Tauri para el ejecutable y el instalador, con sellado de tiempo. La clave privada debe permanecer en el almacén del firmante o servicio de firma, nunca en GitHub ni dentro del launcher. Una firma válida tampoco garantiza reputación inmediata para una aplicación nueva. Otra vía es una distribución compatible con Microsoft Store.

Se publica el SHA-256 del instalador para comprobar integridad. El repositorio privado limita acceso al archivo, pero no firma el programa ni lo convierte automáticamente en confiable para Windows.

Fuentes: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation y https://www.microsoft.com/en-us/wdsi/filesubmission.

## Compilación con certificado propio

El titular confirmó el 28 de octubre de 2026 que todavía no dispone de certificado ni servicio de firma. No se ha añadido una identidad ficticia y los instaladores actuales siguen sin firma Authenticode.

Cuando obtenga un certificado de firma de código emitido por una autoridad confiable, debe configurarlo con su proveedor de clave privada en `CurrentUser/My`. El certificado debe estar vigente, tener el uso de firma de código y una cadena reconocida por Windows. Su huella pública se establece en `TP_SIGN_CERT_THUMBPRINT`; la clave privada y cualquier PIN se mantienen fuera del proyecto.

`node scripts/build-signed-windows.mjs --check-only` verifica estos requisitos sin compilar. `npm run build:signed` genera la configuración temporal ignorada por Git, compila con la firma SHA-256 de Tauri y sello de tiempo RFC 3161, y exige firmas válidas del mismo editor en el ejecutable y el instalador. La ausencia de certificado detiene esta operación antes de compilar. La compilación habitual continúa permitiendo artefactos sin firma y no debe anunciarse como firmada.

La integración preparada es para certificados en el almacén local con su proveedor de clave. Un servicio remoto como Azure Artifact Signing necesita además una cuenta y un perfil de firma verificados y una configuración `signCommand` específica; no se ha contratado ni configurado ese servicio. Referencias: https://tauri.app/distribute/sign/windows/ y https://learn.microsoft.com/windows/apps/package-and-deploy/code-signing-options.
