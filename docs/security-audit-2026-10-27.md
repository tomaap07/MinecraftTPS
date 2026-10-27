# Revisión de credenciales antes de publicar — 2026-10-27

## Cierre del instalador 0.1.1

El instalador definitivo tiene 223.100.169 bytes y SHA-256 `07015ccbb245415e31085d446ee2dffc27b73f65f56fa6c16736e20a5f85a8cf`. Se extrajo con 7-Zip hacia una carpeta nueva, sin instalarlo. Su PE coincide byte a byte con el PE de producción revisado; se comprobaron los tres archivos de licencia y el contenido de ambos JAR. La comparación de los cuatro secretos locales contra fuentes, payload y JAR descomprimidos efectuó 2.060 comprobaciones, con cero coincidencias; la pasada heurística registró cero hallazgos altos. No se encontraron archivos de sesión o configuración privada en el paquete. [Informe completo saneado](validation/installer-payload-0.1.1.json).

La revisión visual abrió el asistente hasta el resumen y lo canceló antes de instalar: comprobó versión, nuevo logo, TP Explorer, licencia, destino y accesos directos. Se corrigieron el contraste de la casilla del escritorio y el espacio de la nota final antes del último empaquetado. No se ejecutó una instalación sobre la copia del usuario.

El reempaquetado final utilizó `npm run tauri -- bundle --bundles nsis --no-binary-patching` para conservar exactamente el PE ya compilado. El único formato de distribución configurado es NSIS y no hay integración de updater. La comparación de una pasada anterior mostró únicamente el marcador de tres bytes de tipo de bundle que Tauri parchea y restaura; se investigó esa diferencia antes de obtener la igualdad exacta del paquete final.

Las secciones siguientes conservan la cronología de las pasadas iniciales; sus referencias al ejecutable anterior o a una extracción pendiente no describen el estado del paquete 0.1.1 ya auditado. La revisión no constituye una garantía absoluta de ausencia de cualquier secreto ni una certificación jurídica.

## Resultado

La pasada heurística sobre los archivos de trabajo publicables y los blobs de texto del historial accesible no detectó credenciales privadas. Los hallazgos informativos corresponden al identificador público de la aplicación OAuth en sus fuentes, ejemplo de configuración y documentación, también presente en objetos históricos. Un Client ID de escritorio no es un client secret.

Esto no garantiza que ningún secreto pueda existir: el análisis no prueba credenciales con proveedores, no descomprime instaladores/archivos y no interpreta imágenes. No se publicaron archivos, no se hicieron commits ni se cambiaron claves, variables remotas o archivos privados del usuario.

## Herramienta y comprobaciones

Se añadió `scripts/audit-secrets.mjs`, que solo devuelve ruta, línea, tipo, severidad y una descripción redactada. Nunca devuelve el valor coincidente ni la línea de código que lo contiene. Detecta formatos conocidos, asignaciones de campos privados, JWT, claves privadas, URLs de base de datos con contraseña y determinadas formas de secretos en SQL/headers. Distingue Client IDs públicos y JWT anon de Supabase de credenciales privadas; esas claves públicas no eximen de comprobar RLS.

Comandos disponibles:

```powershell
node scripts/test-audit-secrets.mjs
node scripts/audit-secrets.mjs --history
node scripts/audit-secrets.mjs --staged
node scripts/audit-secrets.mjs --binary src-tauri/target/release/launchertps.exe
node scripts/audit-secrets.mjs --local-secret-file .env --local-secret-file tp-api/.env.local --binary src-tauri/target/release/launchertps.exe
```

`--history` examina blobs de texto accesibles mediante las referencias de Git. `--staged` lee el contenido del índice, no la copia del archivo en disco. `--path ruta` permite incluir explícitamente un archivo local ignorado; no debe utilizarse para anexar credenciales a un commit. `--report ruta` guarda únicamente resultados redactados. Los informes de esta ejecución se escribieron dentro de `.codex-tmp/`, que está ignorado.

`--local-secret-file ruta` incorpora una segunda comprobación independiente de los patrones: carga únicamente las variables privadas admitidas del archivo ENV indicado y busca su valor completo en los archivos publicables y los binarios explícitos, tanto en UTF-8 como UTF-16LE. El parser admite `export`, comillas y comentarios sin ejecutar comandos, expandir variables ni alterar los signos `$`. Client IDs y claves anon no se incorporan a esa lista de secretos. Los informes muestran únicamente el nombre de la variable, archivo y presencia/ausencia mediante resultados y contadores; no incluyen valores, fragmentos ni hashes de claves.

Esta comparación exacta se probó con valores sintéticos, signos `$`, comillas/export, copias UTF-8/UTF-16LE y fragmentos parciales que deben dar resultado negativo. Una pasada adicional con los valores locales reales de `.env` y `tp-api/.env.local` examinó 406 archivos, realizó 1624 comparaciones y no encontró coincidencias. Incluía el EXE anterior a la compilación 0.1.1; el instalador nuevo requiere su propia pasada cuando esté disponible. Los valores se mantuvieron solo en memoria y nunca se enviaron a los proveedores.

Las pruebas usan credenciales sintéticas generadas durante la ejecución y un repositorio temporal aislado. Comprueban detección, redacción, placeholders, JWT anon frente a service_role, Client ID público y el caso de un secreto que permanece staged aunque se haya quitado de la copia de trabajo. El temporal se elimina únicamente después de validar su ruta y prefijo. Las pruebas pasaron.

La última pasada general previa a este informe examinó aproximadamente 325 textos de trabajo y 337 blobs históricos. Registró cero hallazgos de severidad alta y seis informativos. Los contadores pueden aumentar al añadir este propio documento o nuevos cambios. Los tres archivos tracked eliminados no se leyeron en disco; sus contenidos históricos se cubren cuando son blobs de texto. Los archivos binarios se excluyen de la pasada textual.

## Datos locales privados identificados

| Ubicación | Tipo de dato | Estado comprobado |
| --- | --- | --- |
| `.env` | Clave de CurseForge suministrada para el uso local | Ignorado y no presente en el índice. No se imprimió ni se verificó su vigencia contra CurseForge. Su formato coincide con el de una clave o hash bcrypt; se trata como privado por su finalidad configurada. |
| `tp-api/.env.local` | `TP_DB_BRIDGE_TOKEN`, `TP_MASTER_SECRET`, `CRON_SECRET` | Ignorado y fuera del índice. Contiene configuración privada operativa; no se modificó. |
| `tp-api/.env.local` | Clave anon de Supabase | Identificador público de acceso, no service_role. Su seguridad depende también de las políticas de la base, no auditadas de nuevo en esta tarea. |
| `.codex-tmp/tp-api-deploy-secrets.json` | Copias locales de master, bridge y cron | Archivo temporal privado ignorado; contiene valores y debe conservar protección local. |
| `.codex-tmp/tp-api-local-env.json` | Copias locales de las variables privadas de TP API | Archivo temporal privado ignorado; no publicarlo ni incluirlo en un instalador. |
| `tp-api/.vercel/project.json` | Identificadores de vinculación de proyecto/equipo | Ignorado. Su existencia no demuestra que contenga un token de autenticación; no se usó para autenticar ninguna petición. |

Se mantuvieron intactos todos estos archivos. Si algún valor hubiera sido compartido fuera de sus destinatarios previstos, su propietario debe valorar revocarlo y generarlo de nuevo; esta revisión no hizo rotaciones ni supone una filtración pública por el mero hecho de encontrarse en un archivo local ignorado.

No se leyó el almacén de autenticación de GitHub CLI, el gestor de credenciales del sistema ni los tokens de cuentas de Minecraft de AppData. Tampoco se envió una clave a un servicio ajeno a su finalidad.

## Exclusiones y archivos de autor

Se verificó que las reglas existentes excluyen `.env`/`.env.*` salvo `.env.example`, `integrations.env`, `curseforge-key.dat`, `.codex-tmp`, `.codex-remote-attachments`, `src-tauri/config`, `src-tauri/logs`, `src-tauri/target`, los builds locales de TP Bridge/Companion, `.gradle`, `node_modules`, `.vercel`, `tp-api/dist` y `supabase/.temp` según los archivos `.gitignore` aplicables. Una regla no elimina un archivo que ya estuviera tracked: por ello se comprobó el índice para las ubicaciones privadas concretas.

No se cambió `.gitignore` en esta auditoría. Para futuras carpetas conviene mantener exclusiones equivalentes para `/dist/`, `/coverage/`, `/.cache/`, `/artifacts/`, `/screenshots/`, volcados de memoria y archivos locales de llaves/certificados privados. No debe excluirse genéricamente todo `static/`, `img/`, `docs/` o todos los PNG: ahí puede haber recursos necesarios y avisos públicos. Los originales personales y adjuntos de trabajo deben quedar en una carpeta de autor claramente separada e ignorada; los recursos que se distribuyan requieren revisar derechos y privacidad.

La búsqueda textual no realiza OCR de capturas ni detecta información personal visible dentro de una imagen. Los adjuntos personales de `.codex-remote-attachments` permanecen excluidos. Revisar las imágenes publicables sigue siendo necesario aunque el scanner no encuentre un token.

## Binarios e historial

Se buscó texto ASCII y UTF-16LE en el `src-tauri/target/release/launchertps.exe` ya existente, con fecha 26 de octubre. No se detectaron credenciales con las reglas de esta herramienta. Esto no acredita que ese binario corresponda a los fuentes nuevos ni valida todo su contenido.

Existen instaladores NSIS anteriores bajo `src-tauri/target/release/bundle/nsis/`. Sus payloads comprimidos no se extrajeron. No se puede concluir que estén libres de claves o sesiones a partir de una búsqueda de strings del contenedor. Para publicar corresponde compilar desde las fuentes revisadas, comprobar el contenido del instalador concreto y repetir la auditoría sobre el EXE resultante. La configuración actual de recursos de Tauri enumera los JAR de TP Companion y TP Bridge; no enumera los archivos `.env` o los temporales de despliegue.

### Preparación para extraer NSIS sin ejecutarlo

Se preparó una copia portátil de las herramientas oficiales de [7-Zip](https://www.7-zip.org/download.html), versión 26.05. El [sitio oficial](https://www.7-zip.org/) incluye NSIS entre los formatos de extracción. Se descargaron `7zr.exe` y `7z2605-x64.exe` desde los assets enlazados de [ip7z/7zip, release 26.05](https://github.com/ip7z/7zip/releases/tag/26.05). El segundo archivo se trató exclusivamente como archivo de datos SFX: se listó y extrajo con `7zr`, sin ejecutar su instalador. Se obtuvieron `7z.exe` y `7z.dll` completos; su listado de formatos confirma el handler NSIS. Las variantes reducidas por sí solas no se consideran una comprobación del soporte NSIS completo.

Ambas descargas coincidieron con los digests SHA-256 suministrados por la API de la publicación oficial de GitHub:

| Asset oficial | SHA-256 |
| --- | --- |
| `7zr.exe` | `9bf582b894dfa665f852d7e9112b02290a1c303c2d95a21fcdc41ffcb03143b7` |
| `7z2605-x64.exe` | `fd99e86f57d8f429ef5ed045bf98315af962e2309b0f1ba87a147a44c71647d0` |

`Get-AuthenticodeSignature` informó `NotSigned` para ambos; no se afirma que tengan una firma de código válida. La comprobación realizada fue la correspondencia de los hashes con la metadata de la fuente oficial descargada por HTTPS. Las herramientas permanecen bajo `.codex-tmp/7zip-bootstrap-2605/`, fuera del repositorio publicable. No se instaló 7-Zip, no se ejecutó un setup y no se modificó el registro o la instalación actual del launcher. El siguiente paso es extraer únicamente el NSIS nuevo hacia una carpeta vacía de auditoría y examinar sus archivos, sin ejecutarlos.

No se encontraron secretos privados mediante las reglas aplicadas a los blobs de texto accesibles con `git rev-list --objects --all`. Esto no cubre objetos eliminados sin referencia, backups externos, otras copias del repositorio, releases remotas antiguas o archivos ya compartidos por otros canales. Antes de cualquier publicación deben repetirse los checks sobre el índice final y los artefactos exactos; no se recomienda `git add -f` sobre las carpetas excluidas.
