package dev.tplauncher.companion.mixin;
import dev.tplauncher.companion.Profile;
import dev.tplauncher.companion.TelemetryBridge;
import net.minecraft.client.gui.hud.InGameHud;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.render.RenderTickCounter;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(InGameHud.class)
public abstract class InGameHudMixin{
 @Inject(method="renderCrosshair",at=@At("HEAD"),cancellable=true)
 private void tp$crosshair(DrawContext context,RenderTickCounter ticks,CallbackInfo ci){
  if(TelemetryBridge.nativeVisible()&&Profile.crosshairEnabled())ci.cancel();
 }
}
