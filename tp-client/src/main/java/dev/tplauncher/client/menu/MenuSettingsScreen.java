package dev.tplauncher.client.menu;

import dev.tplauncher.client.TpButton;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.gui.widget.SliderWidget;
import net.minecraft.text.Text;
import net.minecraft.util.Util;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.util.tinyfd.TinyFileDialogs;
import java.nio.file.Path;
import java.nio.file.Files;
import java.util.*;

public final class MenuSettingsScreen extends Screen {
    private final Screen parent;
    private final TitleScreen vanilla;
    private String page="Visual",message="";
    private int scroll;
    public MenuSettingsScreen(Screen parent,TitleScreen vanilla){super(Text.literal("TP Client · Menú"));this.parent=parent;this.vanilla=vanilla;}
    private String text(String es,String en){return client.options.language.startsWith("es")?es:en;}
    private String on(boolean value){return value?"ON":"OFF";}
    private void button(String label,int x,int y,int w,int h,Runnable action){addDrawableChild(new TpButton(Text.literal(label),x,y,w,h,action));}
    private void change(Runnable action){action.run();MenuController.save();refresh();}
    private record Action(String text,Runnable action){}
    @Override protected void init(){
        int x=Math.max(10,width/2-150),w=Math.min(300,width-20),tw=(w-8)/3;
        int i=0;for(String category:List.of("Visual","Audio","Accesibilidad")){String selected=category;button(category,x+i++*(tw+4),34,tw,20,()->{page=selected;scroll=0;refresh();});}
        List<Action> actions=new ArrayList<>();MenuConfig cfg=MenuController.config;
        switch(page){
            case "Visual"->{
                actions.add(new Action(text("Menú TP: ","TP menu: ")+on(cfg.enabled),()->{cfg.enabled=!cfg.enabled;}));
                actions.add(new Action(text("Elegir fondo PNG o JPG","Choose PNG or JPG background"),()->choose(false)));
                actions.add(new Action(text("Elegir logo PNG o JPG","Choose PNG or JPG logo"),()->choose(true)));
                actions.add(new Action(text("Restaurar logo TP Client","Restore TP Client logo"),()->{try{MenuLogo.reset();message=text("Logo anterior conservado como copia.","Previous logo kept as a backup.");}catch(Exception e){message=text("No se pudo restaurar el logo.","Could not restore the logo.");}}));
                actions.add(new Action(text("Restaurar fondo original","Restore original background"),()->{try{MenuBackground.reset();message=text("Fondo anterior conservado como copia local.","Previous background kept as a local backup.");}catch(Exception e){message=text("No se pudo restaurar el fondo.","Could not restore the background.");}}));
                actions.add(new Action(text("Partículas: ","Particles: ")+on(cfg.particles),()->cfg.particles=!cfg.particles));
                actions.add(new Action("Parallax: "+on(cfg.parallax),()->cfg.parallax=!cfg.parallax));
                actions.add(new Action(text("Vista 3D del jugador: ","3D player preview: ")+on(cfg.playerPreview),()->cfg.playerPreview=!cfg.playerPreview));
                actions.add(new Action(text("Disposición: ","Layout: ")+(cfg.layout.equals("balanced")?text("Equilibrada","Balanced"):text("Centrada","Centered")),()->cfg.layout=cfg.layout.equals("balanced")?"center":"balanced"));
            }
            case "Audio"->{
                actions.add(new Action(text("Música original: ","Original music: ")+on(cfg.musicEnabled),()->cfg.musicEnabled=!cfg.musicEnabled));
                actions.add(new Action(text("Abrir carpeta del menú","Open menu folder"),()->{try{Files.createDirectories(MenuController.folder());Util.getOperatingSystem().open(MenuController.folder().toFile());}catch(Exception e){message=text("No se pudo abrir la carpeta.","Could not open the folder.");}}));
                addDrawableChild(new SliderWidget(x,88,w,20,Text.empty(),cfg.musicVolume){
                    {updateMessage();}
                    @Override protected void updateMessage(){setMessage(Text.literal(text("Volumen del menú: ","Menu volume: ")+Math.round(value*100)+"%"));}
                    @Override protected void applyValue(){MenuController.config.musicVolume=value;MenuController.save();}
                    @Override public void renderWidget(DrawContext c,int mx,int my,float delta){
                        int bx=getX(),by=getY(),bw=getWidth(),bh=getHeight();c.fill(bx,by,bx+bw,by+bh,0xF0181522);c.drawBorder(bx,by,bw,bh,isFocused()?0xFFBD64F4:0xFF514464);c.fill(bx+2,by+bh-4,bx+2+(int)((bw-4)*value),by+bh-2,0xFF9B6BFF);c.drawCenteredTextWithShadow(textRenderer,getMessage(),bx+bw/2,by+6,0xEDEAF7);
                    }
                });
            }
            default->{
                actions.add(new Action(text("Movimiento reducido: ","Reduced motion: ")+on(cfg.reducedMotion),()->cfg.reducedMotion=!cfg.reducedMotion));
                actions.add(new Action(text("Bajo consumo: ","Low power: ")+on(cfg.lowPower),()->cfg.lowPower=!cfg.lowPower));
                actions.add(new Action(text("Movimiento de la skin: ","Player motion: ")+on(cfg.playerMotion),()->cfg.playerMotion=!cfg.playerMotion));
                actions.add(new Action(text("Restaurar ajustes del menú","Reset menu settings"),()->{MenuController.config=new MenuConfig();message=text("Ajustes restaurados. El PNG personalizado se conserva.","Settings restored. Custom PNG is preserved.");}));
            }
        }
        int top=page.equals("Audio")?116:66,row=23,rows=Math.max(1,(height-top-65)/row);
        scroll=Math.max(0,Math.min(scroll,Math.max(0,actions.size()-rows)));
        for(int index=scroll;index<Math.min(actions.size(),scroll+rows);index++){Action action=actions.get(index);button(action.text(),x,top+(index-scroll)*row,w,20,()->change(action.action()));}
        if(actions.size()>rows){button("↑",x,height-49,28,18,()->{scroll--;refresh();});button("↓",x+32,height-49,28,18,()->{scroll++;refresh();});}
        button(text("Volver","Back"),Math.max(10,width/2-60),height-27,120,20,this::close);
    }
    private void choose(boolean logo){
        try(var stack=MemoryStack.stackPush()){
            var filters=stack.mallocPointer(3);for(String extension:new String[]{"*.png","*.jpg","*.jpeg"})filters.put(stack.UTF8(extension));filters.flip();
            String file=TinyFileDialogs.tinyfd_openFileDialog(text("Fondo local de TP Client","TP Client local background"),"",filters,"PNG / JPEG",false);
            if(file!=null){if(logo)MenuLogo.importFile(Path.of(file));else MenuBackground.importFile(Path.of(file));message=text("Imagen guardada en esta instancia.","Image saved in this instance.");}
        }catch(Exception error){message=text("Imagen no válida: PNG/JPG, hasta 8 MiB y 8 megapíxeles.","Invalid image: PNG/JPG, up to 8 MiB and 8 megapixels.");}
    }
    private void refresh(){clearChildren();init();}
    @Override public boolean mouseScrolled(double x,double y,double horizontal,double vertical){scroll-=(int)Math.signum(vertical);refresh();return true;}
    @Override public void render(DrawContext c,int mx,int my,float delta){
        MenuBackground.draw(c,width,height,mx,my,MenuController.config);c.fill(Math.max(0,width/2-165),20,Math.min(width,width/2+165),height-10,0xC007070C);
        c.drawCenteredTextWithShadow(textRenderer,title,width/2,13,0xEDEAF7);
        if(page.equals("Audio"))c.drawCenteredTextWithShadow(textRenderer,Text.literal(text("Endless Dusk · composición original de TP LABS","Endless Dusk · original composition by TP LABS")),width/2,69,0xBBA7CF);
        if(!message.isEmpty())c.drawCenteredTextWithShadow(textRenderer,textRenderer.trimToWidth(message,width-20),width/2,height-39,0xC8B4E6);
        super.render(c,mx,my,delta);
    }
    @Override public void close(){MenuController.save();if(!MenuController.config.enabled)MenuController.vanilla(vanilla);else client.setScreen(new TpTitleScreen(vanilla));}
    @Override public void renderBackground(DrawContext c,int mx,int my,float delta){}
    @Override public boolean shouldPause(){return false;}
}
