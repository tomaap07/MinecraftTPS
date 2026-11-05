package dev.tplauncher.client;

public enum WidgetKind {
    FPS("FPS", "Fotogramas reales por segundo; no son TPS.", "Información"),
    FRAMETIME("Tiempo entre cuadros", "Media y percentil 95 de los últimos cuadros del HUD.", "Rendimiento"),
    PING("Latencia", "Latencia comunicada por el servidor; no disponible en individual.", "PvP"),
    COORDINATES("Coordenadas", "Tu posición; respeta la información de depuración reducida.", "Información"),
    DIRECTION("Brújula", "Orientación horizontal y ángulo de tu personaje.", "Información"),
    BIOME("Bioma", "Bioma ya cargado en tu posición.", "Información"),
    DIMENSION("Dimensión", "Dimensión actual del cliente.", "Información"),
    WORLD_TIME("Hora del mundo", "Días y hora del mundo, sin confundirlos con el reloj local.", "Información"),
    CLOCK("Reloj", "Hora local de este equipo.", "Información"),
    CPS("Clics por segundo", "Pulsaciones reales del ratón durante el último segundo.", "PvP"),
    KEYS("Teclas", "Estado de movimiento, salto, sprint y agacharse; no automatiza entradas.", "PvP"),
    ARMOR("Armadura", "Iconos del equipo y porcentaje real de durabilidad restante.", "HUD"),
    HANDS("Herramientas", "Objetos equipados y su durabilidad.", "HUD"),
    EFFECTS("Efectos", "Efectos de estado activos y tiempo restante.", "HUD"),
    VITALS("Salud y hambre", "Vida y hambre que recibe tu cliente.", "HUD"),
    SPEED("Velocidad", "Distancia horizontal real recorrida por segundo.", "Información"),
    SESSION("Sesión", "Tiempo transcurrido desde que se abrió este cliente.", "Información"),
    MEMORY("Memoria Java", "Heap local ocupado y máximo; no mide memoria de otras aplicaciones.", "Rendimiento"),
    TARGET("Objetivo visible", "Nombre del objetivo bajo la mira y distancia visible. Sin ayuda de puntería.", "PvP"),
    LIGHT("Luz local", "Luz del bloque y del cielo en tu posición cargada.", "Información");

    public final String label, description, category;
    WidgetKind(String label, String description, String category) { this.label=label; this.description=description; this.category=category; }
    public String id() { return name().toLowerCase(java.util.Locale.ROOT); }
}
