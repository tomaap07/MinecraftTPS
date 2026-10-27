package dev.tplauncher.companion;
import com.google.gson.*;
import net.minecraft.client.MinecraftClient;
import net.minecraft.sound.SoundCategory;
import java.nio.file.*;import java.nio.charset.StandardCharsets;import java.io.*;import java.util.*;

public final class ControlBridge{
 private static String previous="";
 public static void poll(MinecraftClient c){
  String location=System.getProperty("tp.launcher.profile","");if(location.isBlank())return;
  try{Path folder=Path.of(location).getParent(),path=folder.resolve("command-"+ProcessHandle.current().pid()+".properties");if(!Files.isRegularFile(path)||Files.size(path)>16384)return;
   Properties p=new Properties();try(Reader r=Files.newBufferedReader(path,StandardCharsets.UTF_8)){p.load(r);}String id=p.getProperty("id","");if(!id.matches("[0-9a-f-]{36}")||id.equals(previous)||!p.getProperty("adapter","").equals("companion"))return;previous=id;
   try{String action=p.getProperty("action","");switch(action){
    case "connect"->{String address=p.getProperty("address","");if(!net.minecraft.client.network.ServerAddress.isValid(address))throw new IllegalArgumentException("Dirección inválida");if(c.world!=null)c.world.disconnect();c.disconnect();var parent=new net.minecraft.client.gui.screen.TitleScreen();net.minecraft.client.gui.screen.multiplayer.ConnectScreen.connect(parent,c,net.minecraft.client.network.ServerAddress.parse(address),new net.minecraft.client.network.ServerInfo("Servidor TP",address,net.minecraft.client.network.ServerInfo.ServerType.OTHER),false,null);ack(folder,id,true,"Conexión iniciada. Minecraft comprobará el acceso al servidor.");return;}
    case "option"->option(c,p.getProperty("name",""),Double.parseDouble(p.getProperty("value","")));
    case "packs"->{var manager=c.getResourcePackManager();manager.scanPacks();List<String> selected=new ArrayList<>();for(int i=0;i<128;i++){String name=p.getProperty("pack."+i);if(name==null)break;if(manager.getProfile(name)==null)throw new IllegalArgumentException("Un pack ya no está disponible: "+name);selected.add(name);}manager.setEnabledProfiles(selected);c.options.refreshResourcePacks(manager);c.options.write();c.reloadResources().whenComplete((unused,error)->ack(folder,id,error==null,error==null?"Paquetes recargados en el juego.":"Minecraft no pudo recargar los paquetes."));return;}
    case "reload_resources"->{c.reloadResources().whenComplete((unused,error)->ack(folder,id,error==null,error==null?"Recursos recargados.":"Falló la recarga de recursos."));return;}
    case "shader"->shader(p.getProperty("name",""));
    case "memory_gc"->{System.gc();ack(folder,id,true,"Solicitud de recolección enviada. La JVM puede aplazarla o ignorarla.");return;}
    default->throw new IllegalArgumentException("Control no compatible.");
   }ack(folder,id,true,"Cambio aplicado en Minecraft.");}catch(Exception e){ack(folder,id,false,e.getMessage()==null?"Este control no está disponible en esta edición.":e.getMessage());}
  }catch(Exception ignored){}
 }
 private static double clamp(double v,double min,double max){if(!Double.isFinite(v))throw new IllegalArgumentException("Valor inválido");return Math.clamp(v,min,max);}
 private static void option(MinecraftClient c,String name,double value){switch(name){
  case "fov"->c.options.getFov().setValue((int)clamp(value,30,110));case "chunks"->c.options.getViewDistance().setValue((int)clamp(value,2,32));case "gamma"->c.options.getGamma().setValue(clamp(value,0,1));case "gui"->{c.options.getGuiScale().setValue((int)clamp(value,0,8));c.onResolutionChanged();}
  case "volume"->c.options.getSoundVolumeOption(SoundCategory.MASTER).setValue(clamp(value,0,1));case "music"->c.options.getSoundVolumeOption(SoundCategory.MUSIC).setValue(clamp(value,0,1));case "fps_limit"->c.options.getMaxFps().setValue((int)clamp(value,10,260));default->throw new IllegalArgumentException("Opción no permitida");
 }c.options.write();}
 private static void shader(String name)throws Exception{
  if(net.fabricmc.loader.api.FabricLoader.getInstance().isModLoaded("iris")){
   Class<?> iris=Class.forName("net.irisshaders.iris.Iris");Object config=iris.getMethod("getIrisConfig").invoke(null);config.getClass().getMethod("setShaderPackName",String.class).invoke(config,name);config.getClass().getMethod("setShadersEnabled",boolean.class).invoke(config,!name.isEmpty());config.getClass().getMethod("save").invoke(config);iris.getMethod("reload").invoke(null);
  }else throw new IllegalArgumentException("No se detectó un adaptador de shaders compatible. Instala Iris para esta instancia.");
 }
 private static void ack(Path folder,String id,boolean ok,String message){try{JsonObject out=new JsonObject();out.addProperty("id",id);out.addProperty("ok",ok);out.addProperty("message",message.substring(0,Math.min(512,message.length())));Path path=folder.resolve("ack-"+ProcessHandle.current().pid()+".json"),tmp=path.resolveSibling(path.getFileName()+".tmp");Files.writeString(tmp,out.toString(),StandardCharsets.UTF_8);Files.move(tmp,path,StandardCopyOption.REPLACE_EXISTING,StandardCopyOption.ATOMIC_MOVE);}catch(Exception ignored){}}
}
