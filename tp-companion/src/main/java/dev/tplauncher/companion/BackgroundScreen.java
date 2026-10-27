package dev.tplauncher.companion;
import net.minecraft.client.gui.DrawContext;import net.minecraft.client.gui.screen.Screen;import net.minecraft.client.gui.widget.ButtonWidget;import net.minecraft.text.Text;
import org.lwjgl.util.tinyfd.TinyFileDialogs;import org.lwjgl.system.MemoryStack;
import java.nio.file.*;import javax.imageio.ImageIO;import java.awt.image.BufferedImage;
public final class BackgroundScreen extends Screen{
 private final Screen parent;private String message="La imagen se guarda únicamente en esta instancia.";
 public BackgroundScreen(Screen parent){super(Text.literal("Fondo del menú"));this.parent=parent;}
 @Override protected void init(){int x=width/2-120,y=height/2-20;
  addDrawableChild(ButtonWidget.builder(Text.literal("Seleccionar PNG, JPG o GIF"),b->choose()).dimensions(x,y,240,20).build());
  addDrawableChild(ButtonWidget.builder(Text.literal("Quitar fondo personalizado"),b->{try{Files.deleteIfExists(TitleAppearance.backgroundPath());message="Fondo eliminado. Se aplicará al volver al menú.";}catch(Exception e){message="No se pudo eliminar el fondo.";}}).dimensions(x,y+26,240,20).build());
  addDrawableChild(ButtonWidget.builder(Text.literal("Volver"),b->close()).dimensions(x,y+58,240,20).build());
 }
 private void choose(){try(var stack=MemoryStack.stackPush()){
  var filters=stack.mallocPointer(4);for(String ext:new String[]{"*.png","*.jpg","*.jpeg","*.gif"})filters.put(stack.UTF8(ext));filters.flip();String value=TinyFileDialogs.tinyfd_openFileDialog("Fondo de TP LAUNCHER","",filters,"Imágenes",false);if(value==null)return;Path file=Path.of(value);if(Files.size(file)>20*1024*1024)throw new IllegalArgumentException();
  try(var stream=ImageIO.createImageInputStream(file.toFile())){var readers=ImageIO.getImageReaders(stream);if(!readers.hasNext())throw new IllegalArgumentException();var reader=readers.next();try{reader.setInput(stream);int w=reader.getWidth(0),h=reader.getHeight(0);if(w<=0||h<=0||w>8192||h>8192||(long)w*h>32000000)throw new IllegalArgumentException();BufferedImage image=reader.read(0);Path out=TitleAppearance.backgroundPath();Files.createDirectories(out.getParent());ImageIO.write(image,"PNG",out.toFile());}finally{reader.dispose();}}
  message="Fondo guardado. Los GIF se usan como imagen fija.";
 }catch(Exception e){message="Imagen incompatible o demasiado grande (máximo 20 MB).";}}
 @Override public void render(DrawContext context,int mouseX,int mouseY,float delta){TitleAppearance.render(context,width,height);context.drawCenteredTextWithShadow(textRenderer,title,width/2,height/2-65,0xEADAF5);context.drawCenteredTextWithShadow(textRenderer,Text.literal(message),width/2,height-35,0xBBABCC);super.render(context,mouseX,mouseY,delta);}
 @Override public void close(){client.setScreen(parent);}
}
