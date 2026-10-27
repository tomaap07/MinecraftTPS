package dev.tplauncher.companion;

import com.google.gson.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;


public final class Profile {
    public record Module(String id,boolean enabled,String anchor) {}
    public record Snapshot(double scale,double opacity,boolean background,boolean nativeOverlay,List<Module> modules) {
        public boolean enabled(String id){return modules.stream().anyMatch(module->module.id().equals(id)&&module.enabled());}
    }
    private static final Set<String> ANCHORS=Set.of("top-left","top-center","top-right","bottom-left","bottom-center","bottom-right");
    private static volatile Snapshot current=new Snapshot(1,.9,true,false,List.of(new Module("fps",true,"top-left"),new Module("coordinates",true,"top-left"),new Module("keystrokes",true,"top-right"),new Module("armor",true,"bottom-right")));
    private static volatile JsonObject fog=new JsonObject();
    public static float fogStrength(net.minecraft.client.render.Camera camera){try{
        JsonObject f=fog;if(!f.has("enabled")||!f.get("enabled").getAsBoolean())return 0;
        var client=net.minecraft.client.MinecraftClient.getInstance();if(client.world==null)return 0;
        String env=switch(camera.getSubmersionType()){case WATER->"water";case LAVA->"lava";default->client.world.getRegistryKey()==net.minecraft.world.World.NETHER?"nether":client.world.getRegistryKey()==net.minecraft.world.World.END?"end":"overworld";};
        return f.has(env)&&f.get(env).getAsBoolean()?Math.clamp(f.get("strength").getAsFloat(),0,1):0;
    }catch(Exception ignored){return 0;}}
    private static volatile boolean crosshairEnabled;
    public static boolean crosshairEnabled(){return crosshairEnabled;}
    private static long stamp=Long.MIN_VALUE;
    public static Snapshot get(){return current;}
    public static void reload(){
        String location=System.getProperty("tp.launcher.profile",System.getenv().getOrDefault("TP_LAUNCHER_PROFILE",""));
        if(location.isBlank())return;
        try {
            Path path=Path.of(location);if(!Files.isRegularFile(path)||Files.size(path)>65536)return;
            long modified=Files.getLastModifiedTime(path).toMillis();if(modified==stamp)return;stamp=modified;
            JsonObject root=JsonParser.parseString(Files.readString(path,StandardCharsets.UTF_8)).getAsJsonObject();
            if(root.get("schema_version").getAsInt()!=1)return;
            fog=root.has("fog")?root.getAsJsonObject("fog").deepCopy():new JsonObject();
            crosshairEnabled=root.has("crosshair")&&root.getAsJsonObject("crosshair").has("enabled")&&root.getAsJsonObject("crosshair").get("enabled").getAsBoolean();
            double scale=root.get("scale").getAsDouble(),opacity=root.get("opacity").getAsDouble();
            if(!Double.isFinite(scale)||!Double.isFinite(opacity))return;
            List<Module> modules=new ArrayList<>();Set<String> ids=new HashSet<>();
            for(JsonElement element:root.getAsJsonArray("modules")){
                if(modules.size()>=192)return;JsonObject module=element.getAsJsonObject();
                String id=module.get("id").getAsString(),anchor=module.get("anchor").getAsString();
                if(!ANCHORS.contains(anchor)||id.length()>40||!ids.add(id))return;
                modules.add(new Module(id,module.get("enabled").getAsBoolean(),anchor));
            }
            current=new Snapshot(Math.clamp(scale,.75,1.5),Math.clamp(opacity,.35,1),root.get("background").getAsBoolean(),root.has("native_overlay")&&root.get("native_overlay").getAsBoolean(),List.copyOf(modules));
        } catch(Exception ignored){   }
    }
}
