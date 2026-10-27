package dev.tplauncher.companion;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.loader.api.FabricLoader;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import net.minecraft.item.ItemStack;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;
import org.lwjgl.glfw.GLFW;
import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import java.util.*;

public final class TpCompanion implements ClientModInitializer {
    private static KeyBinding zoom;
    private static boolean externalZoom;
    private int ticks;
    private static String clock="";
    @Override public void onInitializeClient(){
        Profile.reload();
        TelemetryBridge.init();
        externalZoom=List.of("zoomify","ok_zoomer","wi_zoom").stream().anyMatch(id->FabricLoader.getInstance().isModLoaded(id));
        zoom=KeyBindingHelper.registerKeyBinding(new KeyBinding("key.tp_companion.zoom",InputUtil.Type.KEYSYM,GLFW.GLFW_KEY_C,"key.categories.tp_launcher"));
        ClientTickEvents.END_CLIENT_TICK.register(client->{if(ticks++%4==0){Profile.reload();ControlBridge.poll(client);TelemetryBridge.write(client);}if(ticks%20==0){clock=LocalTime.now().format(DateTimeFormatter.ofPattern("HH:mm"));Minimap.update(client);}});
        HudRenderCallback.EVENT.register((context,tickCounter)->render(context));
    }
    public static boolean zooming(){MinecraftClient client=MinecraftClient.getInstance();return !externalZoom&&Profile.get().enabled("optical_zoom")&&zoom!=null&&zoom.isPressed()&&client.player!=null&&client.currentScreen==null;}
    private static Text text(String value){Text text=Text.literal(value);return Profile.get().enabled("custom_font")?text.copy().styled(style->style.withFont(Identifier.of("minecraft","uniform"))):text;}
    private static void render(DrawContext context){
        MinecraftClient client=MinecraftClient.getInstance();if(client.player==null||client.world==null||client.options.hudHidden)return;
        Profile.Snapshot profile=Profile.get();float scale=(float)profile.scale();
        int width=(int)(client.getWindow().getScaledWidth()/scale),height=(int)(client.getWindow().getScaledHeight()/scale);
        Map<String,Integer> offsets=new HashMap<>();
        context.getMatrices().push();context.getMatrices().scale(scale,scale,1);
        try{for(Profile.Module module:profile.modules()){
            if(!module.enabled())continue;
            if(client.hasReducedDebugInfo()&&List.of("coordinates","direction","minimap").contains(module.id()))continue;
            if(profile.nativeOverlay()&&TelemetryBridge.nativeVisible())continue;
            List<String> lines=lines(module.id(),client);boolean armor=module.id().equals("armor"),map=module.id().equals("minimap");
            if(lines.isEmpty()&&!armor&&!map)continue;
            int boxWidth=map?104:armor?100:30;for(String line:lines)boxWidth=Math.max(boxWidth,client.textRenderer.getWidth(text(line))+12);
            int boxHeight=map?114:armor?118:lines.size()*12+10,offset=offsets.getOrDefault(module.anchor(),0);
            boolean bottom=module.anchor().startsWith("bottom");
            int x=module.anchor().endsWith("right")?width-boxWidth-8:module.anchor().endsWith("center")?(width-boxWidth)/2:8;
            int y=bottom?height-8-offset-boxHeight:8+offset;offsets.put(module.anchor(),offset+boxHeight+5);
            if(profile.background())context.fill(x,y,x+boxWidth,y+boxHeight,((int)(profile.opacity()*190)<<24)|0x101019);
            int color=((int)(profile.opacity()*255)<<24)|0xE5D4F7;
            if(map){context.drawText(client.textRenderer,text("N · 96 × 96"),x+4,y+4,color,true);Minimap.render(context,client,x+4,y+14);}
            else if(armor){List<ItemStack> gear=new ArrayList<>();for(var slot:new net.minecraft.entity.EquipmentSlot[]{net.minecraft.entity.EquipmentSlot.HEAD,net.minecraft.entity.EquipmentSlot.CHEST,net.minecraft.entity.EquipmentSlot.LEGS,net.minecraft.entity.EquipmentSlot.FEET})gear.add(client.player.getEquippedStack(slot));gear.add(client.player.getMainHandStack());gear.add(client.player.getOffHandStack());int row=0;for(ItemStack stack:gear){if(stack.isEmpty())continue;context.drawItem(stack,x+4,y+4+row*18);String durability=stack.isDamageable()?Math.round(Math.max(0,stack.getMaxDamage()-stack.getDamage())*100.0/stack.getMaxDamage())+"%":"—";context.drawText(client.textRenderer,text(durability),x+25,y+8+row++*18,color,true);}}
            else{int row=0;for(String line:lines)context.drawText(client.textRenderer,text(line),x+6,y+5+row++*12,color,true);}
        }}finally{context.getMatrices().pop();}
    }
    private static List<String> lines(String id,MinecraftClient client){
        return switch(id){
            case "fps"->List.of("FPS: "+client.getCurrentFps());
            case "coordinates"->List.of("XYZ: "+client.player.getBlockX()+" / "+client.player.getBlockY()+" / "+client.player.getBlockZ());
            case "direction"->List.of(client.player.getHorizontalFacing().asString().toUpperCase(Locale.ROOT));
            case "clock"->List.of(clock);
            case "ping"->{var handler=client.getNetworkHandler();var entry=handler==null?null:handler.getPlayerListEntry(client.player.getUuid());yield List.of(entry==null?"Ping: —":"Ping: "+entry.getLatency()+" ms");}
            case "keystrokes"->List.of(key(client.options.forwardKey),key(client.options.leftKey)+" "+key(client.options.backKey)+" "+key(client.options.rightKey),(client.options.attackKey.isPressed()?"[LMB]":"LMB")+" "+(client.options.useKey.isPressed()?"[RMB]":"RMB"));
            default->List.of();
        };
    }
    private static String key(KeyBinding binding){String label=binding.getBoundKeyLocalizedText().getString();return binding.isPressed()?"["+label+"]":label;}
}
