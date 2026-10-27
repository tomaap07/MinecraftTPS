package dev.tplauncher.companion.mixin;
import dev.tplauncher.companion.Profile;
import com.mojang.blaze3d.systems.RenderSystem;
import net.minecraft.client.render.BackgroundRenderer;
import net.minecraft.client.render.Camera;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(BackgroundRenderer.class)
public abstract class FogMixin{
 @Inject(method="applyFog",at=@At("TAIL"))
 private static void tp$fog(Camera camera,BackgroundRenderer.FogType type,float distance,boolean thick,float delta,CallbackInfo ci){
  float strength=Profile.fogStrength(camera);if(strength<=0)return;
  RenderSystem.setShaderFogStart(RenderSystem.getShaderFogStart()+strength*1000000);
  RenderSystem.setShaderFogEnd(RenderSystem.getShaderFogEnd()+strength*10000000);
 }
}
