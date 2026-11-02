package dev.tplauncher.client.menu;

import dev.tplauncher.client.TpClient;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.util.Identifier;
import java.io.*;
import java.nio.file.*;
import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;

public final class MenuBackground {
    private static Identifier texture;
    private static int imageWidth,imageHeight;
    private static long stamp=-1;
    public static Path path(){return MenuController.folder().resolve("background.png");}
    public static boolean validDimensions(int width,int height){return width>=32&&height>=32&&width<=4096&&height<=4096&&(long)width*height<=8388608;}
    public static byte[] normalize(Path source)throws IOException{
        if(Files.isSymbolicLink(source)||!Files.isRegularFile(source)||Files.size(source)>8388608)throw new IOException("Imagen: máximo 8 MiB, sin enlaces simbólicos");
        try(var input=ImageIO.createImageInputStream(source.toFile())){
            if(input==null)throw new IOException("No se pudo abrir la imagen");
            var readers=ImageIO.getImageReaders(input);if(!readers.hasNext())throw new IOException("Formato de imagen no compatible");
            var reader=readers.next();try{
                reader.setInput(input);String format=reader.getFormatName();if(!format.equalsIgnoreCase("PNG")&&!format.equalsIgnoreCase("JPEG"))throw new IOException("Solo PNG o JPEG");
                if(!validDimensions(reader.getWidth(0),reader.getHeight(0)))throw new IOException("Imagen: máximo 4096 px por lado y 8 megapíxeles");
                BufferedImage image=reader.read(0);ByteArrayOutputStream out=new ByteArrayOutputStream();if(!ImageIO.write(image,"PNG",out)||out.size()>8388608)throw new IOException("El PNG normalizado supera 8 MiB");return out.toByteArray();
            }finally{reader.dispose();}
        }
    }
    public static void importFile(Path source)throws IOException{
        byte[] bytes=normalize(source);Path directory=MenuController.folder();Files.createDirectories(directory);
        if(Files.isSymbolicLink(directory)||Files.isSymbolicLink(path()))throw new IOException("La carpeta del menú no puede ser un enlace");
        Path pending=Files.createTempFile(directory,"menu-",".pending");
        try{Files.write(pending,bytes);Files.move(pending,path(),StandardCopyOption.REPLACE_EXISTING);}finally{Files.deleteIfExists(pending);}
        reload();
    }
    public static void reset()throws IOException{
        if(Files.isSymbolicLink(path()))throw new IOException("Fondo no permitido");
        if(Files.isRegularFile(path()))Files.move(path(),path().resolveSibling("background.previous-"+System.currentTimeMillis()+".png"));
        reload();
    }
    public static void reload(){
        MinecraftClient client=MinecraftClient.getInstance();if(texture!=null){client.getTextureManager().destroyTexture(texture);texture=null;}stamp=-1;
        try{
            Path source=path();if(!Files.isRegularFile(source)||Files.isSymbolicLink(source)||Files.isSymbolicLink(MenuController.folder())||Files.size(source)>8388608)return;
            byte[] bytes=normalize(source);NativeImage image=NativeImage.read(bytes);
            imageWidth=image.getWidth();imageHeight=image.getHeight();texture=client.getTextureManager().registerDynamicTexture("tp-menu-background",new NativeImageBackedTexture(image));stamp=Files.getLastModifiedTime(source).toMillis();
        }catch(Exception error){TpClient.LOG.warn("Custom menu background ignored: {}",error.getClass().getSimpleName());}
    }
    public static void draw(DrawContext c,int width,int height,int mouseX,int mouseY,MenuConfig config){
        if(stamp==-1) {stamp=0;reload();if(stamp==-1)stamp=0;}
        if(texture!=null){
            double scale=Math.max((double)width/imageWidth,(double)height/imageHeight);int drawWidth=(int)Math.ceil(imageWidth*scale),drawHeight=(int)Math.ceil(imageHeight*scale);
            c.drawTexture(texture,(width-drawWidth)/2,(height-drawHeight)/2,drawWidth,drawHeight,0,0,imageWidth,imageHeight,imageWidth,imageHeight);
            c.fill(0,0,width,height,0xB307070C);
        }else{
            c.fillGradient(0,0,width,height,0xFF181124,0xFF07070C);
            int px=config.motion()&&config.parallax?(mouseX-width/2)/60:0,py=config.motion()&&config.parallax?(mouseY-height/2)/80:0;
            for(int i=0;i<12;i++){
                int x=(i*127+49)%Math.max(1,width+90)-45+px,h=36+(i*29)%Math.max(50,height/2),w=20+(i*17)%45;
                c.fill(x,height-h+py,x+w,height,0xFF100D19);c.fill(x,height-h+py,x+w,height-h+2+py,0x306F3DC2);
            }
            for(int y=height/3;y<height;y+=5){int alpha=Math.max(0,24-Math.abs(y-height*2/3)*24/Math.max(1,height/3));c.fill(0,y,width,y+5,(alpha<<24)|0x6F3DC2);}
        }
        c.fillGradient(0,0,width,height,0x5007070C,0x9007070C);
        if(config.particles&&!config.lowPower){
            double t=config.motion()?net.minecraft.util.Util.getMeasuringTimeMs()/2500.0:0;
            for(int i=0;i<18;i++){int x=(i*113+31)%Math.max(1,width),y=Math.floorMod((int)(i*67-t*(5+i%3)),Math.max(1,height));c.fill(x,y,x+1+(i%4==0?1:0),y+1,0x609B6BFF);}
        }
        c.fill(0,0,width,1,0xFF6F3DC2);
    }
    public static void close(){if(texture!=null){MinecraftClient.getInstance().getTextureManager().destroyTexture(texture);texture=null;}}
    private MenuBackground(){}
}
