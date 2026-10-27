package dev.tplauncher.companion.mixin;
import dev.tplauncher.companion.Profile;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(Screen.class)
public abstract class ScreenMixin {
    @Inject(method="renderBackground",at=@At("HEAD"),cancellable=true)
    private void tp$transparent(DrawContext context,int mouseX,int mouseY,float delta,CallbackInfo callback){
        if(MinecraftClient.getInstance().world!=null&&Profile.get().enabled("transparent_menu"))callback.cancel();
    }
}
