# Corrección de perfiles locales

La restricción de 30 días era un control implementado en TP, no un error de Java. Se ha retirado del perfil local de pruebas solicitado por el propietario. La autenticación Microsoft y su comprobación real de licencia siguen separadas.

- Se permite crear/activar/lanzar perfiles locales con nick válido sin una cuenta Microsoft guardada.
- La selección local se consulta antes de intentar restaurar o renovar Microsoft. No se eliminan las credenciales protegidas de las otras cuentas.
- Se mantienen el UUID local anterior y los directorios de juego para conservar los mundos existentes. El token local es 0 y el tipo de usuario legacy; no representa una identidad autenticada.
- La interfaz comprueba la cuenta antes de preparar los cargadores o plugins y lleva a Cuenta si falla la sesión oficial.
- El botón Modo offline crea/selecciona el nick actual.
- Sin internet deben estar ya instalados Java, cliente, bibliotecas y assets. Los servicios online como CurseForge siguen necesitando su configuración y conexión.

Pruebas: npm run check (0 errores/0 advertencias), 78 pruebas Rust y un arranque real independiente de Vanilla 1.21.1 con usuario TPSOfflineTest y sin sesión Microsoft. El cliente inicializó recursos/audio y se cerró únicamente el PID de esa prueba. No se abrió ni modificó ningún mundo de usuario. No se han probado todas las versiones.
