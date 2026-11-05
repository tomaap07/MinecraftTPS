package dev.tplauncher.client;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.tooltip.Tooltip;
import net.minecraft.client.gui.widget.TextFieldWidget;
import net.minecraft.text.Text;
import java.util.*;

public final class ClientScreen extends Screen {
    private static final List<String> TABS=List.of("HUD","PvP","Exploración","Técnico","Cosméticos","Notificaciones","Rendimiento","Ajustes");
    private final Screen parent;
    private String tab="HUD",query="",profileName="mi-perfil";
    private int scroll,detailScroll,bodyX,bodyY,bodyWidth,rightX,rightWidth,listWidth,bodyBottom;
    private boolean wide;
    private WidgetKind selected=WidgetKind.FPS;
    private TextFieldWidget search;
    private record Action(String label,Runnable run){}
    public ClientScreen(Screen parent){super(Text.literal("TP Client"));this.parent=parent;}
    public ClientScreen(Screen parent,String category){this(parent);if(TABS.contains(category))tab=category;}
    private TpButton button(String label,int x,int y,int w,int h,Runnable action){return addDrawableChild(new TpButton(Text.literal(label),x,y,Math.max(25,w),h,action));}
    private TpButton button(String label,int x,int y,int w,Runnable action){return button(label,x,y,w,22,action);}
    private void action(String label,Runnable run){run.run();TpClient.save();refresh();}
    @Override protected void init(){
        wide=width>=620;bodyX=wide?136:12;bodyY=wide?58:91;bodyWidth=width-bodyX-14;bodyBottom=height-54;
        if(wide){for(int i=0;i<TABS.size();i++){String category=TABS.get(i);button(category,12,57+i*27,108,24,()->selectTab(category)).selected(tab.equals(category));}}
        else{int tabWidth=(width-24)/4;for(int i=0;i<TABS.size();i++){String category=TABS.get(i);button(category,12+(i%4)*tabWidth,29+(i/4)*26,tabWidth-3,23,()->selectTab(category)).selected(tab.equals(category));}}
        button("Volver al juego",width-128,height-32,114,24,this::close);
        if(moduleTab())widgetControls();else settingsControls();
    }
    private boolean moduleTab(){return List.of("HUD","PvP","Exploración","Técnico","Rendimiento").contains(tab);}
    private void selectTab(String category){tab=category;scroll=0;detailScroll=0;search=null;refresh();}
    private boolean matches(WidgetKind kind){
        return switch(tab){
            case "PvP" -> EnumSet.of(WidgetKind.FPS,WidgetKind.PING,WidgetKind.CPS,WidgetKind.KEYS,WidgetKind.ARMOR,WidgetKind.HANDS,WidgetKind.EFFECTS,WidgetKind.VITALS,WidgetKind.TARGET).contains(kind);
            case "Exploración" -> EnumSet.of(WidgetKind.COORDINATES,WidgetKind.DIRECTION,WidgetKind.BIOME,WidgetKind.DIMENSION,WidgetKind.WORLD_TIME,WidgetKind.SPEED,WidgetKind.SESSION,WidgetKind.ARMOR).contains(kind);
            case "Técnico" -> EnumSet.of(WidgetKind.LIGHT,WidgetKind.COORDINATES,WidgetKind.DIRECTION,WidgetKind.WORLD_TIME,WidgetKind.MEMORY,WidgetKind.FRAMETIME,WidgetKind.FPS).contains(kind);
            case "Rendimiento" -> EnumSet.of(WidgetKind.FPS,WidgetKind.FRAMETIME,WidgetKind.MEMORY,WidgetKind.PING).contains(kind);
            default -> true;
        };
    }
    private void widgetControls(){
        rightWidth=wide?164:Math.max(125,bodyWidth/2-5);rightX=width-rightWidth-14;listWidth=rightX-bodyX-12;
        search=new TextFieldWidget(textRenderer,bodyX+7,bodyY+5,listWidth-14,18,Text.literal("Buscar módulo"));search.setDrawsBackground(false);search.setMaxLength(60);search.setText(query);search.setChangedListener(value->{query=value;scroll=0;refreshSearch();});addDrawableChild(search);
        List<WidgetKind> filtered=Arrays.stream(WidgetKind.values()).filter(k->matches(k)&&(k.label+" "+k.description).toLowerCase(Locale.ROOT).contains(query.toLowerCase(Locale.ROOT))).toList();
        if(!filtered.isEmpty()&&!filtered.contains(selected)){selected=filtered.getFirst();detailScroll=0;}
        int rowHeight=wide?51:29,rows=Math.max(1,(bodyBottom-bodyY-55)/rowHeight);scroll=Math.max(0,Math.min(scroll,Math.max(0,filtered.size()-rows)));
        for(int i=scroll;i<Math.min(filtered.size(),scroll+rows);i++){
            WidgetKind kind=filtered.get(i);ClientConfig.Widget widget=TpClient.config.widgets.get(kind.id());int y=bodyY+34+(i-scroll)*rowHeight;
            var select=button(kind.label,bodyX,y,listWidth-44,wide?46:25,()->{selected=kind;detailScroll=0;refresh();}).selected(kind==selected);select.setTooltip(Tooltip.of(Text.literal(kind.description)));
            button(widget.enabled?"ON":"OFF",bodyX+listWidth-40,y,40,wide?46:25,()->action("",()->widget.enabled=!widget.enabled)).selected(widget.enabled);
        }
        button("↑",bodyX,bodyBottom-19,30,19,()->{scroll--;refresh();});button("↓",bodyX+34,bodyBottom-19,30,19,()->{scroll++;refresh();});
        button("Todos",bodyX+70,bodyBottom-19,Math.max(40,(listWidth-74)/2),19,()->action("",()->filtered.forEach(k->TpClient.config.widgets.get(k.id()).enabled=true)));
        button("Ninguno",bodyX+73+(listWidth-74)/2,bodyBottom-19,Math.max(40,(listWidth-74)/2),19,()->action("",()->filtered.forEach(k->TpClient.config.widgets.get(k.id()).enabled=false)));
        detailControls();
        if(tab.equals("Rendimiento"))button("Métricas locales: "+(TpClient.config.metricsEnabled?"ON":"OFF"),bodyX,height-32,Math.max(100,width-bodyX-144),24,()->action("",()->TpClient.config.metricsEnabled=!TpClient.config.metricsEnabled));
        else button("Editar distribución",bodyX,height-32,Math.max(100,width-bodyX-144),24,()->{if(client.player!=null)client.setScreen(new HudEditorScreen(this));else TpClient.notices.add("HUD","Entra en un mundo para colocar los widgets");});
    }
    private void detailControls(){
        ClientConfig.Widget w=TpClient.config.widgets.get(selected.id());List<Action> actions=new ArrayList<>();
        actions.add(new Action(String.format(Locale.ROOT,"Tamaño: %.0f%%  +",w.scale*100),()->w.scale=HudMath.finite(w.scale+0.1,1,0.5,3)));
        actions.add(new Action("Reducir tamaño −",()->w.scale=HudMath.finite(w.scale-0.1,1,0.5,3)));
        actions.add(new Action(String.format(Locale.ROOT,"Opacidad: %.0f%%  +",w.opacity*100),()->w.opacity=HudMath.finite(w.opacity+0.1,0.9,0.1,1)));
        actions.add(new Action("Reducir opacidad −",()->w.opacity=HudMath.finite(w.opacity-0.1,0.9,0.1,1)));
        actions.add(new Action("Ancla: "+anchorName(w.anchor),()->{w.anchor=ClientConfig.ANCHORS.get((ClientConfig.ANCHORS.indexOf(w.anchor)+1)%ClientConfig.ANCHORS.size());w.x=0;w.y=0;w.positioned=false;}));
        actions.add(new Action("Estilo: "+w.style,()->{List<String> styles=List.of("panel","plain","outline");w.style=styles.get((styles.indexOf(w.style)+1)%styles.size());}));
        actions.add(new Action(w.locked?"Desbloquear posición":"Bloquear posición",()->w.locked=!w.locked));
        actions.add(new Action("Individual: "+(w.singleplayer?"ON":"OFF"),()->w.singleplayer=!w.singleplayer));
        actions.add(new Action("Multijugador: "+(w.multiplayer?"ON":"OFF"),()->w.multiplayer=!w.multiplayer));
        actions.add(new Action("Prioridad: "+w.order+" +",()->w.order=(w.order+1)%101));
        actions.add(new Action("Restaurar este módulo",()->TpClient.config.widgets.put(selected.id(),new ClientConfig().widgets.get(selected.id()))));
        int top=bodyY+(wide?74:50),rows=Math.max(1,(bodyBottom-top-23)/24);detailScroll=Math.max(0,Math.min(detailScroll,Math.max(0,actions.size()-rows)));
        for(int i=detailScroll;i<Math.min(actions.size(),detailScroll+rows);i++){Action a=actions.get(i);button(a.label,rightX+8,top+(i-detailScroll)*24,rightWidth-16,21,()->action(a.label,a.run));}
        button("↑ Ajustes",rightX+8,bodyBottom-19,(rightWidth-20)/2,19,()->{detailScroll--;refresh();});button("↓",rightX+12+(rightWidth-20)/2,bodyBottom-19,(rightWidth-20)/2,19,()->{detailScroll++;refresh();});
    }
    private void settingsControls(){
        List<Action> actions=new ArrayList<>();int top=bodyY+7;
        switch(tab){
            case "Ajustes"->{
                TextFieldWidget name=new TextFieldWidget(textRenderer,bodyX+8,top,bodyWidth-16,18,Text.literal("Nombre del perfil"));name.setMaxLength(48);name.setDrawsBackground(false);name.setText(profileName);name.setChangedListener(value->profileName=value);addDrawableChild(name);top+=29;
                actions.add(new Action("HUD: "+(TpClient.config.hudEnabled?"activado":"desactivado"),()->TpClient.config.hudEnabled=!TpClient.config.hudEnabled));
                actions.add(new Action("Modo streamer: "+(TpClient.config.streamer?"ON":"OFF"),()->TpClient.config.streamer=!TpClient.config.streamer));
                actions.add(new Action("Zoom con C: "+(TpClient.config.zoomEnabled?"ON":"OFF")+(TpClient.zoomConflict()?" · otro mod detectado":""),()->TpClient.config.zoomEnabled=!TpClient.config.zoomEnabled));
                for(String preset:List.of("Supervivencia","PvP","Exploración","Técnico","Minimalista","Streaming"))actions.add(new Action("Aplicar perfil "+preset,()->TpClient.config.preset(preset)));
                actions.add(new Action("Cuadrícula: "+TpClient.config.grid+" px",()->TpClient.config.grid=TpClient.config.grid==4?8:TpClient.config.grid==8?1:4));
                actions.add(new Action("Color de acento",()->TpClient.config.accent=TpClient.config.accent==0xB678F5?0x62D7D2:TpClient.config.accent==0x62D7D2?0xE6B15D:0xB678F5));
                actions.add(new Action("Guardar perfil personalizado",()->{TpClient.config.profile=profileName;TpClient.exportProfile(profileName);}));
                actions.add(new Action("Importar config/tp-client-import.json",TpClient::importProfile));
                actions.add(new Action("Exportar perfil: "+profileName,()->TpClient.exportProfile(profileName)));
                actions.add(new Action("Ocultar coordenadas al transmitir: "+(TpClient.config.hideCoordinates?"ON":"OFF"),()->TpClient.config.hideCoordinates=!TpClient.config.hideCoordinates));
                actions.add(new Action("Ocultar nombres al transmitir: "+(TpClient.config.hideNames?"ON":"OFF"),()->TpClient.config.hideNames=!TpClient.config.hideNames));
                actions.add(new Action("Restaurar distribución segura",()->TpClient.config.preset(TpClient.config.profile)));
            }
            case "Cosméticos"->{
                actions.add(new Action("Cosméticos TP locales: "+(TpClient.config.cosmeticsEnabled?"ON":"OFF"),()->TpClient.config.cosmeticsEnabled=!TpClient.config.cosmeticsEnabled));
                actions.add(new Action("Capa TP: "+Cosmetics.capeLabel(),Cosmetics::cycleCape));
                actions.add(new Action("Alas Serafín Celestial: "+(Cosmetics.wings()?"ON":"OFF"),Cosmetics::toggleWings));
                actions.add(new Action("Corona Eclipse Elemental: "+(Cosmetics.crown()?"ON":"OFF"),Cosmetics::toggleCrown));
                actions.add(new Action("Abrir carpeta de capas PNG propias",Cosmetics::openFolder));
            }
            case "Notificaciones"->{
                actions.add(new Action("Avisos: "+(TpClient.config.notifications?"ON":"OFF"),()->TpClient.config.notifications=!TpClient.config.notifications));
                actions.add(new Action("Silenciar al transmitir: "+(TpClient.config.hideNotifications?"ON":"OFF"),()->TpClient.config.hideNotifications=!TpClient.config.hideNotifications));
                actions.add(new Action("Guardar captura (F9)",()->TpClient.capture(client)));
                actions.add(new Action("Mostrar aviso de prueba",()->TpClient.notices.add("TP Client","Los avisos se silencian al recibir daño")));
            }
        }
        int rows=Math.max(1,(bodyBottom-top-25)/29);scroll=Math.max(0,Math.min(scroll,Math.max(0,actions.size()-rows)));
        for(int i=scroll;i<Math.min(actions.size(),scroll+rows);i++){Action a=actions.get(i);button(a.label,bodyX+7,top+(i-scroll)*29,bodyWidth-14,25,()->action(a.label,a.run));}
        button("↑",bodyX+7,bodyBottom-20,40,20,()->{scroll--;refresh();});button("↓",bodyX+52,bodyBottom-20,40,20,()->{scroll++;refresh();});
    }
    private static String anchorName(String anchor){return switch(anchor){case "top_right"->"arriba derecha";case "bottom_left"->"abajo izquierda";case "bottom_right"->"abajo derecha";case "top_center"->"arriba centro";case "bottom_center"->"abajo centro";default->"arriba izquierda";};}
    private void refresh(){clearChildren();init();}
    private void refreshSearch(){refresh();if(search!=null){setFocused(search);search.setFocused(true);search.setCursorToEnd(false);}}
    @Override public boolean mouseScrolled(double x,double y,double horizontal,double vertical){if(moduleTab()&&x>=rightX)detailScroll-=(int)Math.signum(vertical);else scroll-=(int)Math.signum(vertical);refresh();return true;}
    @Override public void render(DrawContext c,int mouseX,int mouseY,float delta){
        c.fillGradient(0,0,width,height,0xF50D0B15,0xF5060810);c.fill(0,0,3,height,0xFFA855F7);
        if(wide){c.fill(5,0,126,height,0xE60B0C12);c.fill(125,0,126,height,0x7044305C);c.drawText(textRenderer,"TP CLIENT",16,17,0xFFEAD9FF,true);c.drawText(textRenderer,"TU PARTIDA, TU HUD",16,33,0xFF82738E,false);}
        else{c.drawText(textRenderer,"TP CLIENT",12,10,0xFFEAD9FF,true);c.drawText(textRenderer,"1.21.1 · Fabric",Math.max(12,width-107),10,0xFF8D809D,false);}
        if(wide){c.drawText(textRenderer,tab,bodyX,17,0xFFF0E7FA,true);c.drawText(textRenderer,"Métricas del juego · Perfil "+TpClient.config.profile,bodyX,34,0xFF9D8DB0,false);}
        if(moduleTab()){
            c.fill(bodyX,bodyY,bodyX+listWidth,bodyY+27,0xFF14121E);c.drawBorder(bodyX,bodyY,listWidth,27,0x705F4677);
            if(query.isEmpty()&&!search.isFocused())c.drawText(textRenderer,"Buscar un módulo…",bodyX+8,bodyY+9,0xFF776A85,false);
            c.fill(rightX,bodyY,rightX+rightWidth,bodyBottom+1,0xD511101B);c.drawBorder(rightX,bodyY,rightWidth,bodyBottom-bodyY+1,0x605D3D74);
            c.drawText(textRenderer,textRenderer.trimToWidth(selected.label,rightWidth-16),rightX+8,bodyY+9,0xFFEAD9FF,true);
            int descriptionY=bodyY+24;for(var line:textRenderer.wrapLines(Text.literal(selected.description),rightWidth-16).stream().limit(wide?3:2).toList()){c.drawText(textRenderer,line,rightX+8,descriptionY,0xFF9E8BAD,false);descriptionY+=11;}
        }else{
            c.fill(bodyX,bodyY,bodyX+bodyWidth,bodyBottom+1,0xA512111B);c.drawBorder(bodyX,bodyY,bodyWidth,bodyBottom-bodyY+1,0x604E345E);
        }
        if(wide)c.drawText(textRenderer,"F8 · Editor",16,height-27,0xFF8D7C9E,false);
        if(tab.equals("Cosméticos"))c.drawText(textRenderer,textRenderer.trimToWidth("Locales y originales. No cambian las capas oficiales.",bodyWidth-134),bodyX,height-24,0xFFAD9CBA,false);
        if(tab.equals("Notificaciones"))c.drawText(textRenderer,textRenderer.trimToWidth("Amigos y clips: sin servicio conectado.",bodyWidth-134),bodyX,height-24,0xFFAD9CBA,false);
        super.render(c,mouseX,mouseY,delta);
    }
    @Override public void close(){TpClient.save();client.setScreen(parent);}
    @Override public void renderBackground(DrawContext c,int mouseX,int mouseY,float delta){}
    @Override public boolean shouldPause(){return false;}
}
