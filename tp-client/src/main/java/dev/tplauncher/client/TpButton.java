package dev.tplauncher.client;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

public final class TpButton extends ButtonWidget {
    private boolean selected;
    public TpButton(Text text,int x,int y,int width,int height,Runnable action){super(x,y,width,height,text,button->action.run(),DEFAULT_NARRATION_SUPPLIER);}
    public TpButton selected(boolean selected){this.selected=selected;return this;}
    @Override protected void renderWidget(DrawContext c,int mouseX,int mouseY,float delta){
        boolean hover=isHovered()||isFocused();int x=getX(),y=getY(),w=getWidth(),h=getHeight();
        int accent=TpClient.config==null?0xB678F5:TpClient.config.accent;
        c.fill(x-1,y-1,x+w+1,y+h+1,selected?0x553D2158:hover?0x334E2C6C:0x1A020307);
        c.fillGradient(x,y,x+w,y+h,selected?0xF039244D:hover?0xF0272034:0xF0181722,selected?0xF021182F:0xF00E1119);
        c.drawBorder(x,y,w,h,!active?0x55382F42:selected?0xDD000000|accent:hover?0xAA000000|accent:0x80433855);
        if(selected)c.fill(x,y,x+3,y+h,0xFF000000|accent);
        var font=MinecraftClient.getInstance().textRenderer;
        String label=font.trimToWidth(getMessage().getString(),Math.max(8,w-14));
        c.drawText(font,label,x+(w-font.getWidth(label))/2,y+(h-font.fontHeight)/2,!active?0xFF6F657E:selected?0xFFF1E6FF:hover?0xFFE7D4FF:0xFFC8C4D0,false);
    }
}
