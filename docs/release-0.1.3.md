# TP LAUNCHER 0.1.3 · Diagnóstico del inicio de sesión

Fecha: 28 de octubre de 2026.

## Problema confirmado

El registro local de 0.1.2 muestra que el canje final de Xbox a Minecraft Services devuelve HTTP 403. En Ajustes aparecía «Listo» porque `connectMicrosoft` capturaba la excepción sin rechazar la operación invocada por el panel. El perfil de Minecraft nunca había llegado a obtenerse.

0.1.3 propaga el fallo a Ajustes y al asistente inicial, conserva el mensaje en Cuenta y valida el perfil antes de presentarlo como conectado. El botón puede volver a utilizarse tras el error. La página de retorno del navegador aclara que recibir la autorización de Microsoft todavía no completa la sesión de Minecraft. Se informa del progreso por etapas sin incluir credenciales.

El canje Xbox/XSTS/Minecraft se separa en un módulo comprobable, utiliza las claims de la respuesta XSTS, conserva los reintentos acotados de los tickets Xbox y exige la duración del token de Minecraft. La renovación utiliza esa duración, en lugar de la del token de Microsoft. Las solicitudes usan identificación propia de TP LAUNCHER, plazos de conexión, respuestas acotadas y no siguen redirecciones de autenticación. Los errores no incluyen respuestas privadas.

## Bloqueo externo pendiente

Este cambio **no concede permiso de acceso a Minecraft Services** ni elimina por sí solo un rechazo 403. Para aplicaciones nuevas se requiere revisar la aprobación del Client ID propio con Mojang. El HTTP 403 observado es compatible con esa causa, pero también requiere revisar el servicio si la aplicación ya fue aprobada. No se afirma que la cuenta carezca de licencia ni se usa el identificador de otra aplicación para evitar el rechazo.

El titular manifestó que desconocía este requisito. No se ha enviado ninguna solicitud en su nombre ni se ha confirmado aprobación. El formulario y los pasos están en [configuración Microsoft](microsoft-client-configuration.md). No se ha comprobado un login real completo exitoso con el Client ID actual.

## Firma digital

El titular confirmó que no dispone de certificado ni servicio de firma. **Este instalador sigue sin firma Authenticode**. No se ha generado una firma autofirmada, cambiado certificados raíz ni alterado SmartScreen/Defender.

Se incorpora `npm run build:signed` para un certificado propio confiable en el almacén de Windows: verifica vigencia, uso de firma de código, clave privada y cadena de confianza antes de compilar; configura SHA-256 y sello de tiempo; después exige firmas válidas del mismo editor en el ejecutable y el instalador. No se ha podido ejecutar ese recorrido con una identidad real. La verificación sin certificado se probó y se detiene antes de compilar. Véase [distribución Windows](distribution-windows.md).

## Pruebas

- `cargo test --quiet`: 108 aprobadas, 19 ignoradas, ninguna fallida.
- Cuatro pruebas HTTP locales nuevas: canje exitoso simulado con claims XSTS y duración Minecraft; HTTP 403 con instrucciones y sin filtrar el cuerpo; reintentos Xbox y parada por rechazo XSTS; respuesta incompleta que no se acepta como sesión.
- Interfaz en Edge aislado con IPC simulado: fallo 403 visible en Ajustes sin «Listo», fallo visible en el asistente, reintento exitoso simulado que muestra la cuenta y permite terminar. También se comprueban los diez idiomas, Java opcional y el modo local. No equivale a una autenticación Microsoft real.
- Comprobación Svelte sin errores ni advertencias; contratos de idioma e interfaz y pruebas del auditor de secretos aprobados.
- La compilación de producción conserva la advertencia de chunks de interfaz/3D de más de 500 kB.

El alcance de esta actualización es corregir el falso éxito y el diagnóstico/gestión del flujo. La autorización externa de la aplicación y la obtención de una identidad de firma siguen siendo requisitos pendientes del titular.

## Artefacto verificado

- `TP-LAUNCHER-0.1.3-Windows-x64-Setup.exe`: 220695619 bytes.
- SHA-256 del instalador: `e180e190315181243ba14d1ec2ae6180ccb4e7eeff6f704ad0bb8aa7d8fc9806`.
- SHA-256 del ejecutable incluido: `91173fd13511c50a01eac5aa6079a1ccc4d5c4395a49a133b7ac4373b09605ae`.
- Inspección del contenido NSIS y de ambos conectores JAR extraídos. Los únicos tres bytes diferentes respecto al ejecutable independiente son la marca de tipo de bundle de Tauri (`NSS`/`UNK`). Los 19 chunks actuales de la interfaz están presentes en el ejecutable; se descomprimieron 52 recursos para la revisión.
- Auditoría de código preparado para Git, historial y contenido extraído: 540 archivos de texto analizados, 433 blobs históricos y 753 archivos comparados contra los cuatro secretos locales; 0 coincidencias y 0 hallazgos de severidad alta. Los cinco avisos informativos corresponden a identificadores OAuth públicos. No constituye una garantía absoluta de ausencia de secretos.
- Microsoft Defender: análisis personalizado local del instalador final terminado sin amenazas detectadas. Sin cambios en protecciones, exclusiones ni certificados. Este resultado no es una certificación de seguridad ni establece reputación SmartScreen.
- Authenticode: `NotSigned`, coherente con la ausencia de una identidad de firma. No se publica como ejecutable firmado.
