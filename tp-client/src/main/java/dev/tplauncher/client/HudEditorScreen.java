package dev.tplauncher.client;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;
import org.lwjgl.glfw.GLFW;

public final class HudEditorScreen extends Screen {
    private final Screen parent;
    private WidgetKind selected;
    private double dragX,dragY,originX,originY;
    public HudEditorScreen(Screen parent){super(Text.literal("Editor HUD TP Client"));this.parent=parent;}
    @Override public void render(DrawContext c,int mouseX,int mouseY,float delta){
        c.fill(0,0,width,height,0x40100820);
        for(int x=0;x<width;x+=TpClient.config.grid*4)c.fill(x,0,x+1,height,0x145D407F);
        for(int y=0;y<height;y+=TpClient.config.grid*4)c.fill(0,y,width,y+1,0x145D407F);
        c.fill(width/2,0,width/2+1,height,0x407C55A4);c.fill(0,height/2,width,height/2+1,0x407C55A4);
        TpClient.hud.render(c,true,selected);
        c.fill(0,height-35,width,height,0xF0100C18);c.fill(0,height-36,width,height-35,0x99A855F7);
        c.drawCenteredTextWithShadow(textRenderer,Text.literal(textRenderer.trimToWidth("Arrastra · Rueda: tamaño · Shift+rueda: opacidad · L: bloqueo",width-16)),width/2,height-30,0xFFE4D0FF);
        c.drawCenteredTextWithShadow(textRenderer,Text.literal("R: restaurar · Flechas: mover · Esc: guardar"),width/2,height-16,0xFFB5A8C6);
        if(selected!=null){var b=TpClient.hud.bounds().get(selected);if(b!=null){c.fill(b.x(),0,b.x()+1,height-35,0x608A5AAB);c.fill(0,b.y(),width,b.y()+1,0x608A5AAB);}}
    }
    @Override public boolean mouseClicked(double x,double y,int button){
        var entries=new java.util.ArrayList<>(TpClient.hud.bounds().entrySet());java.util.Collections.reverse(entries);
        for(var entry:entries)if(entry.getValue().contains(x,y)){selected=entry.getKey();var w=TpClient.config.widgets.get(selected.id());var b=entry.getValue();double baseX=w.anchor.endsWith("right")?width-b.width()-6:w.anchor.endsWith("center")?(width-b.width())/2.0:6;double baseY=w.anchor.startsWith("bottom")?height-b.height()-6:6;if(!w.locked){w.x=b.x()-baseX;w.y=b.y()-baseY;w.positioned=true;}dragX=x;dragY=y;originX=w.x;originY=w.y;return true;}
        return false;
    }
    @Override public boolean mouseDragged(double x,double y,int button,double dx,double dy){if(selected==null)return false;var w=TpClient.config.widgets.get(selected.id());if(w.locked)return true;w.x=HudMath.snap(originX+x-dragX,TpClient.config.grid);w.y=HudMath.snap(originY+y-dragY,TpClient.config.grid);return true;}
    @Override public boolean mouseReleased(double x,double y,int button){TpClient.save();return true;}
    @Override public boolean mouseScrolled(double x,double y,double horizontal,double vertical){
        if(selected==null)return false;var w=TpClient.config.widgets.get(selected.id());if(hasShiftDown())w.opacity=HudMath.finite(w.opacity+vertical*0.05,0.9,0.1,1);else w.scale=HudMath.finite(w.scale+vertical*0.1,1,0.5,3);TpClient.save();return true;
    }
    @Override public boolean keyPressed(int key,int scan,int modifiers){if(selected!=null){var w=TpClient.config.widgets.get(selected.id());if(key==GLFW.GLFW_KEY_L){w.locked=!w.locked;TpClient.save();return true;}if(key==GLFW.GLFW_KEY_R){TpClient.config.widgets.put(selected.id(),new ClientConfig().widgets.get(selected.id()));TpClient.save();return true;}if(!w.locked){int amount=hasShiftDown()?TpClient.config.grid:1;if(key==GLFW.GLFW_KEY_LEFT)w.x-=amount;else if(key==GLFW.GLFW_KEY_RIGHT)w.x+=amount;else if(key==GLFW.GLFW_KEY_UP)w.y-=amount;else if(key==GLFW.GLFW_KEY_DOWN)w.y+=amount;else return super.keyPressed(key,scan,modifiers);w.positioned=true;TpClient.save();return true;}}return super.keyPressed(key,scan,modifiers);}
    @Override public void close(){TpClient.save();client.setScreen(parent);}
    @Override public boolean shouldPause(){return false;}
}
