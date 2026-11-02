package dev.tplauncher.client;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import net.minecraft.client.util.ScreenshotRecorder;
import org.lwjgl.glfw.GLFW;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import java.nio.file.*;
import java.util.*;

public final class TpClient implements ClientModInitializer {
    public static final Logger LOG=LoggerFactory.getLogger("TP Client");
    public static ClientConfig config;
    public static Path configDirectory, configFile;
    public static final FrameMetrics frames=new FrameMetrics();
    public static final long started=System.nanoTime();
    public static final Notices notices=new Notices();
    public static final HudRenderer hud=new HudRenderer();
    private static KeyBinding menu,editor,zoom,screenshot;
    private static boolean externalZoom;
    private static int ticks;
    private static final ArrayDeque<Long> leftClicks=new ArrayDeque<>(),rightClicks=new ArrayDeque<>();

    @Override public void onInitializeClient() {
        configDirectory=FabricLoader.getInstance().getConfigDir();configFile=configDirectory.resolve("tp-client.json");
        config=ConfigStore.loadSafely(configFile);
        menu=key("menu",GLFW.GLFW_KEY_RIGHT_SHIFT);editor=key("editor",GLFW.GLFW_KEY_F8);zoom=key("zoom",GLFW.GLFW_KEY_C);screenshot=key("screenshot",GLFW.GLFW_KEY_F9);
        externalZoom=List.of("zoomify","ok_zoomer","wi_zoom").stream().anyMatch(id->FabricLoader.getInstance().isModLoaded(id));
        Cosmetics.initialize();
        dev.tplauncher.client.menu.MenuController.initialize();
        net.fabricmc.fabric.api.client.screen.v1.ScreenEvents.AFTER_INIT.register((client,screen,width,height)->{
            if(screen instanceof net.minecraft.client.gui.screen.GameMenuScreen){
                net.fabricmc.fabric.api.client.screen.v1.Screens.getButtons(screen).add(new TpButton(net.minecraft.text.Text.literal("TP CLIENT"),8,8,100,20,()->client.setScreen(new ClientScreen(screen))));
            }
        });
        ClientTickEvents.END_CLIENT_TICK.register(client->{
            while(menu.wasPressed())if(!(client.currentScreen instanceof ClientScreen))client.setScreen(new ClientScreen(client.currentScreen));
            while(editor.wasPressed())if(client.player!=null)client.setScreen(new HudEditorScreen(client.currentScreen));
            while(screenshot.wasPressed())capture(client);
            if(ticks++%5==0)hud.update(client);
            if(ticks%100==0){PerformanceReporter.sample(client);Cosmetics.reload(client);}
            prune(leftClicks);prune(rightClicks);
        });
        HudRenderCallback.EVENT.register((context,counter)->{
            frames.frame(System.nanoTime());
            MinecraftClient client=MinecraftClient.getInstance();
            if(!(client.currentScreen instanceof HudEditorScreen))hud.render(context,false,null);
            notices.render(context,client);
        });
        ClientLifecycleEvents.CLIENT_STOPPING.register(client->{save();PerformanceReporter.close();Cosmetics.close(client);dev.tplauncher.client.menu.MenuController.close();});
        notices.add("TP Client", "Shift derecho: menú · F8: editar HUD · C: zoom");
        LOG.info("TP Client initialized for Fabric 1.21.1; local HUD does not require the launcher");
    }
    private static KeyBinding key(String id,int code){return KeyBindingHelper.registerKeyBinding(new KeyBinding("key.tp_client."+id,InputUtil.Type.KEYSYM,code,"key.categories.tp_client"));}
    public static boolean zooming(){MinecraftClient c=MinecraftClient.getInstance();return config!=null&&config.zoomEnabled&&!externalZoom&&zoom!=null&&zoom.isPressed()&&c.player!=null&&c.currentScreen==null;}
    public static void click(int button,int action){MinecraftClient c=MinecraftClient.getInstance();if(action!=GLFW.GLFW_PRESS||c.player==null||c.currentScreen!=null)return;ArrayDeque<Long> clicks=button==GLFW.GLFW_MOUSE_BUTTON_LEFT?leftClicks:button==GLFW.GLFW_MOUSE_BUTTON_RIGHT?rightClicks:null;if(clicks!=null){prune(clicks);if(clicks.size()<512)clicks.addLast(System.nanoTime());}}
    private static void prune(ArrayDeque<Long> values){long min=System.nanoTime()-1_000_000_000L;while(!values.isEmpty()&&values.peekFirst()<min)values.removeFirst();}
    public static int cps(boolean left){ArrayDeque<Long> values=left?leftClicks:rightClicks;prune(values);return values.size();}
    public static void save(){try{ConfigStore.write(configFile,config);}catch(Exception error){notices.add("No se guardó el perfil", "Revisa los permisos de la carpeta config");LOG.warn("Could not save TP Client settings: {}",error.getClass().getSimpleName());}}
    public static void capture(MinecraftClient c){ScreenshotRecorder.saveScreenshot(c.runDirectory,c.getFramebuffer(),text->c.execute(()->notices.add("Captura",text.getString())));}
    public static boolean zoomConflict(){return externalZoom;}
    public static void importProfile(){try{ClientConfig loaded=ConfigStore.read(configDirectory.resolve("tp-client-import.json"));config=loaded;save();notices.add("Perfil importado",config.profile);}catch(Exception e){notices.add("No se importó", "Guarda un perfil válido en config/tp-client-import.json");}}
    public static void exportProfile(String name){try{ConfigStore.write(ConfigStore.profilePath(configDirectory,name),config);notices.add("Perfil exportado","config/tp-client-profiles/"+name+".json");}catch(Exception e){notices.add("No se exportó", "Usa un nombre de 1–48 letras, números o guiones");}}
}
