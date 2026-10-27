package dev.tplauncher.companion.mixin;
import dev.tplauncher.companion.*;
import net.minecraft.client.gui.DrawContext;import net.minecraft.client.gui.screen.*;import net.minecraft.client.gui.widget.ButtonWidget;import net.minecraft.text.Text;
import org.spongepowered.asm.mixin.Mixin;import org.spongepowered.asm.mixin.injection.*;import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(TitleScreen.class)
public abstract class TitleScreenMixin extends Screen{
 protected TitleScreenMixin(Text title){super(title);}
 @Inject(method="init",at=@At("TAIL"))
 private void tp$tools(CallbackInfo ci){
  if(TitleAppearance.themed())for(var child:java.util.List.copyOf(children()))if(child instanceof ButtonWidget button){remove(child);addDrawableChild(new TpButton(button));}
  addDrawableChild(ButtonWidget.builder(Text.literal("TP · Herramientas"),b->client.setScreen(new MenuToolsScreen((Screen)(Object)this))).dimensions(width-130,8,122,20).build());
 }
 @Inject(method="renderBackground",at=@At("HEAD"),cancellable=true)
 private void tp$background(DrawContext context,int x,int y,float delta,CallbackInfo ci){if(TitleAppearance.themed()){TitleAppearance.render(context,width,height);ci.cancel();}}

 private static final class TpButton extends ButtonWidget{
  private final ButtonWidget original;
  TpButton(ButtonWidget original){super(original.getX(),original.getY(),original.getWidth(),original.getHeight(),original.getMessage(),b->original.onPress(),DEFAULT_NARRATION_SUPPLIER);this.original=original;this.active=original.active;}
  @Override protected void renderWidget(DrawContext c,int x,int y,float delta){active=original.active;int edge=active?(isHovered()||isFocused()?0xFFD946EF:0xFF75419E):0xFF3C3545;c.fill(getX(),getY(),getX()+width,getY()+height,edge);c.fill(getX()+1,getY()+1,getX()+width-1,getY()+height-2,active?0xF016161E:0xF00B0B0F);c.drawCenteredTextWithShadow(net.minecraft.client.MinecraftClient.getInstance().textRenderer,getMessage(),getX()+width/2,getY()+(height-8)/2,active?0xF2E6FF:0x81778C);}
 }
}
