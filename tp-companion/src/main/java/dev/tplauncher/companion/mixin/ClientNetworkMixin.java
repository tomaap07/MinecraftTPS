package dev.tplauncher.companion.mixin;
import dev.tplauncher.companion.CombatTelemetry;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.network.packet.s2c.play.EntityDamageS2CPacket;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(ClientPlayNetworkHandler.class)
public abstract class ClientNetworkMixin{
 @Inject(method="onStatistics",at=@At("TAIL"))
 private void tp$stats(net.minecraft.network.packet.s2c.play.StatisticsS2CPacket packet,CallbackInfo ci){dev.tplauncher.companion.DetailTelemetry.statisticsReady();}
 @Inject(method="onEntityDamage",at=@At("TAIL"))
 private void tp$damage(EntityDamageS2CPacket packet,CallbackInfo ci){CombatTelemetry.damage(packet);}
}
