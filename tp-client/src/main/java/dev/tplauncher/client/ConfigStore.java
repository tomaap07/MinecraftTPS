package dev.tplauncher.client;

import com.google.gson.*;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;

public final class ConfigStore {
    private static final Gson GSON=new GsonBuilder().setPrettyPrinting().create();
    public static final int MAX_BYTES=262144;
    public static ClientConfig read(Path path) throws IOException {
        if(!Files.exists(path))return new ClientConfig();
        if(Files.isSymbolicLink(path)||!Files.isRegularFile(path)||Files.size(path)>MAX_BYTES)throw new IOException("Archivo de perfil no válido");
        try {
            JsonObject value=JsonParser.parseString(Files.readString(path,StandardCharsets.UTF_8)).getAsJsonObject();
            int version=value.has("schemaVersion")?value.get("schemaVersion").getAsInt():0;
            if(version<0||version>1)throw new IOException("Versión de perfil no compatible");
            ClientConfig config=GSON.fromJson(value,ClientConfig.class);config.normalize();return config;
        } catch(JsonParseException|IllegalStateException|NumberFormatException error) {throw new IOException("Perfil JSON dañado",error);}
    }
    public static ClientConfig loadSafely(Path path) {
        try{return read(path);}catch(Exception error) {
            try{if(Files.isRegularFile(path)&&!Files.isSymbolicLink(path))Files.copy(path,path.resolveSibling(path.getFileName()+".invalid-"+System.currentTimeMillis()),StandardCopyOption.COPY_ATTRIBUTES);}catch(IOException ignored){}
            return new ClientConfig();
        }
    }
    public static void write(Path path,ClientConfig config) throws IOException {
        config.normalize();writeText(path,GSON.toJson(config));
    }
    public static void writeText(Path path,String text) throws IOException {
        byte[] data=text.getBytes(StandardCharsets.UTF_8);if(data.length>MAX_BYTES)throw new IOException("Perfil demasiado grande");
        Path parent=path.toAbsolutePath().getParent();Files.createDirectories(parent);
        if(Files.isSymbolicLink(path)||Files.isSymbolicLink(parent))throw new IOException("No se permiten enlaces simbólicos");
        Path pending=Files.createTempFile(parent,"tp-client-",".pending");
        try {Files.write(pending,data);try{Files.move(pending,path,StandardCopyOption.ATOMIC_MOVE,StandardCopyOption.REPLACE_EXISTING);}catch(AtomicMoveNotSupportedException ignored){Files.move(pending,path,StandardCopyOption.REPLACE_EXISTING);}}finally{Files.deleteIfExists(pending);}
    }
    public static Path profilePath(Path root,String name) throws IOException {
        if(name==null||!name.matches("[a-zA-Z0-9_-]{1,48}"))throw new IOException("Nombre: letras, números, guion o guion bajo");
        return root.resolve("tp-client-profiles").resolve(name+".json");
    }
    private ConfigStore() {}
}
