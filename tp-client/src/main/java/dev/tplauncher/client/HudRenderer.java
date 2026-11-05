package dev.tplauncher.client;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.ChatScreen;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.entity.EquipmentSlot;
import net.minecraft.item.ItemStack;
import net.minecraft.util.math.MathHelper;
import net.minecraft.world.LightType;
import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import java.util.*;

public final class HudRenderer {
    public record Bounds(int x,int y,int width,int height) { public boolean contains(double px,double py){return px>=x&&py>=y&&px<=x+width&&py<=y+height;} }
    private final EnumMap<WidgetKind,List<String>> content=new EnumMap<>(WidgetKind.class);
    private final EnumMap<WidgetKind,List<ItemStack>> equipment=new EnumMap<>(WidgetKind.class);
    private final EnumMap<WidgetKind,Bounds> bounds=new EnumMap<>(WidgetKind.class);
    private final Set<WidgetKind> failed=EnumSet.noneOf(WidgetKind.class);
    private record Measured(List<String> lines,List<ItemStack> items,int width,int height){}
    private final EnumMap<WidgetKind,Measured> measured=new EnumMap<>(WidgetKind.class);
    private int hiddenWidgets;
    private double lastX,lastZ,speed;private Object lastWorld;private long speedTime;
    public Map<WidgetKind,Bounds> bounds(){return Collections.unmodifiableMap(bounds);}
    public int hiddenWidgets(){return hiddenWidgets;}

    public void update(MinecraftClient c){
        if(c.world!=lastWorld){lastWorld=c.world;speedTime=0;TpClient.frames.reset();}
        if(c.player!=null){long now=System.nanoTime();if(speedTime>0)speed=Math.hypot(c.player.getX()-lastX,c.player.getZ()-lastZ)*1e9/(now-speedTime);lastX=c.player.getX();lastZ=c.player.getZ();speedTime=now;}
        for(WidgetKind kind:WidgetKind.values())try{content.put(kind,lines(kind,c));}catch(Exception error){content.put(kind,List.of("No disponible"));if(failed.add(kind))TpClient.LOG.warn("Widget {} isolated after {}",kind.id(),error.getClass().getSimpleName());}
        equipment.clear();if(c.player!=null){equipment.put(WidgetKind.ARMOR,List.of(c.player.getEquippedStack(EquipmentSlot.HEAD),c.player.getEquippedStack(EquipmentSlot.CHEST),c.player.getEquippedStack(EquipmentSlot.LEGS),c.player.getEquippedStack(EquipmentSlot.FEET)));equipment.put(WidgetKind.HANDS,List.of(c.player.getMainHandStack(),c.player.getOffHandStack()));}
    }
    public void render(DrawContext context,boolean edit,WidgetKind selected){
        MinecraftClient c=MinecraftClient.getInstance();bounds.clear();
        if(TpClient.config==null||!TpClient.config.hudEnabled||c.options.hudHidden||c.world==null||c.player==null)return;
        var entries=new ArrayList<>(Arrays.asList(WidgetKind.values()));entries.sort(Comparator.comparingInt(k->TpClient.config.widgets.get(k.id()).order));
        List<HudLayout.Request> requests=new ArrayList<>();measured.clear();
        for(WidgetKind kind:entries){
            ClientConfig.Widget w=TpClient.config.widgets.get(kind.id());if(!w.enabled)continue;
            if(!edit&&((c.isInSingleplayer()&&!w.singleplayer)||(!c.isInSingleplayer()&&!w.multiplayer)))continue;
            if((c.hasReducedDebugInfo()||(TpClient.config.streamer&&TpClient.config.hideCoordinates))&&EnumSet.of(WidgetKind.COORDINATES,WidgetKind.BIOME,WidgetKind.DIMENSION,WidgetKind.DIRECTION,WidgetKind.LIGHT).contains(kind))continue;
            if(TpClient.config.streamer&&TpClient.config.hideNames&&kind==WidgetKind.TARGET)continue;
            List<String> lines=content.getOrDefault(kind,List.of("Esperando datos")).stream().map(line->c.textRenderer.trimToWidth(line,166)).toList();
            List<ItemStack> items=equipment.getOrDefault(kind,List.of()).stream().filter(stack->!stack.isEmpty()).toList();
            int width=Math.max(72,c.textRenderer.getWidth(kind.label)+16);for(String line:lines)width=Math.max(width,c.textRenderer.getWidth(line)+16);
            int height=lines.size()*(c.textRenderer.fontHeight+3)+25;
            if(kind==WidgetKind.ARMOR||kind==WidgetKind.HANDS){width=106;height=Math.max(40,items.size()*20+23);}
            if(kind==WidgetKind.KEYS){width=110;height=105;}
            measured.put(kind,new Measured(lines,items,width,height));
            requests.add(new HudLayout.Request(kind.id(),w.anchor,w.x,w.y,width,height,w.scale,w.locked||w.positioned||selected==kind));
        }
        int sw=c.getWindow().getScaledWidth(),sh=c.getWindow().getScaledHeight();
        var protectedAreas=HudLayout.safeAreas(sw,sh,c.currentScreen instanceof ChatScreen);
        if(edit){
            protectedAreas.add(new HudLayout.Rect(0,Math.max(0,sh-37),sw,37));
            protectedAreas.add(new HudLayout.Rect(0,Math.max(0,sh-63),Math.min(sw,304),25));
        }
        HudLayout.Result layout=HudLayout.arrange(requests,sw,sh,protectedAreas);hiddenWidgets=layout.hidden();
        for(WidgetKind kind:entries){
            var placement=layout.placements().get(kind.id());if(placement==null)continue;
            ClientConfig.Widget w=TpClient.config.widgets.get(kind.id());Measured measure=measured.get(kind);
            var lines=measure.lines;var items=measure.items;int width=measure.width,height=measure.height;
            int x=placement.bounds().x(),y=placement.bounds().y(),pixelWidth=placement.bounds().width(),pixelHeight=placement.bounds().height();
            bounds.put(kind,new Bounds(x,y,pixelWidth,pixelHeight));
            context.getMatrices().push();context.getMatrices().translate(x,y,0);context.getMatrices().scale((float)placement.scale(),(float)placement.scale(),1);
            try{
                int alpha=(int)(w.opacity*255),color=(alpha<<24)|0xEEEAF5;
                if(w.background&&!w.style.equals("plain")){context.fillGradient(0,0,width,height,((int)(w.opacity*232)<<24)|0x13101D,((int)(w.opacity*215)<<24)|0x090B11);context.drawBorder(0,0,width,height,((int)(w.opacity*95)<<24)|TpClient.config.accent);}
                if(w.style.equals("outline")||selected==kind)context.drawBorder(0,0,width,height,(alpha<<24)|TpClient.config.accent);
                context.fill(7,7,10,10,(alpha<<24)|TpClient.config.accent);
                context.drawText(c.textRenderer,kind.label,15,5,((int)(w.opacity*200)<<24)|0xC4B2DD,false);
                context.fill(7,17,width-7,18,((int)(w.opacity*36)<<24)|TpClient.config.accent);
                if(kind==WidgetKind.ARMOR||kind==WidgetKind.HANDS){int row=0;if(items.isEmpty())context.drawText(c.textRenderer,"Sin equipo",8,25,color,true);for(ItemStack item:items){int itemY=22+row*20;context.drawItem(item,7,itemY);String durability=item.isDamageable()?HudMath.percentage(item.getMaxDamage(),item.getDamage())+"%":Integer.toString(item.getCount());context.drawText(c.textRenderer,durability,31,itemY+3,color,true);if(item.isDamageable()){int remaining=HudMath.percentage(item.getMaxDamage(),item.getDamage());context.fill(31,itemY+14,94,itemY+16,0x552D263B);context.fill(31,itemY+14,31+remaining*63/100,itemY+16,(alpha<<24)|(remaining<15?0xFF7487:TpClient.config.accent));}row++;}}
                else if(kind==WidgetKind.KEYS)renderKeys(context,c,color);
                else{int row=0;for(String line:lines)context.drawText(c.textRenderer,line,8,24+row++*(c.textRenderer.fontHeight+3),color,true);}
                if(edit&&w.locked)context.drawText(c.textRenderer,"L",width-9,2,0xFFFFCA70,false);
            }catch(Exception error){if(failed.add(kind))TpClient.LOG.warn("Widget {} render isolated",kind.id());}finally{context.getMatrices().pop();}
        }
        if(edit&&hiddenWidgets>0){context.fill(6,sh-61,Math.min(sw-6,300),sh-40,0xE01C1028);context.drawText(c.textRenderer,"Sin espacio: "+hiddenWidgets+" · Reduce la escala GUI o los widgets",11,sh-55,0xFFFFC680,true);}
    }
    private List<String> lines(WidgetKind kind,MinecraftClient c){
        var p=c.player;var world=c.world;
        if(p==null||world==null)return List.of("Entra en un mundo");
        return switch(kind){
            case FPS -> List.of(c.getCurrentFps()+" FPS");
            case FRAMETIME -> TpClient.frames.count()<3?List.of("Midiendo cuadros…"):List.of(String.format(Locale.ROOT,"Media %.1f ms",TpClient.frames.average()),String.format(Locale.ROOT,"P95 %.1f ms",TpClient.frames.percentile(0.95)));
            case PING -> {var network=c.getNetworkHandler();var entry=network==null?null:network.getPlayerListEntry(p.getUuid());yield List.of(c.isInSingleplayer()?"MS: no aplica (local)":entry==null?"MS: no disponible":entry.getLatency()+" ms");}
            case COORDINATES -> List.of(String.format(Locale.ROOT,"X  %.1f",p.getX()),String.format(Locale.ROOT,"Y  %.1f",p.getY()),String.format(Locale.ROOT,"Z  %.1f",p.getZ()));
            case DIRECTION -> List.of(p.getHorizontalFacing().asString().toUpperCase(Locale.ROOT)+" · "+Math.round(MathHelper.wrapDegrees(p.getYaw()))+"°");
            case BIOME -> List.of(world.getBiome(p.getBlockPos()).getKey().map(key->key.getValue().getPath()).orElse("Bioma desconocido"));
            case DIMENSION -> List.of(world.getRegistryKey().getValue().toString());
            case WORLD_TIME -> {long time=Math.floorMod(world.getTimeOfDay()+6000,24000);yield List.of(String.format(Locale.ROOT,"Mundo %02d:%02d · Día %d",time/1000,(time%1000)*60/1000,world.getTimeOfDay()/24000));}
            case CLOCK -> List.of(LocalTime.now().format(DateTimeFormatter.ofPattern("HH:mm:ss")));
            case CPS -> List.of("LMB "+TpClient.cps(true)+" CPS   RMB "+TpClient.cps(false)+" CPS");
            case KEYS -> List.of(key(c.options.forwardKey)+"  "+key(c.options.leftKey)+"  "+key(c.options.backKey)+"  "+key(c.options.rightKey),key(c.options.jumpKey)+" · "+key(c.options.sprintKey)+" · "+key(c.options.sneakKey));
            case ARMOR,HANDS -> List.of();
            case EFFECTS -> {var list=p.getStatusEffects().stream().limit(12).map(effect->effect.getEffectType().value().getName().getString()+" "+(effect.getAmplifier()+1)+" · "+(effect.isInfinite()?"∞":effect.getDuration()/1200+":"+String.format(Locale.ROOT,"%02d",effect.getDuration()/20%60))).toList();yield list.isEmpty()?List.of("Sin efectos"):list;}
            case VITALS -> List.of(String.format(Locale.ROOT,"Vida %.1f / %.1f",p.getHealth(),p.getMaxHealth()),"Hambre "+p.getHungerManager().getFoodLevel()+" / 20");
            case SPEED -> List.of(String.format(Locale.ROOT,"%.2f bloques/s",Math.min(speed,10000)));
            case SESSION -> {long seconds=(System.nanoTime()-TpClient.started)/1_000_000_000L;yield List.of(String.format(Locale.ROOT,"Sesión %d:%02d:%02d",seconds/3600,seconds/60%60,seconds%60));}
            case MEMORY -> {Runtime r=Runtime.getRuntime();yield List.of("Heap "+(r.totalMemory()-r.freeMemory())/1048576+" / "+r.maxMemory()/1048576+" MiB");}
            case TARGET -> c.targetedEntity==null?List.of("Sin objetivo visible"):List.of(c.targetedEntity.getName().getString(),String.format(Locale.ROOT,"%.1f bloques",p.distanceTo(c.targetedEntity)));
            case LIGHT -> List.of("Luz bloque "+world.getLightLevel(LightType.BLOCK,p.getBlockPos())+" · cielo "+world.getLightLevel(LightType.SKY,p.getBlockPos()));
        };
    }
    private static void renderKeys(DrawContext context,MinecraftClient c,int color){
        keyBox(context,c,c.options.forwardKey,43,23,24,color);keyBox(context,c,c.options.leftKey,15,46,24,color);keyBox(context,c,c.options.backKey,43,46,24,color);keyBox(context,c,c.options.rightKey,71,46,24,color);
        keyBox(context,c,c.options.jumpKey,7,69,52,color);keyBox(context,c,c.options.sneakKey,63,69,40,color);
        context.drawText(c.textRenderer,c.options.sprintKey.isPressed()?"SPRINT ACTIVO":"SPRINT",8,93,c.options.sprintKey.isPressed()?0xFFD3AAFF:color,false);
    }
    private static void keyBox(DrawContext context,MinecraftClient c,KeyBinding binding,int x,int y,int width,int color){boolean down=binding.isPressed();context.fill(x,y,x+width,y+20,down?0xC064328B:0xA0201A2C);context.drawBorder(x,y,width,20,down?0xFFC889FF:0x805B456D);String label=c.textRenderer.trimToWidth(binding.getBoundKeyLocalizedText().getString(),width-4);context.drawText(c.textRenderer,label,x+(width-c.textRenderer.getWidth(label))/2,y+6,color,true);}
    private static String key(KeyBinding key){String value=key.getBoundKeyLocalizedText().getString();return key.isPressed()?"["+value+"]":value;}
}
