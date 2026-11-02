package dev.tplauncher.client;

import com.google.gson.*;
import net.fabricmc.fabric.api.client.rendering.v1.LivingEntityFeatureRendererRegistrationCallback;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.AbstractClientPlayerEntity;
import net.minecraft.client.render.entity.PlayerEntityRenderer;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.client.util.SkinTextures;
import net.minecraft.util.Identifier;
import net.minecraft.util.Util;
import java.io.IOException;
import java.nio.*;
import java.nio.file.*;
import java.util.*;

public final class Cosmetics {
    private static final Gson GSON=new GsonBuilder().setPrettyPrinting().create();
    private static final Identifier BUILTIN=Identifier.of("tp_client","textures/nebulosa.png");
    private static State state=new State();private static Identifier cape;private static long lastStamp=-1;private static String loaded="";
    public static final class State {public int schemaVersion=1;public String equippedCape="";public boolean wings=false,crown=false;}
    private static Path directory(){return TpClient.configDirectory.resolve("tp-client-cosmetics");}
    private static Path stateFile(){return TpClient.configDirectory.resolve("tp-client-cosmetics.json");}
    public static void initialize(){
        LivingEntityFeatureRendererRegistrationCallback.EVENT.register((type,renderer,helper,context)->{if(renderer instanceof PlayerEntityRenderer player)helper.register(new CosmeticFeature(player));});
        reload(MinecraftClient.getInstance());
    }
    public static State parseState(String text) throws IOException {
        try{State state=GSON.fromJson(text,State.class);if(state==null||state.schemaVersion!=1||state.equippedCape==null||(!state.equippedCape.isEmpty()&&!state.equippedCape.equals("builtin:nebulosa")&&!state.equippedCape.matches("[A-Za-z0-9_-]{1,64}\\.png")))throw new IOException("Cosméticos no válidos");return state;}catch(JsonParseException error){throw new IOException("Cosméticos JSON dañados",error);}
    }
    public static boolean validCape(byte[] data){
        if(data.length<33||data.length>262144)return false;
        byte[] signature={(byte)137,80,78,71,13,10,26,10};for(int i=0;i<8;i++)if(data[i]!=signature[i])return false;
        ByteBuffer b=ByteBuffer.wrap(data).order(ByteOrder.BIG_ENDIAN);
        return b.getInt(8)==13&&b.getInt(12)==0x49484452&&b.getInt(16)==64&&b.getInt(20)==32;
    }
    public static void reload(MinecraftClient client){
        try{
            Path source=stateFile();if(!Files.isRegularFile(source)||Files.isSymbolicLink(source)||Files.size(source)>4096)return;
            State next=parseState(Files.readString(source));
            if(Files.isSymbolicLink(directory()))throw new IOException("Carpeta de capas no válida");
            Path image=directory().resolve(next.equippedCape);
            long stamp=next.equippedCape.startsWith("builtin:")?0:Files.isRegularFile(image)&&!Files.isSymbolicLink(image)?Files.getLastModifiedTime(image).toMillis():-1;
            state=next;
            if(loaded.equals(next.equippedCape)&&lastStamp==stamp)return;
            close(client);loaded=next.equippedCape;lastStamp=stamp;
            if(loaded.isEmpty())return;
            if(loaded.equals("builtin:nebulosa")){cape=BUILTIN;return;}
            if(stamp<0||Files.size(image)>262144)throw new IOException("Capa PNG no válida");
            byte[] bytes=Files.readAllBytes(image);if(!validCape(bytes))throw new IOException("La capa debe ser PNG 64×32 de hasta 256 KiB");
            NativeImage decoded=NativeImage.read(bytes);if(decoded.getWidth()!=64||decoded.getHeight()!=32){decoded.close();throw new IOException("Dimensiones de capa no válidas");}
            cape=client.getTextureManager().registerDynamicTexture("tp-local-cape",new NativeImageBackedTexture(decoded));
        }catch(Exception error){TpClient.LOG.warn("Local cosmetic file ignored: {}",error.getClass().getSimpleName());}
    }
    private static void save(){try{ConfigStore.writeText(stateFile(),GSON.toJson(state));reload(MinecraftClient.getInstance());}catch(Exception e){TpClient.notices.add("Cosméticos","No se guardó la selección local");}}
    public static boolean wings(){return state.wings;}
    public static boolean crown(){return state.crown;}
    public static void toggleWings(){state.wings=!state.wings;save();}
    public static void toggleCrown(){state.crown=!state.crown;save();}
    public static String capeLabel(){return state.equippedCape.isEmpty()?"oficial / ninguna":state.equippedCape.equals("builtin:nebulosa")?"Nebulosa Infinita":state.equippedCape;}
    public static void cycleCape(){
        List<String> options=new ArrayList<>(List.of("","builtin:nebulosa"));
        try{Files.createDirectories(directory());if(!Files.isSymbolicLink(directory()))try(var files=Files.list(directory())){files.filter(p->!Files.isSymbolicLink(p)&&Files.isRegularFile(p)&&p.getFileName().toString().matches("[A-Za-z0-9_-]{1,64}\\.png")).limit(128).map(p->p.getFileName().toString()).sorted().forEach(options::add);}}catch(IOException ignored){}
        state.equippedCape=options.get((options.indexOf(state.equippedCape)+1)%options.size());save();
    }
    public static void openFolder(){try{Files.createDirectories(directory());Util.getOperatingSystem().open(directory().toFile());TpClient.notices.add("Capas propias","PNG 64×32, máximo 256 KiB. Se ven solo en tu cliente.");}catch(IOException ignored){TpClient.notices.add("Capas propias","No se pudo abrir la carpeta local");}}
    public static boolean local(AbstractClientPlayerEntity player){MinecraftClient c=MinecraftClient.getInstance();return TpClient.config!=null&&TpClient.config.cosmeticsEnabled&&c.player!=null&&c.player.getUuid().equals(player.getUuid())&&!player.isInvisible();}
    public static SkinTextures decorate(AbstractClientPlayerEntity player,SkinTextures original){return local(player)&&cape!=null?new SkinTextures(original.texture(),original.textureUrl(),cape,original.elytraTexture(),original.model(),original.secure()):original;}
    public static SkinTextures previewTextures(SkinTextures original){return TpClient.config!=null&&TpClient.config.cosmeticsEnabled&&cape!=null?new SkinTextures(original.texture(),original.textureUrl(),cape,original.elytraTexture(),original.model(),original.secure()):original;}
    public static void close(MinecraftClient client){if(cape!=null&&!cape.equals(BUILTIN))client.getTextureManager().destroyTexture(cape);cape=null;}
    private Cosmetics(){}
}
