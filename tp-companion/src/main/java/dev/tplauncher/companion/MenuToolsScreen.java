package dev.tplauncher.companion;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

public final class MenuToolsScreen extends Screen{
 private final Screen parent;private String message="";
 public MenuToolsScreen(Screen parent){super(Text.literal("TP LAUNCHER · Tu partida"));this.parent=parent;}
 @Override protected void init(){int x=width/2-120,y=height/2-72;
  addDrawableChild(ButtonWidget.builder(Text.literal("Editar HUD / Overlays · F8"),b->{if(!Profile.get().nativeOverlay()||!TelemetryBridge.nativeVisible()){message="Activa el HUD universal en TP LAUNCHER y mantén el launcher abierto.";return;}client.setScreen(parent);requestEditor();}).dimensions(x,y,240,20).build());
  addDrawableChild(ButtonWidget.builder(Text.literal("Galería F2 · Capturas"),b->client.setScreen(new ScreenshotScreen(this))).dimensions(x,y+26,240,20).build());
  addDrawableChild(ButtonWidget.builder(Text.literal("Tema de inicio: "+(TitleAppearance.themed()?"TP LAUNCHER":"Vanilla")),b->{TitleAppearance.toggle();client.setScreen(new TitleScreen());}).dimensions(x,y+52,240,20).build());
  addDrawableChild(ButtonWidget.builder(Text.literal("Elegir fondo personalizado"),b->client.setScreen(new BackgroundScreen(this))).dimensions(x,y+78,240,20).build());
  addDrawableChild(ButtonWidget.builder(Text.literal("Volver"),b->close()).dimensions(x,y+110,240,20).build());
 }
 private static void requestEditor(){try{String profile=System.getProperty("tp.launcher.profile","");if(!profile.isBlank())java.nio.file.Files.writeString(java.nio.file.Path.of(profile).getParent().resolve("editor-"+ProcessHandle.current().pid()+".request"),Long.toString(System.currentTimeMillis()));}catch(Exception ignored){}}
 @Override public void render(DrawContext context,int mouseX,int mouseY,float delta){TitleAppearance.render(context,width,height);context.drawCenteredTextWithShadow(textRenderer,title,width/2,height/2-106,0xEADAF5);context.drawCenteredTextWithShadow(textRenderer,Text.literal(message),width/2,height-24,0xC8B1DC);super.render(context,mouseX,mouseY,delta);}
 @Override public void close(){client.setScreen(parent);}
}
