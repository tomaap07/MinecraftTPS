package dev.tplauncher.client.mixin;
import dev.tplauncher.client.Cosmetics;
import net.minecraft.client.network.AbstractClientPlayerEntity;
import net.minecraft.client.util.SkinTextures;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
@Mixin(AbstractClientPlayerEntity.class)
public abstract class SkinMixin {
    @Inject(method="getSkinTextures",at=@At("RETURN"),cancellable=true)
    private void tp$localCape(CallbackInfoReturnable<SkinTextures> callback){callback.setReturnValue(Cosmetics.decorate((AbstractClientPlayerEntity)(Object)this,callback.getReturnValue()));}
}
