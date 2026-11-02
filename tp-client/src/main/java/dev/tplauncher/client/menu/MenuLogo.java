package dev.tplauncher.client.menu;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.util.Identifier;
import java.io.IOException;
import java.nio.file.*;

public final class MenuLogo {
    private static Identifier texture;
    private static int width,height;
    private static boolean loaded;
    private static Path file(){return MenuController.folder().resolve("logo.png");}
    public static void importFile(Path source)throws IOException{
        byte[] bytes=MenuBackground.normalize(source);Files.createDirectories(MenuController.folder());
        if(Files.isSymbolicLink(MenuController.folder())||Files.isSymbolicLink(file()))throw new IOException("No se permiten enlaces simbólicos");
        Path pending=Files.createTempFile(MenuController.folder(),"logo-",".pending");
        try{Files.write(pending,bytes);Files.move(pending,file(),StandardCopyOption.REPLACE_EXISTING);}finally{Files.deleteIfExists(pending);}
        close();
    }
    public static void reset()throws IOException{
        if(Files.isSymbolicLink(MenuController.folder())||Files.isSymbolicLink(file()))throw new IOException("No se permiten enlaces simbólicos");
        if(Files.isRegularFile(file()))Files.move(file(),file().resolveSibling("logo.previous-"+System.currentTimeMillis()+".png"));close();
    }
    private static void load(){
        loaded=true;
        try{
            if(Files.isSymbolicLink(MenuController.folder())||Files.isSymbolicLink(file())||!Files.isRegularFile(file()))return;
            NativeImage image=NativeImage.read(MenuBackground.normalize(file()));width=image.getWidth();height=image.getHeight();
            texture=MinecraftClient.getInstance().getTextureManager().registerDynamicTexture("tp-menu-logo",new NativeImageBackedTexture(image));
        }catch(Exception ignored){}
    }
    public static boolean draw(DrawContext context,int center,int top,int availableWidth,int availableHeight){
        if(!loaded)load();if(texture==null)return false;
        double scale=Math.min((double)availableWidth/width,(double)availableHeight/height);int w=Math.max(1,(int)(width*scale)),h=Math.max(1,(int)(height*scale));
        context.drawTexture(texture,center-w/2,top+(availableHeight-h)/2,w,h,0,0,width,height,width,height);return true;
    }
    public static void close(){if(texture!=null)MinecraftClient.getInstance().getTextureManager().destroyTexture(texture);texture=null;loaded=false;}
    private MenuLogo(){}
}
