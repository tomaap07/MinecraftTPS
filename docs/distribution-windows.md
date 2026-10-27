# Distribución Windows y SmartScreen

«Windows protegió su PC» y «aplicación desconocida» son avisos de SmartScreen sobre confianza/reputación. No equivalen por sí solos a una detección de malware. Un análisis sin amenazas tampoco establece reputación SmartScreen.

Los artefactos 0.1.2 se generan sin un certificado Authenticode de editor configurado. No se utiliza firma autofirmada para aparentar confianza y no se instalan certificados raíz ni exclusiones de Defender. No se garantiza eliminar el aviso recompilando o cambiando el nombre del archivo.

Para futuras publicaciones, TP LABS debe obtener una identidad de firma confiable y configurar la firma de Tauri para el ejecutable y el instalador, con sellado de tiempo. La clave privada debe permanecer en el almacén del firmante o servicio de firma, nunca en GitHub ni dentro del launcher. Una firma válida tampoco garantiza reputación inmediata para una aplicación nueva. Otra vía es una distribución compatible con Microsoft Store.

Se publica el SHA-256 del instalador para comprobar integridad. El repositorio privado limita acceso al archivo, pero no firma el programa ni lo convierte automáticamente en confiable para Windows.

Fuentes: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation y https://www.microsoft.com/en-us/wdsi/filesubmission.
