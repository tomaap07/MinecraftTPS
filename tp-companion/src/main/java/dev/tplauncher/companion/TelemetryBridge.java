package dev.tplauncher.companion;
import com.google.gson.*;
import net.minecraft.client.MinecraftClient;
import net.minecraft.item.ItemStack;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;


public final class TelemetryBridge {
 private static Path output,lease;
 private static boolean nativeVisible;
 public static boolean nativeVisible(){return nativeVisible;}
 public static void init(){
  String source=System.getProperty("tp.launcher.profile",System.getenv().getOrDefault("TP_LAUNCHER_PROFILE",""));if(source.isBlank())return;
  try{Path parent=Path.of(source).toAbsolutePath().getParent();long pid=ProcessHandle.current().pid();output=parent.resolve("telemetry-"+pid+".json");lease=parent.resolve("overlay-"+pid+".active");Runtime.getRuntime().addShutdownHook(new Thread(()->{try{Files.deleteIfExists(output);}catch(Exception ignored){}},"TP telemetry cleanup"));}catch(Exception ignored){}
 }
 public static void write(MinecraftClient client){
  if(output==null)return;
  try{
   long time=System.currentTimeMillis();nativeVisible=Files.isRegularFile(lease)&&time-Files.getLastModifiedTime(lease).toMillis()<2500;
   JsonObject data=new JsonObject();data.addProperty("schema_version",1);data.addProperty("source","TP Companion");
   boolean restricted=client.hasReducedDebugInfo();data.addProperty("coordinates_restricted",restricted);
   JsonArray capabilities=new JsonArray();for(String cap:new String[]{"fps","armor","ping","days","combat","controls","no_fog","optical_zoom"})capabilities.add(cap);if(!restricted)for(String cap:new String[]{"coordinates","direction","minimap"})capabilities.add(cap);data.add("capabilities",capabilities);
   data.addProperty("status",restricted?"El servidor ha reducido la información de depuración.":client.world!=null&&client.player!=null?"Datos internos conectados con Minecraft.":"Entra en un mundo para ver tus coordenadas.");data.addProperty("pid",ProcessHandle.current().pid());data.addProperty("timestamp_ms",time);data.addProperty("game_fps",client.getCurrentFps());data.addProperty("has_world",client.world!=null&&client.player!=null);
   JsonArray armor=new JsonArray();
   if(client.player!=null&&client.world!=null){data.addProperty("x",restricted?0:client.player.getX());data.addProperty("y",restricted?0:client.player.getY());data.addProperty("z",restricted?0:client.player.getZ());data.addProperty("direction",restricted?"":client.player.getHorizontalFacing().asString());var network=client.getNetworkHandler();var entry=network==null?null:network.getPlayerListEntry(client.player.getUuid());if(entry!=null)data.addProperty("ping",entry.getLatency());
    data.addProperty("dimension",client.world.getRegistryKey().getValue().toString());data.addProperty("singleplayer",client.isInSingleplayer());data.addProperty("world_time",client.world.getTimeOfDay());data.addProperty("game_time",client.world.getTime());
    data.addProperty("attack_cooldown",client.player.getAttackCooldownProgress(0));data.addProperty("speed",client.player.getVelocity().horizontalLength()*20);data.addProperty("hit_sequence",CombatTelemetry.sequence());
    if(client.targetedEntity!=null){var target=client.targetedEntity;if(client.crosshairTarget instanceof net.minecraft.util.hit.EntityHitResult hit)data.addProperty("target_distance",client.player.getEyePos().distanceTo(hit.getPos()));data.addProperty("target_kind",target instanceof net.minecraft.entity.mob.HostileEntity||target instanceof net.minecraft.entity.player.PlayerEntity?"hostile":target instanceof net.minecraft.entity.passive.PassiveEntity?"passive":"other");}
    if(!restricted){data.addProperty("biome",client.world.getBiome(client.player.getBlockPos()).getKey().map(key->key.getValue().toString()).orElse(""));capabilities.add("biome");}
    if(client.isInSingleplayer())data.addProperty("ping",0);
    for(var slot:new net.minecraft.entity.EquipmentSlot[]{net.minecraft.entity.EquipmentSlot.HEAD,net.minecraft.entity.EquipmentSlot.CHEST,net.minecraft.entity.EquipmentSlot.LEGS,net.minecraft.entity.EquipmentSlot.FEET})addEquipment(armor,client.player.getEquippedStack(slot),slot.getName());addEquipment(armor,client.player.getMainHandStack(),"mainhand");addEquipment(armor,client.player.getOffHandStack(),"offhand");}

   DetailTelemetry.write(client,data,restricted);
   data.add("armor",armor);if(!restricted&&Profile.get().enabled("minimap")){var map=Minimap.bridge();if(map!=null)data.add("map",map);}Path pending=output.resolveSibling(output.getFileName()+".pending");Files.writeString(pending,data.toString(),StandardCharsets.UTF_8);
   try{Files.move(pending,output,StandardCopyOption.ATOMIC_MOVE,StandardCopyOption.REPLACE_EXISTING);}catch(AtomicMoveNotSupportedException ignored){Files.move(pending,output,StandardCopyOption.REPLACE_EXISTING);}
  }catch(Exception ignored){ }
 }
 private static void addEquipment(JsonArray armor,ItemStack stack,String slot){if(stack.isEmpty())return;JsonObject item=new JsonObject();String name=stack.getName().getString();item.addProperty("name",name.substring(0,name.offsetByCodePoints(0,Math.min(name.codePointCount(0,name.length()),48))));item.addProperty("item_id",net.minecraft.registry.Registries.ITEM.getId(stack.getItem()).toString());item.addProperty("slot",slot);item.addProperty("maximum",stack.getMaxDamage());item.addProperty("remaining",Math.max(0,stack.getMaxDamage()-stack.getDamage()));armor.add(item);}

}
