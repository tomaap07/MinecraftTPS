package dev.tplauncher.companion;
import com.google.gson.*;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.util.Identifier;
import java.nio.file.*;import java.nio.charset.StandardCharsets;

public final class TitleAppearance{
 private static boolean loaded,tp=true;private static Identifier background;private static long stamp;
 private static Path folder(){return MinecraftClient.getInstance().runDirectory.toPath().resolve("config/tp-launcher");}
 public static boolean themed(){if(!loaded){loaded=true;try{JsonObject data=JsonParser.parseString(Files.readString(folder().resolve("menu.json"),StandardCharsets.UTF_8)).getAsJsonObject();tp=data.get("tp_theme").getAsBoolean();}catch(Exception ignored){}}return tp;}
 public static void toggle(){tp=!themed();save();}
 public static void save(){try{Files.createDirectories(folder());Files.writeString(folder().resolve("menu.json"),"{\"tp_theme\":"+tp+"}",StandardCharsets.UTF_8);}catch(Exception ignored){}}
 public static void render(DrawContext context,int width,int height){
  MinecraftClient c=MinecraftClient.getInstance();Path path=folder().resolve("background.png");
  if(!Files.exists(path)&&background!=null){c.getTextureManager().destroyTexture(background);background=null;stamp=0;}
  try{if(Files.isRegularFile(path)&&Files.size(path)<20*1024*1024&&Files.getLastModifiedTime(path).toMillis()!=stamp){stamp=Files.getLastModifiedTime(path).toMillis();NativeImage image=ScreenshotScreen.image(path);if(image.getWidth()>8192||image.getHeight()>8192){image.close();throw new IllegalArgumentException();}if(background!=null)c.getTextureManager().destroyTexture(background);background=c.getTextureManager().registerDynamicTexture("tp-menu",new NativeImageBackedTexture(image));}}catch(Exception ignored){}
  if(background!=null){context.setShaderColor(1,1,1,1);context.drawTexture(background,0,0,0,0,width,height,width,height);context.fill(0,0,width,height,0x8806080D);}
  else{context.fillGradient(0,0,width,height,0xFF16101F,0xFF060608);for(int x=0;x<width;x+=32)context.fill(x,0,x+1,height,0x105A4176);for(int y=0;y<height;y+=32)context.fill(0,y,width,y+1,0x105A4176);}
  context.fill(0,0,width,3,0xFFA855F7);
 }
 public static Path backgroundPath(){return folder().resolve("background.png");}
}
