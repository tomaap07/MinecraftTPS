import licenseGate from './license-gate';
import expanded from './expanded';
import releaseUi0 from './release-ui-pt-BR';
import releaseUi1 from './release-ui-fr-FR';
import releaseUi2 from './release-ui-de-DE';
import releaseUi3 from './release-ui-it-IT';
import releaseUi4 from './release-ui-ru-RU';
import releaseUi5 from './release-ui-ja-JP';
import releaseUi6 from './release-ui-ko-KR';
import releaseUi7 from './release-ui-zh-CN';
import releaseUi8 from './release-ui-ca-ES';
import releaseUi9 from './release-ui-eo-UY';
import { installationCatalogs } from './installation';
import supplementEN from './supplement-en';
import supplementPt from './supplement-pt-BR';
import supplementFr from './supplement-fr-FR';
import supplementDe from './supplement-de-DE';
import supplementIt from './supplement-it-IT';
import supplementCa from './supplement-ca-ES';
import supplementEo from './supplement-eo-UY';
import supplementRu from './supplement-ru-RU';
import supplementJa from './supplement-ja-JP';
import supplementKo from './supplement-ko-KR';
import supplementZh from './supplement-zh-CN';
import ptBR from './pt-BR';
import frFR from './fr-FR';
import deDE from './de-DE';
import itIT from './it-IT';
import caES from './ca-ES';
import eoUY from './eo-UY';
import enPT from './en-PT';
import lolUS from './lol-US';
import ruRU from './ru-RU';
import jaJP from './ja-JP';
import koKR from './ko-KR';
import zhCN from './zh-CN';

export const english: Record<string, string> = {
  "Pantalla": "Display",
  "Abre el launcher de escritorio para ajustar su interfaz.": "Open the desktop launcher to adjust its interface.",
  "Tamaño de interfaz guardado.": "Interface size saved.",
  "El ancho debe estar entre 320 y 7680 y el alto entre 240 y 4320.": "Width must be between 320 and 7680 and height between 240 and 4320.",
  "El tamaño de ventana se ajusta desde la aplicación de escritorio.": "Window size is adjusted in the desktop app.",
  "El launcher se abrirá maximizado.": "The launcher will open maximized.",
  "Tamaño de ventana guardado.": "Window size saved.",
  "No se pudo consultar el tamaño de la ventana.": "Could not read the window size.",
  "PANTALLA": "DISPLAY",
  "Tamaño de la interfaz": "Interface size",
  "Aumenta los textos, botones, iconos y menús de todo el launcher.": "Enlarge text, buttons, icons and menus throughout the launcher.",
  "Reducir tamaño de interfaz": "Decrease interface size",
  "Más grande": "Larger",
  "Aumentar tamaño de interfaz": "Increase interface size",
  "Tamaños de interfaz": "Interface sizes",
  "Restablecer 100%": "Reset to 100%",
  "Se aplica al instante y se conserva al volver a abrir el launcher.": "Applied immediately and remembered when you reopen the launcher.",
  "Sin reiniciar": "No restart needed",
  "Escala de Windows detectada": "Detected Windows scaling",
  "Este ajuste se aplica sobre la escala de tu sistema.": "This setting is applied on top of your system scaling.",
  "Tamaño de la ventana del launcher": "Launcher window size",
  "Elige cuánto espacio ocupa TP LAUNCHER en tu monitor.": "Choose how much space TP LAUNCHER uses on your monitor.",
  "Estas opciones están disponibles en la aplicación de escritorio.": "These options are available in the desktop app.",
  "Resoluciones del launcher": "Launcher window sizes",
  "Ancho": "Width",
  "Alto": "Height",
  "Aplicar tamaño": "Apply size",
  "Ajustar a la pantalla": "Fit to screen",
  "Maximizar": "Maximize",
  "Ventana actual": "Current window",
  "Maximizada": "Maximized",
  "Espacio disponible": "Available space",
  "La resolución de Minecraft se configura por separado en la pestaña Minecraft.": "Minecraft resolution is configured separately in the Minecraft tab.",
  "Quitar logo": "Remove logo",
  "Logo predeterminado restaurado": "Default logo restored",
  "Barra de juego rápido": "Quick launch bar",
  "Mostrar barra de juego": "Show launch bar",
  "Ocultar barra de juego": "Hide launch bar",
  "Nick del jugador": "Player name",
  "Modpack activo": "Active modpack",
  "Filtrar versiones": "Filter versions",
  "Selección actual": "Current selection",
  "Sin versiones disponibles": "No versions available",
  "INSTALANDO…": "INSTALLING…",
  "PREPARANDO…": "PREPARING…",
  "Usar perfil normal": "Use regular profile",
  "Plugins": "Plugins",
  "Plugins del launcher": "Launcher plugins",
  "TU PARTIDA, A TU MANERA": "YOUR GAME, YOUR WAY",
  "Prepara las herramientas que quieres ver mientras juegas.": "Prepare the tools you want to see while playing.",
  "HUD en preparación": "HUD in development",
  "Tu próximo HUD empieza aquí.": "Your next HUD starts here.",
  "Ya puedes diseñarlo y guardar tu perfil. Para verlo dentro de Minecraft añadiremos el complemento de TP LAUNCHER.": "Design your HUD and save a profile. The upcoming TP LAUNCHER companion will display it in Minecraft.",
  "Tu caja de herramientas": "Your toolkit",
  "Así se verá tu HUD": "Your HUD preview",
  "Vista previa · datos de ejemplo": "Preview · sample data",
  "Ajusta tu diseño": "Customize your layout",
  "En tu perfil": "Included in your profile",
  "Sin incluir": "Not included",
  "Incluir en el perfil": "Include in profile",
  "Los plugins están en preparación. Seleccionarlos aquí configura tu futuro HUD.": "Plugins are in development. Select them here to configure your future HUD.",
  "Contador de FPS": "FPS counter",
  "Coordenadas": "Coordinates",
  "Teclas y clics": "Keystrokes and clicks",
  "Equipamiento": "Equipment",
  "Reloj": "Clock",
  "Conexión": "Connection",
  "Consulta la fluidez del juego de un vistazo.": "See how smoothly the game is running.",
  "Ten a mano tu posición mientras exploras.": "Keep track of your position as you explore.",
  "Muestra las teclas de movimiento y los botones del ratón.": "Display movement keys and mouse buttons.",
  "Consulta tu armadura y la duración de tus herramientas.": "Check armor and tool durability.",
  "Ve la hora de tu equipo sin salir de la partida.": "See your local time without leaving the game.",
  "Consulta el tiempo de respuesta del servidor.": "Check the server response time.",
  "Arriba a la izquierda": "Top left",
  "Arriba al centro": "Top center",
  "Arriba a la derecha": "Top right",
  "Abajo a la izquierda": "Bottom left",
  "Abajo al centro": "Bottom center",
  "Abajo a la derecha": "Bottom right",
  "Ubicación": "Position",
  "Nombre del perfil": "Profile name",
  "Tamaño del HUD": "HUD size",
  "Visibilidad": "Visibility",
  "Fondo detrás del texto": "Text background",
  "Guardar perfil del HUD": "Save HUD profile",
  "Cambios sin guardar": "Unsaved changes",
  "Tu perfil está guardado": "Your profile is saved",
  "Cargando tu perfil…": "Loading your profile…",
  "Abrir carpeta de plugins": "Open plugins folder",
  "Exploración": "Exploration",
  "Combate": "Combat",
  "Minimalista": "Minimalist",
  "Empezar con:": "Start with:",
  "Elige una herramienta para añadirla a tu diseño.": "Choose a tool to add to your layout.",
  "Perfil guardado. Tu diseño estará listo para el complemento del juego.": "Profile saved. Your layout will be ready for the game companion.",
  "Día": "Day", "Noche": "Night", "Cuenta y skins": "Account & skins", "Comunidad": "Community", "Contenido": "Content", "TU AVENTURA": "YOUR ADVENTURE", "TU PERSONAJE": "YOUR CHARACTER", "EL LAUNCHER": "THE LAUNCHER", "Tu próximo mundo": "Your next world", "empieza aquí.": "starts here.", "TU MUNDO TE ESPERA": "YOUR WORLD AWAITS", "Prepara tu partida": "Prepare to play",
  "Jugar": "Play", "Mods y modpacks": "Mods & modpacks", "Instalaciones": "Installations", "Cuenta": "Account", "Cuentas": "Accounts", "Estudio 3D": "3D Studio", "Poser": "Poser", "Consola": "Console", "Ajustes": "Settings", "Servidores": "Servers",
  "General": "General", "Idioma": "Language", "Apariencia": "Appearance", "Minecraft": "Minecraft", "Java": "Java", "Cerrar": "Close", "Volver": "Back", "Continuar": "Continue", "Guardar": "Save", "Cancelar": "Cancel", "Eliminar": "Remove", "Activar": "Activate", "Desactivar": "Disable", "Añadir": "Add", "Buscar": "Search", "Actualizar": "Refresh", "Instalar": "Install", "Descargar": "Download", "Abrir": "Open", "Copiar": "Copy", "Copiado": "Copied", "Restablecer": "Reset", "Listo": "Done", "Seleccionar": "Select", "Opcional": "Optional", "Recomendado": "Recommended", "Disponible": "Available", "No instalado": "Not installed", "Activado": "Enabled", "Desactivado": "Disabled", "Cargando…": "Loading…", "Sin resultados": "No results", "Reintentar": "Try again", "Error": "Error", "Advertencia": "Warning", "Información": "Information", "Correcto": "Success",
  "Centro de Mando · TP LAUNCHER": "Command Center · TP LAUNCHER", "Tu universo de Minecraft sin límites.": "Your Minecraft universe, without limits.", "PERFIL ACTIVO": "ACTIVE PROFILE", "VERSIÓN SELECCIONADA": "SELECTED VERSION", "MEMORIA ASIGNADA": "ALLOCATED MEMORY", "TIEMPO JUGADO / SESIÓN": "PLAYTIME / SESSION", "Acceso rápido · jugadas recientemente": "Quick access · recently played", "Ver instalaciones": "View installations", "Jugar de nuevo": "Play again", "NICK DEL JUGADOR · CUENTA MICROSOFT": "PLAYER NAME · MICROSOFT ACCOUNT", "NICK DEL JUGADOR · MODO OFFLINE": "PLAYER NAME · OFFLINE MODE", "VERSIÓN": "VERSION", "Instaladas": "Installed", "Releases": "Releases", "Snapshots": "Snapshots", "Betas / Alphas": "Betas / Alphas", "MEMORIA RAM": "RAM MEMORY", "Modo Boost FPS": "FPS Boost mode", "Optimización JVM con G1GC · puede reducir pausas": "JVM optimization with G1GC · may reduce pauses", "JUGAR AHORA": "PLAY NOW", "Mis modpacks": "My modpacks", "Capturas": "Screenshots", "Mundos": "Worlds", "Carpeta del juego": "Game folder", "Tiempo por versión": "Time by version", "Consola en vivo": "Live console", "Abrir consola completa": "Open full console", "Cuenta Microsoft": "Microsoft account", "Modo Offline": "Offline mode", "Cambiar cuenta": "Switch account", "Cambiar versión Java": "Change Java version",
  "Noticias y novedades": "News & updates", "Noticias de Minecraft": "Minecraft news", "Notas de versión": "Release notes", "Notas de Versión (TP LAUNCHER)": "Release notes (TP LAUNCHER)", "Leer más": "Read more", "Enlaces oficiales y comunidad": "Official & community links", "Todas las noticias": "All news", "No hay noticias disponibles.": "No news available.", "Explorar mods": "Explore mods", "Descubrir modpacks": "Discover modpacks", "Mi biblioteca": "My library", "Crear modpack": "Create modpack", "Importar modpack": "Import modpack", "Exportar modpack": "Export modpack", "Buscar mods": "Search mods", "Buscar modpacks": "Search modpacks", "Todas las versiones": "All versions", "Todos los cargadores": "All loaders", "Todas las categorías": "All categories", "Más descargados": "Most downloaded", "Relevancia": "Relevance", "Fecha de actualización": "Last updated", "Anterior": "Previous", "Siguiente": "Next", "Instalado": "Installed", "Instalar mod": "Install mod", "Instalar modpack": "Install modpack", "Dependencias": "Dependencies", "Dependencias obligatorias": "Required dependencies", "Descripción": "Description", "Versiones": "Versions", "Archivos": "Files", "Autor": "Author", "Versión": "Version", "Tamaño": "Size", "Nombre": "Name", "Nombre del modpack": "Modpack name", "Versión de Minecraft": "Minecraft version", "Cargador de mods": "Mod loader", "Selecciona una instancia": "Select an instance", "Administrar": "Manage", "Jugar esta instancia": "Play this instance", "Abrir carpeta": "Open folder", "Mods instalados": "Installed mods", "Sin mods instalados": "No installed mods", "Importar archivo": "Import file", "Exportar ZIP": "Export ZIP", "Crear": "Create", "Modificar": "Edit", "Selecciona una versión": "Select a version", "Detalles": "Details", "Sin descripción disponible.": "No description available.", "Ambos · unificado": "Both · unified", "Sin conexión": "Offline", "Conectado": "Connected",
  "Instalaciones de Minecraft": "Minecraft installations", "Biblioteca de instalaciones": "Installation library", "Versiones instaladas": "Installed versions", "Nueva instalación": "New installation", "Nueva instalación Vanilla": "New Vanilla installation", "Descargar y jugar": "Download & play", "Crear o administrar": "Create or manage", "Último inicio": "Last launched", "Buscar instalaciones": "Search installations", "Nombre de la instalación": "Installation name", "Descargar versión": "Download version", "Reparar archivos": "Repair files", "Ver carpeta": "View folder",
  "Iniciar sesión con Microsoft": "Sign in with Microsoft", "Cerrar sesión": "Sign out", "Microsoft Premium": "Microsoft Premium", "Offline": "Offline", "Cuenta activa": "Active account", "Sin sesión": "Signed out", "Sesión guardada": "Saved session", "Activar cuenta": "Activate account", "Añadir otra cuenta": "Add another account", "Añadir cuenta Microsoft": "Add Microsoft account", "Añadir perfil Offline": "Add offline profile", "Nombre de jugador": "Player name", "Las credenciales se guardan de forma segura en este dispositivo.": "Credentials are stored securely on this device.", "Las cuentas Offline no autentican una licencia de Minecraft.": "Offline accounts do not authenticate a Minecraft license.", "Sin cuentas guardadas": "No saved accounts", "Eliminar cuenta": "Remove account", "Quitar esta cuenta del dispositivo": "Remove this account from this device", "Biblioteca de skins": "Skin library", "Subir skin PNG": "Upload PNG skin", "Modelo Classic": "Classic model", "Modelo Slim": "Slim model", "Capas oficiales": "Official capes", "Capa activa": "Active cape", "Usar esta capa": "Use this cape", "Ocultar capa en el perfil": "Hide cape on profile", "Avatar": "Avatar", "Vista previa": "Preview", "Guardar skin": "Save skin", "Aplicar skin": "Apply skin", "Importar skin": "Import skin", "Buscar por nick": "Search by name", "Segunda capa": "Outer layer", "Sin capa": "No cape", "Restaurar skin": "Restore skin",
  "Estudio de Renderizado y Poses 3D": "3D Rendering & Pose Studio", "Catálogo de poses": "Pose catalog", "Buscar una pose…": "Search poses…", "Pose aleatoria": "Random pose", "Respiración": "Breathing", "Cabeza": "Head", "Torso": "Body", "Brazo derecho": "Right arm", "Brazo izquierdo": "Left arm", "Pierna derecha": "Right leg", "Pierna izquierda": "Left leg", "Capa": "Cape", "Iluminación": "Lighting", "Cámara": "Camera", "Objeto en mano": "Held item", "Ninguno": "None", "Espada de Diamante": "Diamond Sword", "Espada de Netherite": "Netherite Sword", "Pico de Diamante": "Diamond Pickaxe", "Tótem de la Inmortalidad": "Totem of Undying", "Escudo": "Shield", "Tridente": "Trident", "Arco": "Bow", "Manzana Dorada Encantada": "Enchanted Golden Apple", "Antorcha": "Torch", "Linterna": "Lantern", "Perla de Ender": "Ender Pearl", "Bloque de Césped": "Grass Block", "Descargar Render PNG HD": "Download HD PNG render", "Copiar imagen al portapapeles": "Copy image to clipboard", "Guardar pose": "Save pose", "Nombre de la pose": "Pose name", "Poses personalizadas": "Custom poses", "Créditos": "Credits", "Abrir Estudio 3D": "Open 3D Studio",
  "Personaliza tu experiencia": "Make it yours", "Preferencias del launcher, el juego y tus cuentas.": "Launcher, game and account preferences.", "Comportamiento al iniciar Minecraft": "When Minecraft starts", "Mantener launcher abierto": "Keep launcher open", "Minimizar ventana": "Minimize window", "Minimizar a la bandeja": "Minimize to tray", "Cerrar launcher": "Close launcher", "Carpeta raíz de datos": "Root data folder", "Cambiar carpeta": "Change folder", "Aplicar carpeta": "Apply folder", "Discord Rich Presence": "Discord Rich Presence", "Compartir el estado del juego en Discord": "Share game activity on Discord", "Actualizaciones automáticas": "Automatic updates", "Buscar actualizaciones al abrir el launcher": "Check for updates when the launcher starts", "Secretos descubiertos": "Secrets discovered", "Repetir asistente de bienvenida": "Run setup wizard again", "Idioma de la interfaz": "Interface language", "Buscar idioma o región…": "Search language or region…", "Región / Código": "Region / Code", "Completado": "Complete", "Detectar idioma del sistema": "Detect system language", "Cobertura del catálogo traducible. Los textos pendientes se muestran en español.": "Coverage of the translatable catalog. Untranslated text falls back to Spanish.", "Galería de Temas Oficiales": "Official Theme Gallery", "Índigo Profundo": "Deep Indigo", "Amatista Real": "Royal Amethyst", "Nébula Violeta": "Violet Nebula", "Cristal Carmesí": "Crimson Crystal", "Cian Técnico": "Technical Cyan", "Verde Esmeralda": "Emerald Green", "Ámbar Dorado": "Golden Amber", "Azul Zafiro": "Sapphire Blue", "Cuarzo Sakura": "Sakura Quartz", "Ocaso Synthwave": "Synthwave Sunset", "Aurora Boreal": "Northern Lights", "Hielo Ártico": "Arctic Ice", "Oro Rosa": "Rose Gold", "Bosque Matrix": "Matrix Forest", "Obsidiana Neutro": "Neutral Obsidian", "Paleta a Medida": "Custom Palette", "Colores personalizados": "Custom colors", "Acento principal": "Primary accent", "Acento secundario": "Secondary accent", "Fondo base": "Base background", "Transparencia": "Transparency", "Fantasma": "Ghost", "Equilibrado": "Balanced", "Lectura": "Reading", "Sólido": "Solid", "Opacidad del fondo": "Background opacity", "Opacidad del menú": "Menu opacity", "Opacidad de barra": "Bar opacity", "Opacidad de tarjetas": "Card opacity", "Desenfoque": "Blur", "Fondo personalizado": "Custom background", "Elegir fondo": "Choose background", "Quitar fondo": "Remove background", "Logo personalizado": "Custom logo", "Elegir imagen": "Choose image", "Se ajusta automáticamente a 128 × 128.": "Automatically resized to 128 × 128.",
  "Ventana del juego": "Game window", "Resolución predeterminada": "Default resolution", "Iniciar en pantalla completa": "Start in fullscreen", "Memoria global": "Global memory", "Optimización JVM": "JVM optimization", "Activa G1GC para reducir pausas. No garantiza más FPS.": "Enable G1GC to reduce pauses. Higher FPS is not guaranteed.", "Accesos rápidos": "Quick access", "Motores Java": "Java runtimes", "Detectar Java": "Detect Java", "Escaneando…": "Scanning…", "Descargar runtime": "Download runtime", "Probar ejecutable Java": "Test Java executable", "Probar": "Test", "Ruta de Java": "Java path", "Java del sistema": "System Java", "No se detectaron runtimes de Java.": "No Java runtimes detected.", "El launcher selecciona un runtime compatible según el manifiesto de cada versión.": "The launcher selects a compatible runtime using each version's manifest.", "Memoria del equipo": "System memory", "Configuración lista": "Setup complete", "Bienvenido a TP LAUNCHER": "Welcome to TP LAUNCHER", "Prepara tu próxima aventura en tres pasos.": "Get ready for your next adventure in three steps.", "Selecciona tu idioma": "Choose your language", "Java y memoria": "Java & memory", "Tu estilo y tu cuenta": "Your style & account", "Paso": "Step", "Conectar Microsoft": "Connect Microsoft", "Continuar sin cuenta": "Continue without an account", "Empezar a jugar": "Start playing", "Puedes ajustar estas opciones más tarde en Ajustes.": "You can change these options later in Settings.", "Java 8 · clásicos y versiones antiguas": "Java 8 · classics and older versions", "Java 17 · Minecraft 1.18 a 1.20.4": "Java 17 · Minecraft 1.18 to 1.20.4", "Java 21 · Minecraft moderno": "Java 21 · modern Minecraft", "Java 25 · nuevas versiones y snapshots": "Java 25 · new versions and snapshots", "Instalar los runtimes faltantes": "Install missing runtimes", "Memoria recomendada": "Recommended memory", "Usar recomendación": "Use recommendation", "Conectando…": "Connecting…", "Preparando…": "Preparing…", "Aplicando…": "Applying…", "Guardando…": "Saving…", "Cambios guardados": "Changes saved",
  "Registro en vivo": "Live log", "Limpiar consola": "Clear console", "Copiar registro": "Copy log", "Exportar registro": "Export log", "Seguimiento automático": "Auto scroll", "Todos los registros": "All logs", "Solo errores": "Errors only", "Diagnóstico": "Diagnostics", "El juego terminó correctamente.": "The game closed successfully.", "El juego se cerró con un error.": "The game closed with an error.", "No se pudo iniciar Minecraft.": "Minecraft could not be started.", "Revisa el registro para conocer la causa.": "Check the log for the cause.", "No se pudo descargar el archivo.": "The file could not be downloaded.", "Conexión de red no disponible.": "Network connection unavailable.", "Esta versión necesita un Java más reciente.": "This version requires a newer Java runtime.", "El servidor rechazó la solicitud.": "The server rejected the request.", "Falta una biblioteca necesaria.": "A required library is missing.", "Sesión caducada. Vuelve a iniciar sesión.": "Session expired. Sign in again.", "Sin permisos para acceder a la carpeta.": "Permission denied for this folder.", "La descarga no pasó la comprobación de integridad.": "The download failed the integrity check.", "Espacio insuficiente en disco.": "Not enough disk space.", "Servidor añadido": "Server added", "Añadir servidor": "Add server", "Dirección del servidor": "Server address", "Entrar directo al servidor": "Join server directly", "Jugadores": "Players", "Latencia": "Latency", "Alojar Mundo LAN / Túnel Multijugador": "Host LAN World / Multiplayer Tunnel", "Puerto local": "Local port", "Conexión local": "Local connection", "Crear túnel": "Create tunnel", "Detener túnel": "Stop tunnel", "Copiar dirección": "Copy address", "Estado": "Status", "Servidor sin conexión": "Server offline", "Mover arriba": "Move up", "Mover abajo": "Move down", "Actualizar estado": "Refresh status", "Permitir conexiones de amigos": "Allow friends to connect", "Revisar actualizaciones": "Check for updates", "No hay cuentas disponibles.": "No accounts available.", "Elige una cuenta para esta sesión.": "Choose an account for this session.", "Esta acción conserva tus mundos y archivos.": "This keeps your worlds and files.", "Abrir ubicación": "Open location", "Pendiente": "Pending", "Restaurar valores": "Restore defaults", "Se requiere reiniciar el launcher.": "Restart the launcher to apply this change.", "La configuración se guarda automáticamente.": "Settings are saved automatically.", "Conectar cuenta": "Connect account"
};


const commonKeys = ["Jugar", "Mods y modpacks", "Instalaciones", "Cuenta", "Cuentas", "Estudio 3D", "Consola", "Ajustes", "Servidores", "General", "Idioma", "Apariencia", "Cerrar", "Volver", "Continuar", "Guardar", "Cancelar", "Eliminar", "Activar", "Desactivar", "Añadir", "Buscar", "Actualizar", "Instalar", "Descargar", "Abrir", "Copiar", "Listo", "Seleccionar", "Recomendado", "Disponible", "No instalado", "Activado", "Desactivado", "Cargando…", "Sin resultados", "Reintentar", "Error", "Advertencia", "Información", "Nombre", "Versión", "Tamaño", "Autor", "Descripción", "Anterior", "Siguiente", "Cuenta activa", "Sin sesión", "Activar cuenta", "Añadir otra cuenta", "Nombre de jugador", "Idioma de la interfaz", "Completado", "Detectar idioma del sistema", "Galería de Temas Oficiales", "Colores personalizados", "Acento principal", "Acento secundario", "Fondo base", "Transparencia", "Desenfoque", "Fondo personalizado", "Logo personalizado", "Resolución predeterminada", "Iniciar en pantalla completa", "Memoria global", "Accesos rápidos", "Motores Java", "Detectar Java", "Descargar runtime", "Probar ejecutable Java", "Cambios guardados", "Bienvenido a TP LAUNCHER", "Selecciona tu idioma", "Java y memoria", "Tu estilo y tu cuenta", "Conectar Microsoft", "Continuar sin cuenta", "Empezar a jugar", "Paso", "Noticias y novedades", "Mis modpacks", "Capturas", "Mundos", "Carpeta del juego", "JUGAR AHORA", "Consola en vivo", "Cambiar cuenta", "Iniciar sesión con Microsoft", "Cerrar sesión", "Capas oficiales", "Vista previa", "Créditos", "Sin conexión", "Conectado"] as const;

const rows: Record<string, string[]> = {
  "pt-BR": ["Jogar","Mods e modpacks","Instalações","Conta","Contas","Estúdio 3D","Console","Configurações","Servidores","Geral","Idioma","Aparência","Fechar","Voltar","Continuar","Salvar","Cancelar","Excluir","Ativar","Desativar","Adicionar","Pesquisar","Atualizar","Instalar","Baixar","Abrir","Copiar","Pronto","Selecionar","Recomendado","Disponível","Não instalado","Ativado","Desativado","Carregando…","Sem resultados","Tentar novamente","Erro","Aviso","Informação","Nome","Versão","Tamanho","Autor","Descrição","Anterior","Próximo","Conta ativa","Sem sessão","Ativar conta","Adicionar outra conta","Nome do jogador","Idioma da interface","Concluído","Detectar idioma do sistema","Galeria de temas oficiais","Cores personalizadas","Destaque principal","Destaque secundário","Fundo base","Transparência","Desfoque","Fundo personalizado","Logo personalizado","Resolução padrão","Iniciar em tela cheia","Memória global","Acessos rápidos","Runtimes Java","Detectar Java","Baixar runtime","Testar executável Java","Alterações salvas","Boas-vindas ao TP LAUNCHER","Selecione seu idioma","Java e memória","Seu estilo e sua conta","Conectar Microsoft","Continuar sem conta","Começar a jogar","Etapa","Notícias e novidades","Meus modpacks","Capturas","Mundos","Pasta do jogo","JOGAR AGORA","Console ao vivo","Trocar conta","Entrar com Microsoft","Sair","Capas oficiais","Prévia","Créditos","Desconectado","Conectado"],
  "fr-FR": ["Jouer","Mods et modpacks","Installations","Compte","Comptes","Studio 3D","Console","Paramètres","Serveurs","Général","Langue","Apparence","Fermer","Retour","Continuer","Enregistrer","Annuler","Supprimer","Activer","Désactiver","Ajouter","Rechercher","Actualiser","Installer","Télécharger","Ouvrir","Copier","Terminé","Sélectionner","Recommandé","Disponible","Non installé","Activé","Désactivé","Chargement…","Aucun résultat","Réessayer","Erreur","Avertissement","Information","Nom","Version","Taille","Auteur","Description","Précédent","Suivant","Compte actif","Déconnecté","Activer le compte","Ajouter un autre compte","Nom du joueur","Langue de l’interface","Terminé","Détecter la langue du système","Galerie des thèmes officiels","Couleurs personnalisées","Accent principal","Accent secondaire","Fond de base","Transparence","Flou","Fond personnalisé","Logo personnalisé","Résolution par défaut","Démarrer en plein écran","Mémoire globale","Accès rapides","Environnements Java","Détecter Java","Télécharger Java","Tester l’exécutable Java","Modifications enregistrées","Bienvenue sur TP LAUNCHER","Choisissez votre langue","Java et mémoire","Votre style et votre compte","Connecter Microsoft","Continuer sans compte","Commencer à jouer","Étape","Actualités et nouveautés","Mes modpacks","Captures","Mondes","Dossier du jeu","JOUER MAINTENANT","Console en direct","Changer de compte","Se connecter avec Microsoft","Se déconnecter","Capes officielles","Aperçu","Crédits","Hors connexion","Connecté"],
  "de-DE": ["Spielen","Mods und Modpacks","Installationen","Konto","Konten","3D-Studio","Konsole","Einstellungen","Server","Allgemein","Sprache","Darstellung","Schließen","Zurück","Weiter","Speichern","Abbrechen","Entfernen","Aktivieren","Deaktivieren","Hinzufügen","Suchen","Aktualisieren","Installieren","Herunterladen","Öffnen","Kopieren","Fertig","Auswählen","Empfohlen","Verfügbar","Nicht installiert","Aktiviert","Deaktiviert","Wird geladen…","Keine Ergebnisse","Erneut versuchen","Fehler","Warnung","Information","Name","Version","Größe","Autor","Beschreibung","Zurück","Weiter","Aktives Konto","Abgemeldet","Konto aktivieren","Weiteres Konto hinzufügen","Spielername","Sprache der Oberfläche","Abgeschlossen","Systemsprache erkennen","Offizielle Designgalerie","Eigene Farben","Primärer Akzent","Sekundärer Akzent","Grundhintergrund","Transparenz","Unschärfe","Eigener Hintergrund","Eigenes Logo","Standardauflösung","Im Vollbild starten","Globaler Arbeitsspeicher","Schnellzugriffe","Java-Laufzeiten","Java erkennen","Laufzeit herunterladen","Java-Programm testen","Änderungen gespeichert","Willkommen bei TP LAUNCHER","Wähle deine Sprache","Java und Arbeitsspeicher","Dein Stil und dein Konto","Mit Microsoft verbinden","Ohne Konto fortfahren","Spielen beginnen","Schritt","Neuigkeiten","Meine Modpacks","Screenshots","Welten","Spielordner","JETZT SPIELEN","Live-Konsole","Konto wechseln","Mit Microsoft anmelden","Abmelden","Offizielle Umhänge","Vorschau","Mitwirkende","Offline","Verbunden"],
  "it-IT": ["Gioca","Mod e modpack","Installazioni","Account","Account","Studio 3D","Console","Impostazioni","Server","Generale","Lingua","Aspetto","Chiudi","Indietro","Continua","Salva","Annulla","Rimuovi","Attiva","Disattiva","Aggiungi","Cerca","Aggiorna","Installa","Scarica","Apri","Copia","Fatto","Seleziona","Consigliato","Disponibile","Non installato","Attivato","Disattivato","Caricamento…","Nessun risultato","Riprova","Errore","Avviso","Informazione","Nome","Versione","Dimensione","Autore","Descrizione","Precedente","Successivo","Account attivo","Disconnesso","Attiva account","Aggiungi un altro account","Nome del giocatore","Lingua dell’interfaccia","Completato","Rileva lingua del sistema","Galleria dei temi ufficiali","Colori personalizzati","Accento primario","Accento secondario","Sfondo di base","Trasparenza","Sfocatura","Sfondo personalizzato","Logo personalizzato","Risoluzione predefinita","Avvia a schermo intero","Memoria globale","Accessi rapidi","Runtime Java","Rileva Java","Scarica runtime","Prova eseguibile Java","Modifiche salvate","Benvenuto in TP LAUNCHER","Seleziona la lingua","Java e memoria","Il tuo stile e il tuo account","Connetti Microsoft","Continua senza account","Inizia a giocare","Passaggio","Notizie e novità","I miei modpack","Schermate","Mondi","Cartella di gioco","GIOCA ORA","Console in tempo reale","Cambia account","Accedi con Microsoft","Esci","Mantelli ufficiali","Anteprima","Crediti","Non connesso","Connesso"],
  "ru-RU": ["Играть","Моды и сборки","Установки","Аккаунт","Аккаунты","3D-студия","Консоль","Настройки","Серверы","Общие","Язык","Внешний вид","Закрыть","Назад","Продолжить","Сохранить","Отмена","Удалить","Активировать","Отключить","Добавить","Поиск","Обновить","Установить","Скачать","Открыть","Копировать","Готово","Выбрать","Рекомендуется","Доступно","Не установлено","Включено","Отключено","Загрузка…","Нет результатов","Повторить","Ошибка","Предупреждение","Информация","Название","Версия","Размер","Автор","Описание","Назад","Далее","Активный аккаунт","Вход не выполнен","Активировать аккаунт","Добавить аккаунт","Имя игрока","Язык интерфейса","Завершено","Определить язык системы","Галерея официальных тем","Свои цвета","Основной акцент","Вторичный акцент","Основной фон","Прозрачность","Размытие","Свой фон","Свой логотип","Разрешение по умолчанию","Запускать на весь экран","Общая память","Быстрый доступ","Среды Java","Найти Java","Скачать среду","Проверить Java","Изменения сохранены","Добро пожаловать в TP LAUNCHER","Выберите язык","Java и память","Оформление и аккаунт","Подключить Microsoft","Продолжить без аккаунта","Начать игру","Шаг","Новости и обновления","Мои сборки","Снимки экрана","Миры","Папка игры","ИГРАТЬ СЕЙЧАС","Живая консоль","Сменить аккаунт","Войти с Microsoft","Выйти","Официальные плащи","Просмотр","Благодарности","Нет связи","Подключено"],
  "ja-JP": ["プレイ","MODとMODパック","インストール","アカウント","アカウント","3Dスタジオ","コンソール","設定","サーバー","一般","言語","外観","閉じる","戻る","続行","保存","キャンセル","削除","有効化","無効化","追加","検索","更新","インストール","ダウンロード","開く","コピー","完了","選択","推奨","利用可能","未インストール","有効","無効","読み込み中…","結果なし","再試行","エラー","警告","情報","名前","バージョン","サイズ","作者","説明","前へ","次へ","使用中のアカウント","未ログイン","アカウントを有効化","別のアカウントを追加","プレイヤー名","表示言語","完了率","システム言語を検出","公式テーマギャラリー","カスタムカラー","メインカラー","サブカラー","背景色","透明度","ぼかし","カスタム背景","カスタムロゴ","既定の解像度","全画面で起動","共通メモリ","クイックアクセス","Javaランタイム","Javaを検出","ランタイムをダウンロード","Java実行ファイルをテスト","変更を保存しました","TP LAUNCHERへようこそ","言語を選択","Javaとメモリ","外観とアカウント","Microsoftに接続","アカウントなしで続行","プレイを開始","ステップ","ニュースと更新","自分のMODパック","スクリーンショット","ワールド","ゲームフォルダー","今すぐプレイ","ライブコンソール","アカウント切替","Microsoftでログイン","ログアウト","公式マント","プレビュー","クレジット","オフライン","接続済み"],
  "ko-KR": ["플레이","모드 및 모드팩","설치 목록","계정","계정","3D 스튜디오","콘솔","설정","서버","일반","언어","모양","닫기","뒤로","계속","저장","취소","삭제","활성화","비활성화","추가","검색","새로 고침","설치","다운로드","열기","복사","완료","선택","권장","사용 가능","설치되지 않음","활성화됨","비활성화됨","불러오는 중…","결과 없음","다시 시도","오류","경고","정보","이름","버전","크기","제작자","설명","이전","다음","활성 계정","로그인하지 않음","계정 활성화","다른 계정 추가","플레이어 이름","인터페이스 언어","완료율","시스템 언어 감지","공식 테마 갤러리","사용자 색상","기본 강조색","보조 강조색","기본 배경","투명도","흐림","사용자 배경","사용자 로고","기본 해상도","전체 화면으로 시작","전역 메모리","빠른 접근","Java 런타임","Java 감지","런타임 다운로드","Java 실행 파일 테스트","변경 사항 저장됨","TP LAUNCHER에 오신 것을 환영합니다","언어 선택","Java 및 메모리","스타일 및 계정","Microsoft 연결","계정 없이 계속","플레이 시작","단계","뉴스 및 업데이트","내 모드팩","스크린샷","월드","게임 폴더","지금 플레이","실시간 콘솔","계정 변경","Microsoft로 로그인","로그아웃","공식 망토","미리 보기","크레딧","오프라인","연결됨"],
  "zh-CN": ["开始游戏","模组与整合包","安装管理","账户","账户","3D工作室","控制台","设置","服务器","常规","语言","外观","关闭","返回","继续","保存","取消","删除","启用","禁用","添加","搜索","刷新","安装","下载","打开","复制","完成","选择","推荐","可用","未安装","已启用","已禁用","加载中…","无结果","重试","错误","警告","信息","名称","版本","大小","作者","描述","上一页","下一页","当前账户","未登录","启用账户","添加其他账户","玩家名称","界面语言","完成度","检测系统语言","官方主题库","自定义颜色","主要强调色","次要强调色","基础背景色","透明度","模糊","自定义背景","自定义标志","默认分辨率","以全屏模式启动","全局内存","快捷访问","Java运行环境","检测Java","下载运行环境","测试Java可执行文件","更改已保存","欢迎使用TP LAUNCHER","选择语言","Java与内存","外观和账户","连接Microsoft","不登录继续","开始游戏","步骤","新闻与更新","我的整合包","截图","世界","游戏文件夹","立即开始游戏","实时控制台","切换账户","使用Microsoft登录","退出登录","官方披风","预览","鸣谢","离线","已连接"],
  "ca-ES": ["Juga","Mods i paquets","Instal·lacions","Compte","Comptes","Estudi 3D","Consola","Configuració","Servidors","General","Llengua","Aparença","Tanca","Enrere","Continua","Desa","Cancel·la","Elimina","Activa","Desactiva","Afegeix","Cerca","Actualitza","Instal·la","Baixa","Obre","Copia","Fet","Selecciona","Recomanat","Disponible","No instal·lat","Activat","Desactivat","S’està carregant…","Sense resultats","Torna-ho a provar","Error","Avís","Informació","Nom","Versió","Mida","Autor","Descripció","Anterior","Següent","Compte actiu","Sense sessió","Activa el compte","Afegeix un altre compte","Nom del jugador","Llengua de la interfície","Completat","Detecta la llengua del sistema","Galeria de temes oficials","Colors personalitzats","Accent principal","Accent secundari","Fons base","Transparència","Desenfocament","Fons personalitzat","Logotip personalitzat","Resolució predeterminada","Inicia a pantalla completa","Memòria global","Accessos ràpids","Entorns Java","Detecta Java","Baixa l’entorn","Prova l’executable Java","Canvis desats","Us donem la benvinguda a TP LAUNCHER","Selecciona la llengua","Java i memòria","El teu estil i el teu compte","Connecta Microsoft","Continua sense compte","Comença a jugar","Pas","Notícies i novetats","Els meus paquets","Captures","Mons","Carpeta del joc","JUGA ARA","Consola en directe","Canvia de compte","Inicia sessió amb Microsoft","Tanca la sessió","Capes oficials","Previsualització","Crèdits","Sense connexió","Connectat"],
  "eo-UY": ["Ludi","Modifoj kaj pakoj","Instalaĵoj","Konto","Kontoj","3D-studio","Konzolo","Agordoj","Serviloj","Ĝenerale","Lingvo","Aspekto","Fermi","Reen","Daŭrigi","Konservi","Nuligi","Forigi","Aktivigi","Malaktivigi","Aldoni","Serĉi","Refreŝigi","Instali","Elŝuti","Malfermi","Kopii","Preta","Elekti","Rekomendita","Disponebla","Ne instalita","Aktiva","Malaktiva","Ŝargante…","Neniuj rezultoj","Reprovi","Eraro","Averto","Informo","Nomo","Versio","Grando","Aŭtoro","Priskribo","Antaŭa","Sekva","Aktiva konto","Ne ensalutinta","Aktivigi konton","Aldoni alian konton","Ludantnomo","Lingvo de la interfaco","Kompletigita","Detekti sisteman lingvon","Galerio de oficialaj etosoj","Propraj koloroj","Ĉefa akcento","Dua akcento","Baza fono","Travidebleco","Malklarigo","Propra fono","Propra emblemo","Defaŭlta distingivo","Lanĉi plenekrane","Ĝenerala memoro","Rapidaj aliroj","Java-rultempoj","Detekti Java","Elŝuti rultempon","Provi Java-programon","Ŝanĝoj konservitaj","Bonvenon al TP LAUNCHER","Elektu vian lingvon","Java kaj memoro","Via stilo kaj via konto","Konekti Microsoft","Daŭrigi sen konto","Komenci ludi","Paŝo","Novaĵoj","Miaj modifpakoj","Ekrankopioj","Mondoj","Luddosierujo","LUDU NUN","Viva konzolo","Ŝanĝi konton","Ensaluti per Microsoft","Elsaluti","Oficialaj manteloj","Antaŭrigardo","Dankoj","Nekonektita","Konektita"],
  "en-PT": ["Set sail","Mods an’ modpacks","Shipyards","Sailor","Crew accounts","3D deck","Ship’s log","Ship fittings","Harbours","General orders","Tongue","Colours","Batten down","Astern","Onward","Stow","Belay","Scuttle","Hoist","Lower","Recruit","Seek","Refresh charts","Fit aboard","Haul aboard","Open hold","Copy chart","Aye","Choose","Captain’s choice","Ready aboard","Not aboard","Hoisted","Lowered","Haulin’…","No treasure found","Try again, matey","Trouble","Lookout","Ship’s news","Name","Edition","Weight","Maker","Tale","Astern","Ahead","Sailor on deck","No sailor aboard","Put sailor on deck","Recruit another sailor","Sailor’s name","Crew tongue","Charted","Find the ship’s tongue","Official ship colours","Custom colours","Foremast colour","Mainmast colour","Ocean colour","Clear waters","Sea mist","Custom horizon","Ship’s crest","Porthole size","Fill the horizon","Ship’s memory","Quick ports","Java engines","Find Java","Haul Java aboard","Test Java engine","Orders stowed","Welcome aboard TP LAUNCHER","Choose yer tongue","Java an’ memory","Colours an’ crew","Board with Microsoft","Sail without a sailor","Set sail now","Leg","News from the seven seas","Me modpacks","Sea portraits","Worlds","Ship’s hold","SET SAIL NOW","Live ship’s log","Change sailor","Board with Microsoft","Leave the ship","Official capes","Spyglass view","Honours","Lost at sea","Anchored"],
  "lol-US": ["Plez play","Modz n packz","Installz","Kitteh account","Kitteh accountz","3D catwalk","Logz","Settingz","Serverz","Generalz","Langwage","Shiny lookz","Bai","Go bak","Moar","Savez","Nope","Remuv","Turn onz","Turn offz","Add moar","Findz","Freshen","Installz","Gimme filez","Openz","Copycat","All dun","Pick one","Gud choice","Haz it","No haz it","Is on","Is off","Waitin…","No can haz resultz","Try agin","Oopsie","Watch outz","FYI kitteh","Naym","Vershun","How bigz","Maker kitteh","Wut iz it","Befor","Next plz","Current kitteh","No kitteh here","Use dis kitteh","Moar kitteh accountz","Kitteh naym","Talky langwage","Dun","Find computer talkz","Official shiny lookz","Mai own colourz","Main shine","Other shine","Back colour","See thru","Fuzziness","Mai backdrop","Mai logo","Window bignez","Take all screenz","Memory noms","Quick thingz","Java enginez","Find Java plz","Gimme Java","Test Java thingy","Saved, kthx","Hai TP LAUNCHER kitteh","Pick ur talkz","Java n memory noms","Ur lookz n account","Microsoft plz","Go wif no account","Play nao","Stepz","New shiny stuffz","Mai modpackz","Picz","Worldz","Game filez","PLAY NAO","Live logz","Change kitteh","Login wif Microsoft","Bai account","Official cape nomz","Sneak peek","Thx kittehz","No interwebz","Haz interwebz"]
};

export const partialCatalogs: Record<string, Record<string, string>> = Object.fromEntries(
  Object.entries(rows).map(([code, values]) => {
    if (values.length !== commonKeys.length) throw new Error(`Locale ${code}: ${values.length} translations for ${commonKeys.length} keys`);
    return [code, Object.fromEntries(commonKeys.map((key, index) => [key, values[index]]))];
  })
);

Object.assign(english, {
  'Usa una imagen PNG, JPG o WebP de hasta 12 MB.': 'Use a PNG, JPG or WebP image up to 12 MB.',
  'No se pudo leer la imagen.': 'The image could not be read.',
  'Usa entre 3 y 16 letras, números o guiones bajos.': 'Use 3 to 16 letters, numbers or underscores.',
  'Elige cómo se comporta la ventana del launcher cuando abres un juego.': 'Choose how the launcher window behaves when a game starts.',
  'Los registros de Minecraft se conservan en disco.': 'Minecraft logs are kept on disk.',
  'Versiones, runtimes, instancias y archivos del launcher.': 'Versions, runtimes, instances and launcher files.',
  'Integraciones y actualizaciones': 'Integrations & updates',
  'ID de aplicación de Discord': 'Discord application ID',
  'Usa el ID de tu aplicación de Discord para publicar la actividad.': 'Use your Discord application ID to publish activity.',
  'Hay historias que solo aparecen cuando menos las esperas.': 'Some stories appear when you least expect them.',
  'Asistente de bienvenida': 'Setup wizard',
  'Revisa tu idioma, memoria, motores Java y apariencia en tres pasos.': 'Review your language, memory, Java runtimes and appearance in three steps.',
  'Paletas diseñadas para acompañar cada sesión.': 'Palettes designed to complement every session.',
  'Recolector de memoria': 'Garbage collector',
  'ZGC requiere Java 17 o superior. El launcher valida la compatibilidad antes de iniciar.': 'ZGC requires Java 17 or newer. The launcher checks compatibility before starting.',
  'Activa Modo Boost FPS para elegir el recolector de memoria.': 'Enable FPS Boost mode to choose the garbage collector.',
  'Tu próxima aventura empieza aquí.': 'Your next adventure starts here.',
  'Las versiones faltantes también se pueden instalar al jugar.': 'Missing runtimes can also be installed when you play.',
  'Sincroniza tu perfil de Minecraft, tu skin y tus capas oficiales.': 'Sync your Minecraft profile, skin and official capes.',
  'También puedes continuar con Steve y añadir cuentas desde Ajustes.': 'You can also continue with Steve and add accounts in Settings.',
  'Registro en vivo de descargas, runtime Java y salida estándar/error de Minecraft.': 'Live downloads, Java runtime and Minecraft standard/error output.',
  'Sin registros todavía.': 'No logs yet.',
  'Todos': 'All', 'Errores': 'Errors', 'Limpiar': 'Clear', 'Exportar': 'Export', 'Filtrar registros…': 'Filter logs…',
  'Pausar seguimiento': 'Pause auto scroll', 'Reanudar seguimiento': 'Resume auto scroll',
  'Error de autenticación': 'Authentication error', 'Archivo no encontrado': 'File not found',
  'La versión requiere otro runtime Java.': 'This version requires a different Java runtime.',
  'No se pudo cargar el catálogo.': 'The catalog could not be loaded.'
});

const diagnosticKeys = ['El juego terminó correctamente.', 'El juego se cerró con un error.', 'No se pudo iniciar Minecraft.', 'Revisa el registro para conocer la causa.', 'No se pudo descargar el archivo.', 'Conexión de red no disponible.', 'Esta versión necesita un Java más reciente.', 'El servidor rechazó la solicitud.', 'Falta una biblioteca necesaria.', 'Sesión caducada. Vuelve a iniciar sesión.', 'Sin permisos para acceder a la carpeta.', 'La descarga no pasó la comprobación de integridad.', 'Espacio insuficiente en disco.', 'Estado', 'Secretos descubiertos'] as const;
const diagnosticRows: Record<string, string[]> = {
  'pt-BR': ['O jogo foi fechado corretamente.', 'O jogo foi fechado com um erro.', 'Não foi possível iniciar Minecraft.', 'Consulte o registro para saber a causa.', 'Não foi possível baixar o arquivo.', 'Conexão de rede indisponível.', 'Esta versão precisa de um Java mais recente.', 'O servidor rejeitou a solicitação.', 'Uma biblioteca necessária está ausente.', 'Sessão expirada. Entre novamente.', 'Sem permissão para acessar a pasta.', 'O download falhou na verificação de integridade.', 'Espaço insuficiente em disco.', 'Estado', 'Segredos descobertos'],
  'fr-FR': ['Le jeu s’est fermé correctement.', 'Le jeu s’est fermé avec une erreur.', 'Impossible de démarrer Minecraft.', 'Consultez le journal pour connaître la cause.', 'Impossible de télécharger le fichier.', 'Connexion réseau indisponible.', 'Cette version nécessite un Java plus récent.', 'Le serveur a refusé la requête.', 'Une bibliothèque nécessaire est manquante.', 'Session expirée. Reconnectez-vous.', 'Accès au dossier refusé.', 'Le téléchargement a échoué au contrôle d’intégrité.', 'Espace disque insuffisant.', 'État', 'Secrets découverts'],
  'de-DE': ['Das Spiel wurde ordnungsgemäß beendet.', 'Das Spiel wurde mit einem Fehler beendet.', 'Minecraft konnte nicht gestartet werden.', 'Die Ursache steht im Protokoll.', 'Die Datei konnte nicht heruntergeladen werden.', 'Keine Netzwerkverbindung verfügbar.', 'Diese Version benötigt ein neueres Java.', 'Der Server hat die Anfrage abgelehnt.', 'Eine erforderliche Bibliothek fehlt.', 'Sitzung abgelaufen. Bitte erneut anmelden.', 'Keine Berechtigung für diesen Ordner.', 'Die Integritätsprüfung des Downloads ist fehlgeschlagen.', 'Nicht genug Speicherplatz.', 'Status', 'Entdeckte Geheimnisse'],
  'it-IT': ['Il gioco si è chiuso correttamente.', 'Il gioco si è chiuso con un errore.', 'Impossibile avviare Minecraft.', 'Consulta il registro per conoscere la causa.', 'Impossibile scaricare il file.', 'Connessione di rete non disponibile.', 'Questa versione richiede un Java più recente.', 'Il server ha rifiutato la richiesta.', 'Manca una libreria necessaria.', 'Sessione scaduta. Accedi di nuovo.', 'Accesso alla cartella negato.', 'Il download non ha superato il controllo di integrità.', 'Spazio su disco insufficiente.', 'Stato', 'Segreti scoperti'],
  'ru-RU': ['Игра завершилась успешно.', 'Игра завершилась с ошибкой.', 'Не удалось запустить Minecraft.', 'Причина указана в журнале.', 'Не удалось скачать файл.', 'Сетевое подключение недоступно.', 'Для этой версии нужна более новая Java.', 'Сервер отклонил запрос.', 'Отсутствует необходимая библиотека.', 'Сессия истекла. Войдите снова.', 'Нет прав доступа к папке.', 'Файл не прошёл проверку целостности.', 'Недостаточно места на диске.', 'Состояние', 'Открыто секретов'],
  'ja-JP': ['ゲームは正常に終了しました。', 'ゲームはエラーで終了しました。', 'Minecraftを起動できませんでした。', '原因をログで確認してください。', 'ファイルをダウンロードできませんでした。', 'ネットワーク接続を利用できません。', 'このバージョンには新しいJavaが必要です。', 'サーバーがリクエストを拒否しました。', '必要なライブラリがありません。', 'セッションが切れました。再ログインしてください。', 'フォルダーへのアクセス権限がありません。', 'ダウンロードの整合性確認に失敗しました。', 'ディスクの空き容量が不足しています。', '状態', '発見した秘密'],
  'ko-KR': ['게임이 정상적으로 종료되었습니다.', '게임이 오류로 종료되었습니다.', 'Minecraft를 시작할 수 없습니다.', '원인은 로그를 확인하세요.', '파일을 다운로드할 수 없습니다.', '네트워크 연결을 사용할 수 없습니다.', '이 버전은 더 최신 Java가 필요합니다.', '서버가 요청을 거부했습니다.', '필수 라이브러리가 없습니다.', '세션이 만료되었습니다. 다시 로그인하세요.', '폴더에 접근할 권한이 없습니다.', '다운로드 무결성 검사에 실패했습니다.', '디스크 공간이 부족합니다.', '상태', '발견한 비밀'],
  'zh-CN': ['游戏已正常关闭。', '游戏因错误而关闭。', '无法启动Minecraft。', '请查看日志了解原因。', '无法下载文件。', '网络连接不可用。', '此版本需要更新的Java。', '服务器拒绝了请求。', '缺少必要的库。', '会话已过期，请重新登录。', '没有访问此文件夹的权限。', '下载文件未通过完整性校验。', '磁盘空间不足。', '状态', '已发现秘密'],
  'ca-ES': ['El joc s’ha tancat correctament.', 'El joc s’ha tancat amb un error.', 'No s’ha pogut iniciar Minecraft.', 'Consulta el registre per saber-ne la causa.', 'No s’ha pogut baixar el fitxer.', 'La connexió de xarxa no està disponible.', 'Aquesta versió necessita un Java més recent.', 'El servidor ha rebutjat la petició.', 'Falta una biblioteca necessària.', 'La sessió ha caducat. Torna a iniciar-la.', 'No tens permís per accedir a la carpeta.', 'La baixada no ha superat la comprovació d’integritat.', 'No hi ha prou espai al disc.', 'Estat', 'Secrets descoberts'],
  'eo-UY': ['La ludo sukcese fermiĝis.', 'La ludo fermiĝis pro eraro.', 'Ne eblis lanĉi Minecraft.', 'Legu la protokolon por trovi la kaŭzon.', 'Ne eblis elŝuti la dosieron.', 'Reta konekto ne disponeblas.', 'Ĉi tiu versio bezonas pli novan Java.', 'La servilo rifuzis la peton.', 'Mankas bezonata biblioteko.', 'La seanco eksvalidiĝis. Ensalutu denove.', 'Vi ne rajtas aliri la dosierujon.', 'La elŝuto malsukcesis la integreckontrolon.', 'Nesufiĉa diskospaco.', 'Stato', 'Malkovritaj sekretoj'],
  'en-PT': ['The voyage ended safely.', 'The ship ran into trouble.', 'Could not set sail in Minecraft.', 'Read the ship’s log fer the cause.', 'Could not haul the file aboard.', 'The network be lost at sea.', 'This voyage needs a newer Java engine.', 'The harbour refused the request.', 'A needed library be missing.', 'Yer papers expired. Board again.', 'Ye cannot enter this hold.', 'The cargo failed inspection.', 'The ship’s hold be full.', 'Ship status', 'Secrets charted'],
  'lol-US': ['Game closed all gud.', 'Game had an oopsie.', 'No can start Minecraft.', 'Check logz fer why.', 'No can haz file.', 'No haz interwebz.', 'Dis vershun needz newer Java.', 'Server sez nope.', 'Missing library nomz.', 'Login expired. Login agin plz.', 'No can enter dis folder.', 'File check sez nope.', 'Disk iz full of nomz.', 'Wut happenin', 'Secretz found']
};
for (const [code, values] of Object.entries(diagnosticRows)) {
  if (values.length !== diagnosticKeys.length) throw new Error(`Diagnostic locale ${code}: incomplete row`);
  diagnosticKeys.forEach((key, index) => { partialCatalogs[code][key] = values[index]; });
}
Object.assign(partialCatalogs['pt-BR'], ptBR);
Object.assign(partialCatalogs['fr-FR'], frFR);
Object.assign(partialCatalogs['de-DE'], deDE);
Object.assign(partialCatalogs['it-IT'], itIT);
Object.assign(partialCatalogs['ca-ES'], caES);
Object.assign(partialCatalogs['eo-UY'], eoUY);
Object.assign(partialCatalogs['en-PT'], enPT);
Object.assign(partialCatalogs['lol-US'], lolUS);
Object.assign(partialCatalogs['ru-RU'], ruRU);
Object.assign(partialCatalogs['ja-JP'], jaJP);
Object.assign(partialCatalogs['ko-KR'], koKR);
Object.assign(partialCatalogs['zh-CN'], zhCN);

Object.assign(english, supplementEN);
Object.assign(partialCatalogs['pt-BR'], supplementPt);
Object.assign(partialCatalogs['fr-FR'], supplementFr);
Object.assign(partialCatalogs['de-DE'], supplementDe);
Object.assign(partialCatalogs['it-IT'], supplementIt);
Object.assign(partialCatalogs['ca-ES'], supplementCa);
Object.assign(partialCatalogs['eo-UY'], supplementEo);
Object.assign(partialCatalogs['ru-RU'], supplementRu);
Object.assign(partialCatalogs['ja-JP'], supplementJa);
Object.assign(partialCatalogs['ko-KR'], supplementKo);
Object.assign(partialCatalogs['zh-CN'], supplementZh);



const pirateWords: Record<string,string> = {your:'yer',you:'ye',my:'me',friends:'crew',friend:'matey',folder:'hold',folders:'holds',library:'chart room',libraries:'chart rooms',download:'haul aboard',downloads:'hauls',world:'realm',worlds:'realms',player:'sailor',players:'sailors',search:'scout',back:'astern',welcome:'ahoy',saved:'stowed',save:'stow',copy:'duplicate',view:'spy',show:'reveal',select:'choose',choose:'pick',current:'present',new:'fresh',loading:'hoisting'};
const catWords: Record<string,string> = {your:'ur',you:'u',my:'mah',the:'teh',this:'dis',these:'dese',that:'dat',is:'iz',are:'r',and:'an',with:'wif',have:'haz',has:'haz',no:'noes',not:'nao',please:'plz',files:'filez',file:'file',worlds:'worldz',mods:'modz',settings:'settingz',players:'playerz',friends:'frenz',saved:'savedz',loading:'loadin',preparing:'preparin',select:'pick',copy:'copi'};
function dialect(value:string, words:Record<string,string>):string {
  return value.replace(/\b[A-Za-z]+\b/g, word => {
    const replacement=words[word.toLowerCase()];
    if(!replacement)return word;
    return word===word.toUpperCase()?replacement.toUpperCase():/^[A-Z]/.test(word)?replacement[0].toUpperCase()+replacement.slice(1):replacement;
  });
}
for(const [key,value] of Object.entries(supplementEN)) {
  partialCatalogs['en-PT'][key]=dialect(value,pirateWords);
  partialCatalogs['lol-US'][key]=dialect(value,catWords);
}

Object.assign(english,installationCatalogs["en-US"]);
for (const [code,catalog] of Object.entries(installationCatalogs)) { if(partialCatalogs[code]) Object.assign(partialCatalogs[code],catalog); }

Object.assign(partialCatalogs['pt-BR'],releaseUi0);
Object.assign(partialCatalogs['fr-FR'],releaseUi1);
Object.assign(partialCatalogs['de-DE'],releaseUi2);
Object.assign(partialCatalogs['it-IT'],releaseUi3);
Object.assign(partialCatalogs['ru-RU'],releaseUi4);
Object.assign(partialCatalogs['ja-JP'],releaseUi5);
Object.assign(partialCatalogs['ko-KR'],releaseUi6);
Object.assign(partialCatalogs['zh-CN'],releaseUi7);
Object.assign(partialCatalogs['ca-ES'],releaseUi8);
Object.assign(partialCatalogs['eo-UY'],releaseUi9);
Object.assign(english,licenseGate['en-US']);
for(const [code,values] of Object.entries(licenseGate)){if(partialCatalogs[code])Object.assign(partialCatalogs[code],values);}
Object.assign(english,expanded['en-US']);
for(const [code,values] of Object.entries(expanded)){if(partialCatalogs[code])Object.assign(partialCatalogs[code],values);}
for(const [key,value] of Object.entries(english)){
 if(!partialCatalogs['en-PT'][key])partialCatalogs['en-PT'][key]=dialect(value,pirateWords);
 if(!partialCatalogs['lol-US'][key])partialCatalogs['lol-US'][key]=dialect(value,catWords);
}
