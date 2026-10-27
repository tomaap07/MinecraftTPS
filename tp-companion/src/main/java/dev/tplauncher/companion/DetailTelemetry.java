package dev.tplauncher.companion;
import com.google.gson.*;
import net.minecraft.client.MinecraftClient;
import net.minecraft.item.ItemStack;
import net.minecraft.registry.Registries;
import net.minecraft.world.LightType;

public final class DetailTelemetry{
 private static net.minecraft.client.world.ClientWorld lastWorld;

 public static void statisticsReady(){}
 private static void add(JsonObject m,String key,double value){if(Double.isFinite(value))m.addProperty(key,value);}
 public static void write(MinecraftClient c,JsonObject data,boolean restricted){
  if(c.player==null||c.world==null)return;
  if(lastWorld!=c.world){lastWorld=c.world;CombatTelemetry.reset();}
  var p=c.player;var w=c.world;JsonObject m=new JsonObject();
  add(m,"health",p.getHealth());add(m,"food",p.getHungerManager().getFoodLevel());add(m,"saturation",p.getHungerManager().getSaturationLevel());add(m,"oxygen",p.getAir());add(m,"xp_level",p.experienceLevel);add(m,"attack_cooldown",p.getAttackCooldownProgress(0));add(m,"combo",CombatTelemetry.combo());
  if(c.crosshairTarget instanceof net.minecraft.util.hit.EntityHitResult hit)add(m,"target_distance",p.getEyePos().distanceTo(hit.getPos()));
  int free=0;java.util.Map<String,Integer> counts=new java.util.HashMap<>();
  for(int i=0;i<36;i++){ItemStack stack=p.getInventory().getStack(i);if(stack.isEmpty()){free++;continue;}String id=Registries.ITEM.getId(stack.getItem()).getPath();counts.merge(id,stack.getCount(),Integer::sum);}
  add(m,"inventory_free",free);
  for(String id:new String[]{"arrow","ender_pearl","golden_apple","totem_of_undying","firework_rocket"})add(m,"count_"+id,counts.getOrDefault(id,0));
  if(!restricted){add(m,"block_light",w.getLightLevel(LightType.BLOCK,p.getBlockPos()));add(m,"sky_light",w.getLightLevel(LightType.SKY,p.getBlockPos()));}
  data.add("world_metrics",m);
 }
}
