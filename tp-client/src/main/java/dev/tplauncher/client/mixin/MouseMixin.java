package dev.tplauncher.client.mixin;
import dev.tplauncher.client.TpClient;
import net.minecraft.client.Mouse;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(Mouse.class)
public abstract class MouseMixin {
    @Inject(method="onMouseButton",at=@At("HEAD"))
    private void tp$click(long window,int button,int action,int mods,CallbackInfo callback){TpClient.click(button,action);}
}
