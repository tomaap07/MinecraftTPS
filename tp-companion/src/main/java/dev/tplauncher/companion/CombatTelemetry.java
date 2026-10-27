package dev.tplauncher.companion;
import net.minecraft.client.MinecraftClient;
import net.minecraft.network.packet.s2c.play.EntityDamageS2CPacket;

public final class CombatTelemetry {
 private static long sequence,lastHit;private static int combo,target=-1;
 public static int combo(){return System.currentTimeMillis()-lastHit>2000?0:combo;}
 public static void reset(){combo=0;target=-1;lastHit=0;}
 public static long sequence(){return sequence;}
 public static void damage(EntityDamageS2CPacket packet){
  MinecraftClient c=MinecraftClient.getInstance();if(c.world==null||c.player==null)return;
  if(packet.entityId()==c.player.getId()){reset();return;}
  if(packet.createDamageSource(c.world).getAttacker()==c.player&&packet.entityId()!=c.player.getId()){long now=System.currentTimeMillis();combo=target==packet.entityId()&&now-lastHit<2000?combo+1:1;target=packet.entityId();lastHit=now;sequence++;}
 }
}
