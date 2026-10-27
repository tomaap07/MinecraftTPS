package dev.tplauncher.companion.mixin;
import dev.tplauncher.companion.TpCompanion;
import net.minecraft.client.render.Camera;
import net.minecraft.client.render.GameRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
@Mixin(GameRenderer.class)
public abstract class GameRendererMixin {
    @Inject(method="getFov",at=@At("RETURN"),cancellable=true)
    private void tp$zoom(Camera camera,float tickDelta,boolean changingFov,CallbackInfoReturnable<Double> callback){
        if(TpCompanion.zooming())callback.setReturnValue(Math.max(5,callback.getReturnValueD()/4.0));
    }
}
