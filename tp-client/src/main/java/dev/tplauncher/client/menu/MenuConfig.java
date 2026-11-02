package dev.tplauncher.client.menu;

import com.google.gson.*;
import dev.tplauncher.client.ConfigStore;
import java.io.IOException;
import java.nio.file.*;

public final class MenuConfig {
    public int schemaVersion=1;
    public boolean enabled=true,particles=true,parallax=true,playerPreview=true,playerMotion=true,reducedMotion=false,lowPower=false,musicEnabled=true;
    public double musicVolume=0.26;
    public String layout="balanced";
    public static MenuConfig parse(String text) throws IOException {
        try {
            JsonObject json=JsonParser.parseString(text).getAsJsonObject();
            int version=json.has("schemaVersion")?json.get("schemaVersion").getAsInt():0;
            if(version<0||version>1)throw new IOException("Versión de menú incompatible");
            MenuConfig config=new Gson().fromJson(json,MenuConfig.class);
            config.schemaVersion=1;
            if(!Double.isFinite(config.musicVolume))config.musicVolume=0.26;
            config.musicVolume=Math.max(0,Math.min(1,config.musicVolume));
            if(config.layout==null||!java.util.List.of("balanced","center").contains(config.layout))config.layout="balanced";
            return config;
        }catch(JsonParseException|IllegalStateException|NumberFormatException error){throw new IOException("Configuración del menú dañada",error);}
    }
    public static MenuConfig load(Path path){
        try {
            if(!Files.exists(path))return new MenuConfig();
            if(Files.isSymbolicLink(path)||!Files.isRegularFile(path)||Files.size(path)>16384)throw new IOException("Archivo de menú no válido");
            return parse(Files.readString(path));
        }catch(Exception error){
            try{if(Files.isRegularFile(path)&&!Files.isSymbolicLink(path))Files.copy(path,path.resolveSibling(path.getFileName()+".invalid-"+System.currentTimeMillis()));}catch(IOException ignored){}
            return new MenuConfig();
        }
    }
    public void save(Path path)throws IOException{ConfigStore.writeText(path,new GsonBuilder().setPrettyPrinting().create().toJson(this));}
    public boolean motion(){return !reducedMotion&&!lowPower;}
}
