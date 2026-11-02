package dev.tplauncher.client;

import java.util.*;

public final class ClientConfig {
    public int schemaVersion=1;
    public String mode="minimal";
    public boolean hudEnabled=true, cosmeticsEnabled=false, streamer=false, zoomEnabled=true, notifications=true, metricsEnabled=false;
    public boolean hideCoordinates=true, hideNames=true, hideNotifications=true;
    public int grid=4, accent=0xB678F5;
    public String profile="Supervivencia";
    public LinkedHashMap<String,Widget> widgets=new LinkedHashMap<>();
    public static final List<String> ANCHORS=List.of("top_left","top_right","bottom_left","bottom_right","top_center","bottom_center");

    public static final class Widget {
        public boolean enabled, locked=false, background=true, singleplayer=true, multiplayer=true, positioned=false;
        public double scale=1.0, opacity=0.9, x=0, y=0;
        public String anchor="top_left", style="panel";
        public int order=0;
    }
    public ClientConfig() { preset("Supervivencia"); }
    public void preset(String name) {
        profile=name; widgets.clear(); streamer=name.equals("Streaming");
        Set<WidgetKind> active=switch(name) {
            case "PvP" -> EnumSet.of(WidgetKind.FPS,WidgetKind.PING,WidgetKind.CPS,WidgetKind.KEYS,WidgetKind.ARMOR,WidgetKind.HANDS,WidgetKind.EFFECTS);
            case "Minimalista", "Streaming" -> EnumSet.of(WidgetKind.FPS,WidgetKind.PING);
            case "Exploración" -> EnumSet.of(WidgetKind.FPS,WidgetKind.COORDINATES,WidgetKind.DIRECTION,WidgetKind.BIOME,WidgetKind.DIMENSION,WidgetKind.WORLD_TIME,WidgetKind.SPEED,WidgetKind.ARMOR);
            case "Técnico" -> EnumSet.of(WidgetKind.FPS,WidgetKind.COORDINATES,WidgetKind.LIGHT,WidgetKind.DIRECTION,WidgetKind.WORLD_TIME,WidgetKind.MEMORY,WidgetKind.FRAMETIME);
            default -> EnumSet.of(WidgetKind.FPS,WidgetKind.COORDINATES,WidgetKind.DIRECTION,WidgetKind.ARMOR,WidgetKind.HANDS,WidgetKind.EFFECTS);
        };
        for(WidgetKind kind:WidgetKind.values()) {
            Widget w=new Widget(); w.enabled=active.contains(kind); w.order=kind.ordinal();
            boolean equipment=kind==WidgetKind.ARMOR||kind==WidgetKind.HANDS||kind==WidgetKind.EFFECTS;
            w.anchor=equipment?"top_right":"top_left";
            widgets.put(kind.id(),w);
        }
    }
    public void normalize() {
        schemaVersion=1;
        if(!List.of("minimal","full").contains(mode))mode="minimal";
        grid=(int)HudMath.finite(grid,4,1,32);accent&=0xFFFFFF;
        if(profile==null||profile.length()>48)profile="Personalizado";
        if(widgets==null)widgets=new LinkedHashMap<>();
        widgets.keySet().removeIf(id->Arrays.stream(WidgetKind.values()).noneMatch(k->k.id().equals(id)));
        for(WidgetKind kind:WidgetKind.values()) {
            Widget w=widgets.computeIfAbsent(kind.id(),id->new Widget());
            if(w==null){w=new Widget();widgets.put(kind.id(),w);}
            w.scale=HudMath.finite(w.scale,1,0.5,3);w.opacity=HudMath.finite(w.opacity,0.9,0.1,1);
            w.x=HudMath.finite(w.x,0,-10000,10000);w.y=HudMath.finite(w.y,0,-10000,10000);
            if(!ANCHORS.contains(w.anchor))w.anchor="top_left";
            if(!List.of("panel","plain","outline").contains(w.style))w.style="panel";
            w.order=(int)HudMath.finite(w.order,kind.ordinal(),0,100);
        }
    }
}
