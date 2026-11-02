package dev.tplauncher.client.menu;

import dev.tplauncher.client.TpClient;
import dev.tplauncher.client.TpButton;
import net.fabricmc.fabric.api.client.screen.v1.ScreenEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.text.Text;
import java.nio.file.Path;

public final class MenuController {
    public static MenuConfig config;
    private static Screen bypass;
    public static Path folder(){return TpClient.configDirectory.resolve("tp-client-menu");}
    public static void initialize(){
        config=MenuConfig.load(folder().resolve("menu.json"));
        ScreenEvents.AFTER_INIT.register((client,screen,width,height)->{
            if(screen instanceof TitleScreen title&&screen!=bypass&&config.enabled){client.execute(()->{if(client.currentScreen==screen)client.setScreen(new TpTitleScreen(title));});}
            else if(screen instanceof TitleScreen title){net.fabricmc.fabric.api.client.screen.v1.Screens.getButtons(screen).add(new TpButton(Text.literal("TP CLIENT"),8,8,100,20,()->client.setScreen(new MenuSettingsScreen(screen,title))));}
        });
        ClientTickEvents.END_CLIENT_TICK.register(client->{boolean active=client.world==null&&config.enabled&&(client.currentScreen instanceof TpTitleScreen||client.currentScreen instanceof MenuSettingsScreen);MenuMusic.tick(client,active);});
    }
    public static void vanilla(TitleScreen original){bypass=original;MinecraftClient.getInstance().setScreen(original);}
    public static void save(){try{config.save(folder().resolve("menu.json"));}catch(Exception error){TpClient.notices.add("Menú TP","No se pudo guardar la personalización");}}
    public static void close(){MenuMusic.close();MenuBackground.close();MenuLogo.close();}
    private MenuController(){}
}
