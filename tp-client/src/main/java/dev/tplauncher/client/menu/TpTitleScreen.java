package dev.tplauncher.client.menu;

import dev.tplauncher.client.*;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.gui.screen.option.AccessibilityOptionsScreen;
import net.minecraft.client.gui.screen.option.LanguageOptionsScreen;
import net.minecraft.client.gui.screen.option.OptionsScreen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.client.gui.widget.PlayerSkinWidget;
import net.minecraft.text.Text;
import net.minecraft.text.TranslatableTextContent;
import net.minecraft.client.gui.tooltip.Tooltip;

public final class TpTitleScreen extends Screen {
    private final TitleScreen vanilla;
    private MenuLayout layout;
    public TpTitleScreen(TitleScreen vanilla){super(Text.literal("TP Client"));this.vanilla=vanilla;}
    private String label(String spanish,String english){return client.options.language.startsWith("es")?spanish:english;}
    private TpButton button(Text text,int x,int y,int w,int h,Runnable action){return addDrawableChild(new TpButton(text,x,y,w,h,action));}
    private ButtonWidget original(String key){for(var child:vanilla.children())if(child instanceof ButtonWidget button&&button.getMessage().getContent() instanceof TranslatableTextContent content&&content.getKey().equals(key))return button;return null;}
    private void originalButton(String text,String key,int row){ButtonWidget original=original(key);TpButton button=button(Text.literal(text),layout.x(),layout.y(row),layout.width(),layout.row(),()->{if(original!=null&&original.active)original.onPress();});button.active=original!=null&&original.active;if(!button.active)button.setTooltip(Tooltip.of(Text.literal(label("No disponible para esta sesión. Se conservan las restricciones de Minecraft.","Unavailable for this session. Minecraft account restrictions are preserved."))));}
    @Override protected void init(){
        layout=MenuLayout.of(width,height,MenuController.config.playerPreview,MenuController.config.layout);
        originalButton(client.isDemo()?label("JUGAR DEMOSTRACIÓN","PLAY DEMO"):label("UN JUGADOR","SINGLEPLAYER"),client.isDemo()?"menu.playdemo":"menu.singleplayer",0);
        originalButton(label("MULTIJUGADOR","MULTIPLAYER"),"menu.multiplayer",1);
        originalButton("MINECRAFT REALMS","menu.online",2);
        button(Text.literal(label("COSMÉTICOS","COSMETICS")),layout.x(),layout.y(3),layout.width(),layout.row(),()->client.setScreen(new ClientScreen(this,"Cosméticos")));
        button(Text.literal("TP CLIENT"),layout.x(),layout.y(4),layout.width(),layout.row(),()->client.setScreen(new ClientScreen(this)));
        int half=(layout.width()-4)/2;
        button(Text.translatable("menu.options"),layout.x(),layout.y(5),half,layout.row(),()->client.setScreen(new OptionsScreen(this,client.options)));
        button(Text.literal(label("Personalizar","Customize")),layout.x()+half+4,layout.y(5),half,layout.row(),()->client.setScreen(new MenuSettingsScreen(this,vanilla)));
        button(Text.literal(label("Idioma","Language")),layout.x(),layout.y(6),half,layout.row(),()->client.setScreen(new LanguageOptionsScreen(this,client.options,client.getLanguageManager())));
        button(Text.literal(label("Accesibilidad","Accessibility")),layout.x()+half+4,layout.y(6),half,layout.row(),()->client.setScreen(new AccessibilityOptionsScreen(this,client.options)));
        button(Text.literal(label("Menú Minecraft","Minecraft menu")),layout.x(),layout.y(7),half,layout.row(),()->MenuController.vanilla(vanilla));
        button(Text.translatable("menu.quit"),layout.x()+half+4,layout.y(7),half,layout.row(),client::scheduleStop);
        if(layout.preview()){
            var supplier=client.getSkinProvider().getSkinTexturesSupplier(client.getGameProfile());
            PlayerSkinWidget preview=new MenuPlayerPreview(layout.previewWidth(),layout.previewHeight(),client.getEntityModelLoader(),()->Cosmetics.previewTextures(supplier.get()));
            preview.setPosition(layout.previewX(),layout.previewY());addDrawableChild(preview);
        }
    }
    @Override public void render(DrawContext c,int mouseX,int mouseY,float delta){
        MenuBackground.draw(c,width,height,mouseX,mouseY,MenuController.config);
        int center=layout.x()+layout.width()/2;
        int logoTop=Math.max(5,layout.top()-36);
        if(!MenuLogo.draw(c,center,logoTop,layout.width(),27)){
            c.getMatrices().push();c.getMatrices().translate(center,Math.max(8,layout.top()-33),0);c.getMatrices().scale(1.7f,1.7f,1);c.drawCenteredTextWithShadow(textRenderer,"TP CLIENT",0,0,0xEDEAF7);c.getMatrices().pop();
        }
        c.fill(center-25,layout.top()-12,center+25,layout.top()-11,0xFF9B6BFF);
        if(layout.preview()){
            int x=layout.previewX()+layout.previewWidth()/2,y=Math.min(height-26,layout.previewY()+layout.previewHeight()+8);
            c.drawCenteredTextWithShadow(textRenderer,textRenderer.trimToWidth(client.getSession().getUsername(),layout.previewWidth()),x,y,0xDAC8F4);
            if(width>440)c.drawCenteredTextWithShadow(textRenderer,label("Arrastra para girar","Drag to rotate"),x,y+12,0x857B96);
        }
        c.drawText(textRenderer,"1.21.1 · Fabric · TP Client 0.1.0",8,height-20,0x9589A6,false);
        if(height>250)c.drawText(textRenderer,label("Producto independiente · Sin afiliación con Mojang o Microsoft","Independent product · Not affiliated with Mojang or Microsoft"),8,height-10,0x6E657C,false);
        super.render(c,mouseX,mouseY,delta);
    }
    @Override public void renderBackground(DrawContext c,int mouseX,int mouseY,float delta){}
    @Override public boolean shouldCloseOnEsc(){return false;}
    @Override public boolean shouldPause(){return false;}
}
