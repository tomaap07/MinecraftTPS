package dev.tplauncher.client;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import java.util.ArrayDeque;

public final class Notices {
    private record Notice(String title,String message,long expires){}
    private final ArrayDeque<Notice> notices=new ArrayDeque<>();
    public void add(String title,String message){if(notices.size()>=8)notices.removeFirst();notices.addLast(new Notice(title, message.length()>100?message.substring(0,100):message,System.nanoTime()+5_000_000_000L));}
    public void render(DrawContext c,MinecraftClient mc){
        while(!notices.isEmpty()&&notices.peekFirst().expires<System.nanoTime())notices.removeFirst();
        if(notices.isEmpty()||TpClient.config==null||!TpClient.config.notifications||mc.options.hudHidden)return;
        if(TpClient.config.streamer&&TpClient.config.hideNotifications)return;
        if(mc.player!=null&&mc.player.hurtTime>0)return;
        Notice n=notices.peekFirst();var area=HudLayout.notificationArea(mc.getWindow().getScaledWidth());int width=area.width(),x=area.x();if(width==0)return;
        c.fill(x,8,x+width,42,0xE812101C);c.fill(x,8,x+3,42,0xFFA855F7);
        c.drawText(mc.textRenderer,n.title,x+9,13,0xFFE9D9FF,true);
        c.drawText(mc.textRenderer,mc.textRenderer.trimToWidth(n.message,width-18),x+9,28,0xFFB9B0C9,false);
    }
}
