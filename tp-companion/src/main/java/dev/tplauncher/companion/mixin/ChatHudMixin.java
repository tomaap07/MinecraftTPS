package dev.tplauncher.companion.mixin;
import dev.tplauncher.companion.Profile;
import net.minecraft.client.gui.hud.ChatHud;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import java.time.LocalTime;
import java.time.format.DateTimeFormatter;

@Mixin(ChatHud.class)
public abstract class ChatHudMixin {
    @ModifyVariable(method="addMessage(Lnet/minecraft/text/Text;Lnet/minecraft/network/message/MessageSignatureData;Lnet/minecraft/client/gui/hud/MessageIndicator;)V",at=@At("HEAD"),argsOnly=true,ordinal=0)
    private Text tp$timestamp(Text original){
        if(!Profile.get().enabled("enhanced_chat"))return original;
        return Text.literal("["+LocalTime.now().format(DateTimeFormatter.ofPattern("HH:mm"))+"] ").formatted(Formatting.DARK_GRAY).append(original.copy());
    }
}
