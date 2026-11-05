package dev.tplauncher.client.menu;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.widget.PlayerSkinWidget;
import net.minecraft.client.render.entity.model.EntityModelLoader;
import net.minecraft.client.util.SkinTextures;
import net.minecraft.util.Util;
import java.util.function.Supplier;

final class MenuPlayerPreview extends PlayerSkinWidget {
    private double previousX,previousY;
    private boolean observed;
    MenuPlayerPreview(int width,int height,EntityModelLoader models,Supplier<SkinTextures> skins){super(width,height,models,skins);}
    @Override protected void renderWidget(DrawContext c,int mouseX,int mouseY,float delta){
        var config=MenuController.config;
        boolean motion=config.motion()&&config.playerMotion;
        double x=Math.max(-300,Math.min(300,mouseX-(getX()+getWidth()/2.0)));
        double y=Math.max(-160,Math.min(160,mouseY-(getY()+getHeight()/2.0)));
        if(observed&&motion)super.onDrag(mouseX,mouseY,(x-previousX)*0.018,(y-previousY)*0.012);
        previousX=x;previousY=y;observed=true;
        c.getMatrices().push();
        if(motion)c.getMatrices().translate(0,Math.sin(Util.getMeasuringTimeMs()/1150.0)*0.7,0);
        try{super.renderWidget(c,mouseX,mouseY,delta);}finally{c.getMatrices().pop();}
    }
}
