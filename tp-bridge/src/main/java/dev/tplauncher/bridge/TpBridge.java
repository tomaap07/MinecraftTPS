package dev.tplauncher.bridge;

import java.io.*;
import java.lang.instrument.Instrumentation;
import java.lang.management.ManagementFactory;
import java.lang.reflect.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import java.util.regex.Pattern;
import java.net.*;



public final class TpBridge implements Runnable {
    private static final int LIMIT = 65536;
    private final Instrumentation instrumentation;
    private final Properties bindings;
    private final Path profile, output, pending;
    private final long pid;
    private final Map<String, AccessibleObject> members = Collections.synchronizedMap(new HashMap<String, AccessibleObject>());
    private Class<?> clientClass;
    private Field instanceField;
    private Method instanceGetter;
    private String namespace = "named";
    private long lastDiscovery, lastPreference;
    private long lastFocusSetting;
    private volatile boolean fullscreenBorderless;
    private String lastCommand="";
    private String fogProfile="";
    private Object saveServer;private long lastSaveCheck;private Long savedDay;
    private boolean enabled = true;
    private volatile boolean stopping;
    private final java.util.concurrent.atomic.AtomicBoolean zoomQueued=new java.util.concurrent.atomic.AtomicBoolean();
    private volatile boolean zoomAvailable;
    private Object zoomTarget;private Method zoomSetter;private Field zoomField;private Number zoomOriginal;
    private boolean zoomDelegate;private long lastZoomDelegateCheck;
    private long lastPublish;
    private DatagramSocket transport;private int port;private String transportToken;
    private static final Pattern ACTIVE = Pattern.compile("\"native_overlay\"\\s*:\\s*true");

    public static void premain(String options, Instrumentation instrumentation) {

        try {
            TpBridge bridge = new TpBridge(Paths.get(options), instrumentation);
            try{RenderHook.install(instrumentation,bridge.profile.getParent().resolve("bridge"));}catch(Throwable ignored){System.err.println("[TP HUD] Contador LWJGL no disponible; se usará el contador del cliente.");}
            Thread worker = new Thread(bridge, "TP HUD local telemetry");
            worker.setDaemon(true);
            Runtime.getRuntime().addShutdownHook(new Thread(new Runnable() {
                public void run() { bridge.stopping = true; bridge.removeFiles(); }
            }, "TP HUD cleanup"));
            worker.start();
        } catch (Throwable ignored) { System.err.println("[TP HUD] Puente local no disponible; Minecraft puede continuar."); }
    }

    private TpBridge(Path config, Instrumentation instrumentation) throws Exception {
        this.instrumentation = instrumentation;
        this.bindings = new Properties();
        if (Files.size(config) > LIMIT) throw new IOException("Bindings too large");
        try (Reader input = Files.newBufferedReader(config, StandardCharsets.UTF_8)) { bindings.load(input); }
        profile = Paths.get(required("profile")).toAbsolutePath().normalize();
        pid = Long.parseLong(ManagementFactory.getRuntimeMXBean().getName().split("@", 2)[0]);
        if (pid < 1 || profile.getParent() == null) throw new IOException("Invalid process/profile");
        output = profile.getParent().resolve("bridge-" + pid + ".json");
        pending = output.resolveSibling(output.getFileName() + ".pending");
        try{port=Integer.parseInt(bindings.getProperty("udp.port","0"));transportToken=bindings.getProperty("udp.token","");if(port>0&&port<=65535&&transportToken.matches("[0-9a-f]{64}"))transport=new DatagramSocket();}catch(Exception ignored){}
    }

    private String required(String key) throws IOException {
        String value = bindings.getProperty(key, "").trim();
        if (value.length() == 0) throw new IOException("Missing binding");
        return value;
    }

    public void run() {
        while (!stopping) {
            try {
                long now = System.currentTimeMillis();
                if (now - lastPreference > 1000) {
                    lastPreference = now;
                    fogProfile=Files.isRegularFile(profile)&&Files.size(profile)<=LIMIT?new String(Files.readAllBytes(profile),StandardCharsets.UTF_8):"";
                    enabled=ACTIVE.matcher(fogProfile).find();
                }
                Object client = client(now);
                if(client!=null)pollZoom(client,enabled);
                if (!enabled) { RenderHook.fog(false,0);removeFiles(); Thread.sleep(100); continue; }
                if(client!=null)pollCommand(client);
                if(now-lastPublish>=200){lastPublish=now;publish(client == null ? waiting() : sample(client));}
                Thread.sleep(50);
            } catch (InterruptedException interrupted) { break; }
            catch (Throwable ignored) {
                try { publish(waiting()); Thread.sleep(1500); } catch (Throwable unavailable) {   }
            }
        }
        removeFiles();
    }

    private Object client(long now) throws Exception {
        if (instanceField != null) return instanceField.get(null);
        if (instanceGetter != null) return instanceGetter.invoke(null);
        if (now - lastDiscovery < 2000) return null;
        lastDiscovery = now;
        Set<String> candidates = new HashSet<String>(Arrays.asList(bindings.getProperty("client.classes",
            "net.minecraft.client.Minecraft,net.minecraft.client.MinecraftClient,net.minecraft.class_310").split(",")));


        Set<String> executing = new HashSet<String>(), initializing = new HashSet<String>();
        for (StackTraceElement[] stack : Thread.getAllStackTraces().values()) {
            for (StackTraceElement item : stack) if (candidates.contains(item.getClassName())) {
                if (item.getMethodName().equals("<clinit>")) initializing.add(item.getClassName());
                else if (!item.getMethodName().equals("<init>")) executing.add(item.getClassName());
            }
        }
        for (Class<?> type : instrumentation.getAllLoadedClasses()) {
            String name = type.getName();
            if (!candidates.contains(name) || !executing.contains(name) || initializing.contains(name)) continue;
            Field singleton = null;
            for (Field field : type.getDeclaredFields()) if (Modifier.isStatic(field.getModifiers()) && field.getType() == type) {
                if (singleton != null) { singleton = null; break; }
                singleton = field;
            }
            namespace = name.equals(bindings.getProperty("class.official")) ? "official"
                : name.equals(bindings.getProperty("class.fabric", "net.minecraft.class_310")) ? "fabric" : "named";
            if (singleton != null) try {
                singleton.setAccessible(true);
                Object value=singleton.get(null);
                if(value!=null){clientClass=type;instanceField=singleton;return value;}
            }catch(ReflectiveOperationException ignored){}catch(RuntimeException inaccessible){}


            for(String getter:names("client.instance","getInstance,getMinecraft"))try{
                Method method=type.getMethod(getter);
                if(method.getReturnType()!=type||!Modifier.isStatic(method.getModifiers()))continue;
                Object value=method.invoke(null);
                if(value!=null){clientClass=type;instanceGetter=method;return value;}
            }catch(ReflectiveOperationException ignored){}catch(RuntimeException inaccessible){}
        }
        return null;
    }

    private String[] names(String role, String defaults) {
        String resolved = bindings.getProperty(namespace + "." + role, "");
        return (resolved.length() == 0 ? defaults : resolved + "," + defaults).split(",");
    }

    private AccessibleObject member(Class<?> type, String role, boolean method, String defaults, Class<?>... parameters) {
        String key = type.getName() + ":" + role + ":" + method + ":" + Arrays.toString(parameters);
        if (members.containsKey(key)) return members.get(key);
        for (String name : names(role, defaults)) {
            if (name.length() == 0) continue;
            for (Class<?> owner = type; owner != null; owner = owner.getSuperclass()) {
                try {
                    AccessibleObject value = method ? owner.getDeclaredMethod(name, parameters) : owner.getDeclaredField(name);
                    value.setAccessible(true); members.put(key, value); return value;
                } catch (ReflectiveOperationException missing) {   }
                catch (RuntimeException inaccessible) {   }
            }
            if (method) try {
                Method value = type.getMethod(name, parameters); value.setAccessible(true); members.put(key, value); return value;
            } catch (ReflectiveOperationException missing) {   }
              catch (RuntimeException inaccessible) {   }
        }
        members.put(key, null); return null;
    }

    private Object field(Object target, String role, String defaults) {
        if (target == null) return null;
        try {
            Field field = (Field) member(target.getClass(), role, false, defaults);
            return field == null ? null : field.get(Modifier.isStatic(field.getModifiers()) ? null : target);
        } catch (Throwable missing) { return null; }
    }

    private Object invoke(Object target, String role, String defaults, Class<?>[] types, Object... arguments) {
        if (target == null) return null;
        try {
            Method method = (Method) member(target.getClass(), role, true, defaults, types);
            return method == null ? null : method.invoke(target, arguments);
        } catch (Throwable missing) { return null; }
    }
    private Object call(Object target, String role, String defaults) { return invoke(target, role, defaults, new Class<?>[0]); }
    private Object callOne(Object target,String role,String defaults,Object argument){
        if(target==null||argument==null)return null;
        String key=target.getClass().getName()+":one:"+role+":"+argument.getClass().getName();
        if(!members.containsKey(key)){
            Method match=null;search:for(String name:names(role,defaults))for(Method method:target.getClass().getMethods())if(method.getName().equals(name)&&method.getParameterTypes().length==1&&method.getParameterTypes()[0].isInstance(argument)){try{method.setAccessible(true);match=method;break search;}catch(RuntimeException ignored){}}
            members.put(key,match);
        }
        try{Method method=(Method)members.get(key);return method==null?null:method.invoke(target,argument);}catch(Throwable ignored){return null;}
    }
    private String registryKey(Object value){
        Object key=call(value,"holder.key","unwrapKey,getKey");if(key instanceof Optional)key=((Optional<?>)key).orElse(null);if(key==null)key=value;
        Object location=call(key,"key.location","location,identifier,getValue");String result=location==null?"":String.valueOf(location);
        return result.matches("[a-z0-9_.-]+:[a-z0-9_./-]{1,100}")?result:"";
    }
    private Double number(Object value) {
        if (!(value instanceof Number)) return null;
        double n = ((Number) value).doubleValue(); return Double.isNaN(n) || Double.isInfinite(n) ? null : n;
    }
    private Double coordinate(Object player, String axis) {
        Double value = number(call(player, "entity." + axis, "get" + axis.toUpperCase(Locale.ROOT)));
        if (value == null) value = number(field(player, "entity.pos_" + axis, "pos" + axis.toUpperCase(Locale.ROOT)));
        return value != null && Math.abs(value) < 100000000 ? value : null;
    }

    private Double fps(Object client) {
        Double value = number(field(client, "client.fps", "fps,debugFPS,currentFps"));
        if (value == null) value = number(call(client, "client.fps_method", "getFps,getCurrentFps,getDebugFPS"));
        if (value == null) {
            Object debug = field(client, "client.debug", "debug,debugString");
            if (debug instanceof String) {
                java.util.regex.Matcher match = Pattern.compile("^(\\d{1,6})\\s+fps\\b").matcher((String)debug);
                if (match.find()) value = Double.valueOf(match.group(1));
            }
        }
        return value != null && value >= 0 && value <= 100000 ? value : null;
    }

    private Object typedField(Object client, String role, String defaults, String... suffixes) {
        Object value = field(client, role, defaults);
        if (value != null) return value;
        Field match = null;
        for (Field candidate : clientClass.getDeclaredFields()) {
            if (Modifier.isStatic(candidate.getModifiers())) continue;
            for (String suffix : suffixes) if (candidate.getType().getName().endsWith(suffix)) {
                if (match != null) return null; match = candidate; break;
            }
        }
        if (match != null) try { match.setAccessible(true); return match.get(client); } catch (Throwable ignored) { }
        return null;
    }

    private String armor(Object player, Set<String> capabilities) {
        Object inventory = call(player, "player.inventory_method", "getInventory");
        if (inventory == null) inventory = field(player, "player.inventory", "inventory,field_71071_by");
        Object slots = equippedArmor(player);boolean namedSlots=slots!=null;
        if(slots==null)slots = field(inventory, "inventory.armor", "armor,armorInventory,field_70460_b");
        if (slots == null) slots = call(player, "player.armor", "getArmorSlots,getArmorItems");
        Iterable<?> entries;
        if (slots instanceof Iterable) entries = (Iterable<?>) slots;
        else if (slots instanceof Object[]) entries = Arrays.asList((Object[]) slots);
        else return "[]";
        List<String> rows = new ArrayList<String>(); int count = 0;
        for(Object stack:entries){if(++count>4)return "[]";String slot=(namedSlots?new String[]{"head","chest","legs","feet"}:new String[]{"feet","legs","chest","head"})[count-1];String row=equipmentRow(stack,slot);if(row!=null)rows.add(row);}
        if(rows.size()>4)return "[]";
        for(String[] hand:new String[][]{{"mainhand","getMainHandItem,getMainHandStack,getHeldItem"},{"offhand","getOffhandItem,getOffHandStack"}}){String row=equipmentRow(call(player,"player."+hand[0],hand[1]),hand[0]);if(row!=null)rows.add(row);}
        capabilities.add("armor");return "["+join(rows)+"]";
    }
    private String equipmentRow(Object stack,String slot){
        if(stack==null||Boolean.TRUE.equals(call(stack,"stack.empty","isEmpty,func_190926_b")))return null;
        Double max=number(call(stack,"stack.max","getMaxDamage,func_77958_k")),used=number(call(stack,"stack.damage","getDamageValue,getDamage,getItemDamage,func_77952_i"));
        if(max==null||used==null||max<0||max>Integer.MAX_VALUE)return null;
        Object component=call(stack,"stack.name","getHoverName,getName,getDisplayName,func_82833_r");Object label=component instanceof String?component:call(component,"component.text","getString,asString");String name=label instanceof String?(String)label:"Equipamiento";
        if(name.codePointCount(0,name.length())>48)name=name.substring(0,name.offsetByCodePoints(0,48));
        String value=itemId(stack),id=value.matches("[a-z0-9_]+")?"minecraft:"+value:"";
        return "{\"name\":"+quote(name)+",\"item_id\":"+quote(id)+",\"slot\":"+quote(slot)+",\"maximum\":"+max.intValue()+",\"remaining\":"+Math.max(0,Math.min(max.intValue(),max.intValue()-used.intValue()))+"}";
    }



    private Object equippedArmor(Object player) {
        String key=player.getClass().getName()+":equipmentGetter";
        if (!members.containsKey(key)) {
            Method match=null;
            search: for(String name:names("player.equipment_slot","getItemBySlot,getEquippedStack")) {
                for(Class<?> owner=player.getClass();owner!=null;owner=owner.getSuperclass()) {
                    for(Method method:owner.getDeclaredMethods()) if(method.getName().equals(name)
                        && method.getParameterTypes().length==1 && method.getParameterTypes()[0].isEnum()) {
                        try{method.setAccessible(true);match=method;break search;}catch(RuntimeException ignored){}
                    }
                }
            }
            members.put(key,match);
        }
        Method getter=(Method)members.get(key);if(getter==null)return null;
        try {
            Class<?> type=getter.getParameterTypes()[0];Object[] constants=type.getEnumConstants();
            if(constants==null||constants.length==0)return null;
            List<Object> result=new ArrayList<Object>();
            for(String slot:new String[]{"head","chest","legs","feet"}) {
                Object value=field(constants[0],"slot."+slot,slot.toUpperCase(Locale.ROOT));
                if(value==null) for(Object item:constants) if(((Enum<?>)item).name().equalsIgnoreCase(slot)){value=item;break;}
                if(value==null)return null;
                result.add(getter.invoke(player,value));
            }
            return result;
        }catch(Throwable unavailable){return null;}
    }

    private String sample(Object client) {
        if(System.currentTimeMillis()-lastFocusSetting>1000){lastFocusSetting=System.currentTimeMillis();configureFocus(client);}
        Set<String> capabilities = new LinkedHashSet<String>();
        Double fps = fps(client); double[] render=RenderHook.metrics();if(fps==null&&render!=null)fps=render[0];if (fps != null) capabilities.add("fps");
        if(zoomAvailable)capabilities.add("optical_zoom");
        Object player = typedField(client, "client.player", "player,thePlayer,field_71439_g", ".LocalPlayer", ".EntityPlayerSP", ".ClientPlayerEntity");
        Object world = typedField(client, "client.world", "level,world,theWorld,field_71441_e", ".ClientLevel", ".WorldClient", ".ClientWorld");
        boolean hasWorld = player != null && world != null;
        boolean singleplayer=hasWorld&&Boolean.TRUE.equals(call(client,"client.local","isLocalServer,isInSingleplayer,isSingleplayer,func_71356_B"));
        Double worldTime=hasWorld?number(call(world,"world.day_time","getDayTime,getDefaultClockTime,getTimeOfDay,getWorldTime,func_72820_D")):null;
        String timeSource="live";
        if(worldTime==null&&singleplayer){Long saved=savedWorldTime(client);if(saved!=null){worldTime=saved.doubleValue();timeSource="level.dat · último guardado";}}
        if(worldTime!=null&&worldTime>=0)capabilities.add("days");
        Double cooldown=hasWorld?number(invoke(player,"player.cooldown","getAttackStrengthScale,getAttackCooldownProgress",new Class<?>[]{float.class},0f)):null;
        Double targetDistance=null;Object target=hasWorld?field(client,"client.target","crosshairPickEntity,targetedEntity"):null;
        if(target!=null){Object hit=field(client,"client.hit","hitResult,crosshairTarget"),point=call(hit,"hit.location","getLocation,getPos"),eye=call(player,"entity.eye","getEyePosition,getEyePos");Double tx=number(field(point,"vec.x","x")),ty=number(field(point,"vec.y","y")),tz=number(field(point,"vec.z","z")),px=number(field(eye,"vec.x","x")),py=number(field(eye,"vec.y","y")),pz=number(field(eye,"vec.z","z"));if(tx!=null&&ty!=null&&tz!=null&&px!=null&&py!=null&&pz!=null)targetDistance=Math.sqrt(Math.pow(tx-px,2)+Math.pow(ty-py,2)+Math.pow(tz-pz,2));}
        Double gameTime=hasWorld?number(call(world,"world.game_time","getGameTime,getTime,getTotalWorldTime,func_82737_E")):null;if(gameTime==null&&hasWorld)gameTime=number(call(call(world,"world.level_data","getLevelData,getLevelProperties"),"level.game_time","getGameTime,getTime"));
        boolean restricted = hasWorld && (Boolean.TRUE.equals(call(client,"client.reduced","showOnlyReducedInfo,hasReducedDebugInfo"))
            || Boolean.TRUE.equals(call(player, "player.reduced", "isReducedDebugInfo,hasReducedDebugInfo,func_175140_cp")));
        Double x = hasWorld && !restricted ? coordinate(player, "x") : null;
        Double y = hasWorld && !restricted ? coordinate(player, "y") : null;
        Double z = hasWorld && !restricted ? coordinate(player, "z") : null;
        boolean position = x != null && y != null && z != null;
        String biome="",dimension="";
        if(hasWorld){dimension=registryKey(call(world,"world.dimension","dimension,getRegistryKey"));if(!restricted){Object pos=call(player,"entity.block_pos","blockPosition,getBlockPos,getPosition");biome=registryKey(callOne(world,"world.biome","getBiome",pos));if(!biome.isEmpty())capabilities.add("biome");}}
        if (position) capabilities.add("coordinates");
        Double yaw = hasWorld && !restricted ? number(call(player, "entity.yaw", "getYRot,getYaw")) : null;
        if (yaw == null && hasWorld && !restricted) yaw = number(field(player, "entity.yaw_field", "rotationYaw"));
        String direction = yaw == null ? "" : new String[]{"south","west","north","east"}[((int)Math.floor(yaw / 90 + .5)) & 3];
        if (yaw != null) capabilities.add("direction");
        configureFog(player,dimension,hasWorld);
        if(RenderHook.fogAvailable())capabilities.add("no_fog");
        String armor = hasWorld ? armor(player, capabilities) : "[]";
        Double ping = null;
        if (hasWorld) {
            Object connection = call(client, "client.connection", "getConnection,getNetworkHandler,getNetHandler");
            Object uuid = call(player, "entity.uuid", "getUUID,getUuid,getUniqueID");
            if (uuid instanceof UUID) {
                Object info = invoke(connection, "connection.player_info", "getPlayerInfo,getPlayerListEntry", new Class<?>[]{UUID.class}, uuid);
                ping = number(call(info, "info.ping", "getLatency,getResponseTime"));
                if (ping != null && ping >= 0 && ping <= 600000) capabilities.add("ping"); else ping = null;
            }
        }
        if(singleplayer){ping=0.0;capabilities.add("ping");}
        Object server=call(client,"client.server","getCurrentServer,getCurrentServerEntry");Object address=field(server,"server.address","ip,address,serverIP");String serverAddress=singleplayer||!(address instanceof String)?"":(String)address;
        String status = restricted ? "El servidor ha reducido la información de depuración; se respeta esa restricción."
            : !hasWorld ? "Entra en un mundo para recibir posición y equipamiento."
            : !position ? "Esta edición no expone todavía una posición compatible con el puente."
            : "Datos internos conectados con Minecraft.";
        List<String> caps = new ArrayList<String>(); for (String cap : capabilities) caps.add(quote(cap));
        return base() + ",\"has_world\":" + hasWorld + ",\"game_fps\":" + fps
            + ",\"x\":" + (position ? x : 0) + ",\"y\":" + (position ? y : 0) + ",\"z\":" + (position ? z : 0)
            + ",\"direction\":" + quote(direction) + ",\"ping\":" + ping + ",\"armor\":" + armor
            + ",\"world_time_source\":"+quote(timeSource)+",\"singleplayer\":"+singleplayer+",\"world_time\":"+worldTime+",\"game_time\":"+gameTime+",\"biome\":"+quote(biome)+",\"dimension\":"+quote(dimension)+",\"server_address\":"+quote(serverAddress)
            + ",\"attack_cooldown\":"+cooldown+",\"target_distance\":"+targetDistance
            + ",\"labels\":"+screenLabels(client)+",\"world_metrics\":"+(hasWorld?playerMetrics(player,world,client,restricted):"{}")
            + ",\"capabilities\":[" + join(caps) + "],\"coordinates_restricted\":" + restricted
            + ",\"status\":" + quote(status) + "}";
    }

    private Long savedWorldTime(Object client){
        long now=System.currentTimeMillis();
        try{Object server=call(client,"client.integrated","getSingleplayerServer,getServer,getIntegratedServer,func_71401_C");if(server==null)return null;if(server==saveServer&&now-lastSaveCheck<5000)return savedDay;saveServer=server;lastSaveCheck=now;savedDay=null;Path path=null;
            outer:for(String name:names("server.world_path","getWorldPath,getSavePath"))for(Method method:server.getClass().getMethods())if(method.getName().equals(name)&&method.getParameterTypes().length==1&&Path.class.isAssignableFrom(method.getReturnType())){
                Field root=(Field)member(method.getParameterTypes()[0],"storage.root",false,"ROOT");if(root==null||!Modifier.isStatic(root.getModifiers()))continue;path=(Path)method.invoke(server,root.get(null));break outer;
            }
            if(path==null){Object name=call(server,"server.folder","getFolderName");if(name instanceof String&&((String)name).matches("[^/\\\\]{1,128}")){Path saves=Paths.get(System.getProperty("user.dir")).resolve("saves").toRealPath(),candidate=saves.resolve((String)name).toRealPath();if(candidate.startsWith(saves))path=candidate;}}
            if(path!=null)savedDay=WorldTime.read(path);
        }catch(Throwable ignored){}return savedDay;
    }

    private void configureFog(Object player,String dimension,boolean world){
        java.util.regex.Matcher section=Pattern.compile("\"fog\"\\s*:\\s*\\{([^}]{0,1024})\\}").matcher(fogProfile);boolean active=false;float strength=1;
        if(world&&section.find()){String json=section.group(1);String environment=Boolean.TRUE.equals(call(player,"entity.lava","isInLava"))?"lava":Boolean.TRUE.equals(call(player,"entity.water","isInWater,isTouchingWater"))?"water":dimension.endsWith(":the_nether")?"nether":dimension.endsWith(":the_end")?"end":dimension.endsWith(":overworld")?"overworld":"";
            active=flag(json,"enabled")&&!environment.isEmpty()&&flag(json,environment);java.util.regex.Matcher value=Pattern.compile("\"strength\"\\s*:\\s*([0-9.]+)").matcher(json);if(value.find())try{strength=Float.parseFloat(value.group(1));}catch(NumberFormatException ignored){}
        }RenderHook.fog(active,strength);
    }
    private static boolean flag(String json,String key){return Pattern.compile("\""+key+"\"\\s*:\\s*true").matcher(json).find();}

    private Object screen(Object client){Object screen=field(client,"client.screen","screen,currentScreen,field_71462_r");if(screen==null){Object gui=field(client,"client.gui","gui");screen=call(gui,"gui.screen","screen");}return screen;}
    private boolean title(Object screen){if(screen==null)return false;String name=screen.getClass().getName();return name.endsWith(".TitleScreen")||name.endsWith(".GuiMainMenu")||name.equals("net.minecraft.class_442")||name.equals(bindings.getProperty(namespace+".screen.title_class",""));}
    private String titleRole(Object button){
        Object message=call(button,"widget.message","getMessage,getText");Object contents=call(message,"component.contents","getContents,getContent");Object key=call(contents,"translation.key","getKey");String text=key instanceof String?(String)key:"";
        if(text.isEmpty()){Object label=call(message,"component.text","getString,asString");if(label instanceof String)text=((String)label).toLowerCase(Locale.ROOT);else{Object label2=field(button,"widget.label","displayString");if(label2 instanceof String)text=((String)label2).toLowerCase(Locale.ROOT);}}
        if(Arrays.asList("menu.singleplayer","singleplayer","un jugador","un solo jugador").contains(text))return "singleplayer";
        if(Arrays.asList("menu.multiplayer","multiplayer","multijugador").contains(text))return "multiplayer";
        if(Arrays.asList("menu.options","options...","opciones...","options","opciones").contains(text))return "options";
        if(Arrays.asList("menu.quit","quit game","salir del juego").contains(text))return "quit";return "";
    }
    private Iterable<?> buttons(Object screen){Object children=call(screen,"screen.children","children,children");if(!(children instanceof Iterable))children=field(screen,"screen.buttons","buttonList,field_146292_n");return children instanceof Iterable?(Iterable<?>)children:Collections.emptyList();}
    private String screenLabels(Object client){List<String> labels=new ArrayList<String>();labels.add("\"fullscreen_borderless\":"+quote(Boolean.toString(fullscreenBorderless)));Object current=screen(client);if(title(current)){labels.add("\"screen\":\"title\"");for(Object button:buttons(current)){String role=titleRole(button);if(!role.isEmpty()&&!Boolean.FALSE.equals(field(button,"widget.active","active,enabled,field_146124_l")))labels.add(quote("title_"+role)+":\"true\"");}}return "{"+join(labels)+"}";}
    private void titleAction(Object client,String role)throws Exception{
        Object current=screen(client);if(!title(current))throw new IOException("El menú principal ya no está abierto");
        for(Object button:buttons(current)){if(!role.equals(titleRole(button)))continue;if(Boolean.FALSE.equals(field(button,"widget.active","active,enabled,field_146124_l")))throw new IOException("Acción deshabilitada por Minecraft");
            Method method=(Method)member(button.getClass(),"button.press",true,"onPress,onPress");if(method!=null){method.invoke(button);return;}
            for(Method candidate:button.getClass().getMethods())if(candidate.getName().equals("onPress")&&candidate.getParameterTypes().length==1&&candidate.getParameterTypes()[0].isInterface()){
                Class<?> input=candidate.getParameterTypes()[0];Object event=java.lang.reflect.Proxy.newProxyInstance(input.getClassLoader(),new Class<?>[]{input},new InvocationHandler(){public Object invoke(Object proxy,Method method,Object[] args){if(method.getReturnType()==int.class)return method.getName().equals("input")?257:0;if(method.getReturnType()==boolean.class)return method.getName().equals("isConfirmation");return null;}});candidate.invoke(button,event);return;
            }

            for(Class<?> owner=current.getClass();owner!=null;owner=owner.getSuperclass())for(Method action:owner.getDeclaredMethods())if(Arrays.asList(names("screen.action","actionPerformed,func_146284_a")).contains(action.getName())&&action.getParameterTypes().length==1&&action.getParameterTypes()[0].isInstance(button)){action.setAccessible(true);action.invoke(current,button);return;}
        }throw new IOException("Este menú no expone la acción solicitada");
    }
    private void fullscreen(Object client){
        if(!Pattern.compile("\"auto_borderless\"\\s*:\\s*true").matcher(fogProfile).find())return;
        Object window=call(client,"client.window","getWindow");if(window==null)return;
        if(Boolean.TRUE.equals(call(window,"window.fullscreen","isFullscreen,isFullscreen"))){Method toggle=(Method)member(window.getClass(),"window.toggle_fullscreen",true,"toggleFullScreen,toggleFullscreen");if(toggle!=null)try{toggle.invoke(window);fullscreenBorderless=true;}catch(Exception ignored){}}
    }



    private void pollZoom(final Object client,final boolean overlayEnabled){
        if(!zoomQueued.compareAndSet(false,true))return;
        Runnable task=new Runnable(){public void run(){try{updateZoom(client,overlayEnabled);}catch(Throwable ignored){restoreZoom();zoomAvailable=false;}finally{zoomQueued.set(false);}}};
        try{if(client instanceof java.util.concurrent.Executor)((java.util.concurrent.Executor)client).execute(task);
            else{Method scheduler=(Method)member(client.getClass(),"client.execute",true,"execute,addScheduledTask,func_152344_a",Runnable.class);if(scheduler==null){zoomQueued.set(false);zoomAvailable=false;return;}scheduler.invoke(client,task);}}
        catch(Throwable ignored){zoomQueued.set(false);zoomAvailable=false;}
    }
    private boolean delegatesZoom(){
        long time=System.currentTimeMillis();if(time-lastZoomDelegateCheck<2000)return zoomDelegate;lastZoomDelegateCheck=time;zoomDelegate=false;
        for(Class<?> type:instrumentation.getAllLoadedClasses()){String name=type.getName();if(name.equals("dev.tplauncher.companion.TpCompanion")||name.equals("net.optifine.Config")||name.equals("Config")||name.startsWith("dev.isxander.zoomify.")||name.startsWith("io.github.ennuil.ok_zoomer.")){zoomDelegate=true;break;}}
        return zoomDelegate;
    }
    private Boolean zoomKey(Object client){
        try{ClassLoader loader=client.getClass().getClassLoader();Object window=call(client,"client.window","getWindow");Object handle=call(window,"window.handle","handle,getWindow");
            if(handle instanceof Long&&((Long)handle)>0){Class<?> glfw=Class.forName("org.lwjgl.glfw.GLFW",false,loader);int focused=((Number)glfw.getMethod("glfwGetWindowAttrib",long.class,int.class).invoke(null,handle,0x00020001)).intValue();return focused==1&&((Number)glfw.getMethod("glfwGetKey",long.class,int.class).invoke(null,handle,67)).intValue()==1;}
            Class<?> display=Class.forName("org.lwjgl.opengl.Display",false,loader);boolean focused=Boolean.TRUE.equals(display.getMethod("isActive").invoke(null));Class<?> keyboard=Class.forName("org.lwjgl.input.Keyboard",false,loader);return focused&&Boolean.TRUE.equals(keyboard.getMethod("isKeyDown",int.class).invoke(null,46));
        }catch(Throwable unavailable){return null;}
    }
    private void updateZoom(Object client,boolean overlayEnabled)throws Exception{
        if(delegatesZoom()){restoreZoom();zoomAvailable=false;return;}
        Object options=field(client,"client.options","options,gameSettings,field_71474_y");if(options==null){restoreZoom();zoomAvailable=false;return;}
        Object option=call(options,"options.fov","fov,getFov");Object value=call(option,"option.get","get,getValue");Method setter=null;Field setting=null;
        if(value instanceof Number)setter=(Method)member(option.getClass(),"option.set",true,"set,setValue",Object.class);
        if(setter==null){setting=(Field)member(options.getClass(),"options.fov_field",false,"fovSetting,fov");if(setting!=null&&Arrays.asList(float.class,double.class,int.class).contains(setting.getType()))value=setting.get(options);else value=null;}
        Boolean key=zoomKey(client);zoomAvailable=value instanceof Number&&(setter!=null||setting!=null)&&key!=null;
        boolean active=zoomAvailable&&overlayEnabled&&Pattern.compile("\\{[^{}]{0,1024}\"id\"\\s*:\\s*\"optical_zoom\"[^{}]{0,1024}\"enabled\"\\s*:\\s*true[^{}]{0,1024}\\}").matcher(fogProfile).find()
            &&Boolean.TRUE.equals(key)&&field(client,"client.screen","screen,currentScreen,field_71462_r")==null&&field(client,"client.player","player,thePlayer,field_71439_g")!=null;
        if(!active){restoreZoom();return;}
        Object target=setter!=null?option:options;
        if(zoomTarget!=target){restoreZoom();zoomTarget=target;zoomSetter=setter;zoomField=setting;zoomOriginal=(Number)value;double original=zoomOriginal.doubleValue();if(!Double.isFinite(original)||original<30||original>180){restoreZoom();zoomAvailable=false;return;}writeZoom(Math.max(30,original/3));}
    }
    private void writeZoom(double number)throws Exception{
        Object value=zoomOriginal instanceof Integer?(Object)Integer.valueOf((int)Math.round(number)):zoomOriginal instanceof Float?(Object)Float.valueOf((float)number):(Object)Double.valueOf(number);
        if(zoomSetter!=null)zoomSetter.invoke(zoomTarget,value);else if(zoomField!=null)zoomField.set(zoomTarget,value);
    }
    private void restoreZoom(){
        try{if(zoomTarget!=null&&zoomOriginal!=null)writeZoom(zoomOriginal.doubleValue());}catch(Throwable ignored){}
        zoomTarget=null;zoomSetter=null;zoomField=null;zoomOriginal=null;
    }
    private void configureFocus(Object client){
        final Object options=field(client,"client.options","options,gameSettings,field_71474_y");if(options==null)return;
        final Field setting=(Field)member(options.getClass(),"options.pause",false,"pauseOnLostFocus,field_82881_y");if(setting==null||setting.getType()!=boolean.class)return;
        Runnable action=new Runnable(){public void run(){try{setting.setBoolean(options,false);fullscreen(client);}catch(Exception ignored){}}};
        if(client instanceof java.util.concurrent.Executor)((java.util.concurrent.Executor)client).execute(action);
        else callOne(client,"client.execute","execute,addScheduledTask,func_152344_a",action);
    }

    private void pollCommand(final Object client){
        final Path folder=profile.getParent(),path=folder.resolve("command-"+pid+".properties");
        try{if(!Files.isRegularFile(path)||Files.size(path)>16384)return;final Properties p=new Properties();try(Reader reader=Files.newBufferedReader(path,StandardCharsets.UTF_8)){p.load(reader);}final String id=p.getProperty("id","");if(!id.matches("[0-9a-f-]{36}")||id.equals(lastCommand)||!p.getProperty("adapter","").equals("bridge"))return;lastCommand=id;
            Runnable action=new Runnable(){public void run(){try{String kind=p.getProperty("action","");
                if(kind.equals("memory_gc")){System.gc();ack(id,true,"Solicitud de recolección enviada. La JVM puede aplazarla o ignorarla.");return;}
                if(kind.equals("title")){titleAction(client,p.getProperty("target",""));}
                else if(kind.equals("option"))setOption(client,p.getProperty("name",""),Double.parseDouble(p.getProperty("value","")));
                else if(kind.equals("reload_resources")){Method method=(Method)member(client.getClass(),"client.reload",true,"reloadResourcePacks,reloadResources,refreshResources,func_110436_a");if(method==null)throw new IOException("Esta versión no expone una recarga compatible");method.invoke(client);}
                else if(kind.equals("packs")){
                    Object options=field(client,"client.options","options,gameSettings,field_71474_y"),manager=call(client,"client.packs","getResourcePackRepository,getResourcePackManager");if(options==null||manager==null)throw new IOException("No pack manager");
                    List<String> selected=new ArrayList<String>();for(int i=0;i<128;i++){String pack=p.getProperty("pack."+i);if(pack==null)break;selected.add(pack);}
                    Field packs=(Field)member(options.getClass(),"options.packs",false,"resourcePacks,field_151453_l");if(packs==null||!java.util.Collection.class.isAssignableFrom(packs.getType()))throw new IOException("No pack list");
                    Method refresh=null;for(String name:names("options.refresh_packs","loadSelectedResourcePacks,addResourcePackProfilesToManager"))for(Method method:options.getClass().getMethods())if(method.getName().equals(name)&&method.getParameterTypes().length==1&&method.getParameterTypes()[0].isInstance(manager)){refresh=method;break;}
                    if(refresh==null)throw new IOException("No pack adapter");
                    packs.set(options,selected);Method scan=(Method)member(manager.getClass(),"packs.reload",true,"reload,scanPacks");if(scan!=null)scan.invoke(manager);refresh.invoke(options,manager);
                    Method save=(Method)member(options.getClass(),"options.save",true,"save,write,saveOptions,func_74303_b");if(save!=null)save.invoke(options);
                    Method reload=(Method)member(client.getClass(),"client.reload",true,"reloadResourcePacks,reloadResources,refreshResources,func_110436_a");if(reload==null)throw new IOException("No reload");Object result=reload.invoke(client);
                    if(result instanceof java.util.concurrent.CompletionStage){((java.util.concurrent.CompletionStage<?>)result).whenComplete(new java.util.function.BiConsumer<Object,Throwable>(){public void accept(Object value,Throwable error){ack(id,error==null,error==null?"Paquetes recargados en Minecraft.":"Minecraft no pudo recargar los paquetes.");}});return;}
                }
                else if(kind.equals("shader")){
                    ClassLoader loader=client.getClass().getClassLoader();Class<?> iris=null;for(String candidate:new String[]{"net.irisshaders.iris.Iris","net.coderbot.iris.Iris"})try{iris=Class.forName(candidate,false,loader);break;}catch(ClassNotFoundException ignored){}
                    String name=p.getProperty("name","");if(iris!=null){Object config=iris.getMethod("getIrisConfig").invoke(null);config.getClass().getMethod("setShaderPackName",String.class).invoke(config,name);config.getClass().getMethod("setShadersEnabled",boolean.class).invoke(config,!name.isEmpty());config.getClass().getMethod("save").invoke(config);iris.getMethod("reload").invoke(null);}
                    else{Class<?> shaders=Class.forName("net.optifine.shaders.Shaders",true,loader);shaders.getMethod("setShaderPack",String.class).invoke(null,name);shaders.getMethod("uninit").invoke(null);}
                }
                else throw new IOException("Este control necesita TP Companion compatible.");
                ack(id,true,"Cambio aplicado en Minecraft.");
            }catch(Throwable error){ack(id,false,"El adaptador de esta versión no pudo aplicar este control.");}}};
            if(client instanceof java.util.concurrent.Executor)((java.util.concurrent.Executor)client).execute(action);
            else{Method scheduler=(Method)member(client.getClass(),"client.execute",true,"execute,addScheduledTask,func_152344_a",Runnable.class);if(scheduler==null){ack(id,false,"Esta versión no expone un ejecutor compatible.");return;}scheduler.invoke(client,action);}
        }catch(Throwable ignored){}
    }
    private void setOption(Object client,String name,double value)throws Exception{
        if(!Double.isFinite(value))throw new IOException("Invalid value");Object options=field(client,"client.options","options,gameSettings,field_71474_y");if(options==null)throw new IOException("No options");
        if(name.equals("volume")||name.equals("music")){
            boolean applied=false;for(String getter:names("options.audio","getSoundSourceOptionInstance,getSoundVolumeOption"))for(Method method:options.getClass().getMethods())if(method.getName().equals(getter)&&method.getParameterTypes().length==1&&method.getParameterTypes()[0].isEnum()){
                Field category=(Field)member(method.getParameterTypes()[0],name.equals("music")?"audio.music":"audio.master",false,name.equals("music")?"MUSIC":"MASTER");if(category==null)continue;Object option=method.invoke(options,category.get(null));Method setter=(Method)member(option.getClass(),"option.set",true,"set,setValue",Object.class);if(setter!=null){setter.invoke(option,Double.valueOf(Math.max(0,Math.min(1,value))));applied=true;break;}
            }if(!applied)throw new IOException("No compatible audio option");Method save=(Method)member(options.getClass(),"options.save",true,"save,write,saveOptions,func_74303_b");if(save!=null)save.invoke(options);return;
        }
        String fieldName,getter;double min,max;boolean integer=false;
        if(name.equals("fov")){fieldName="fovSetting,fov";getter="fov,getFov";min=30;max=110;integer=true;}
        else if(name.equals("chunks")){fieldName="renderDistanceChunks,renderDistance";getter="renderDistance,getViewDistance";min=2;max=32;integer=true;}
        else if(name.equals("gamma")){fieldName="gammaSetting,gamma";getter="gamma,getGamma";min=0;max=1;}
        else if(name.equals("gui")){fieldName="guiScale";getter="guiScale,getGuiScale";min=0;max=8;integer=true;}
        else if(name.equals("fps_limit")){fieldName="limitFramerate";getter="framerateLimit,getMaxFps";min=10;max=260;integer=true;}
        else throw new IOException("Unsupported option");
        double bounded=Math.max(min,Math.min(max,value));Object option=call(options,"options."+name,getter);boolean applied=false;
        if(option!=null){Object typed=integer?(Object)Integer.valueOf((int)bounded):(Object)Double.valueOf(bounded);Method setter=(Method)member(option.getClass(),"option.set",true,"set,setValue",Object.class);if(setter!=null){setter.invoke(option,typed);applied=true;}}
        if(!applied){Field f=(Field)member(options.getClass(),"options."+name+"_field",false,fieldName);if(f==null)throw new IOException("No option");if(f.getType()==float.class)f.setFloat(options,(float)bounded);else if(f.getType()==double.class)f.setDouble(options,bounded);else if(f.getType()==int.class)f.setInt(options,(int)bounded);else throw new IOException("No numeric option");}
        Method save=(Method)member(options.getClass(),"options.save",true,"save,write,saveOptions,func_74303_b");if(save!=null)save.invoke(options);
    }
    private void ack(String id,boolean ok,String message){try{Path path=profile.getParent().resolve("ack-"+pid+".json"),pending=path.resolveSibling(path.getFileName()+".tmp");Files.write(pending,("{\"id\":"+quote(id)+",\"ok\":"+ok+",\"message\":"+quote(message)+"}").getBytes(StandardCharsets.UTF_8));Files.move(pending,path,StandardCopyOption.REPLACE_EXISTING,StandardCopyOption.ATOMIC_MOVE);}catch(Exception ignored){}}

    private static void numeric(List<String> rows,String key,Object value){double number;if(value instanceof Boolean)number=((Boolean)value)?1:0;else if(value instanceof Number)number=((Number)value).doubleValue();else return;if(Double.isNaN(number)||Double.isInfinite(number))return;rows.add(quote(key)+":"+number);}
    private String itemId(Object stack){
        Object item=call(stack,"stack.item","getItem,func_77973_b"),key=call(item,"item.translation","getDescriptionId,getTranslationKey,getUnlocalizedName,func_77658_a");
        if(key instanceof String){String text=(String)key;if(text.startsWith("item.minecraft.")||text.startsWith("block.minecraft."))return text.substring(text.indexOf("minecraft.")+10);
            String legacy=text.startsWith("item.")?text.substring(5):text;
            for(String[] piece:new String[][]{{"helmet","helmet"},{"chestplate","chestplate"},{"leggings","leggings"},{"boots","boots"},{"sword","sword"},{"shovel","shovel"},{"pickaxe","pickaxe"},{"hatchet","axe"},{"hoe","hoe"}})
                for(String[] material:new String[][]{{"Cloth","leather"},{"Chain","chainmail"},{"Iron","iron"},{"Gold","golden"},{"Diamond","diamond"},{"Wood","wooden"},{"Stone","stone"}})if(legacy.equals(piece[0]+material[0]))return material[1]+"_"+piece[1];
            for(String[] tool:new String[][]{{"bow","bow"},{"fishingRod","fishing_rod"},{"flintAndSteel","flint_and_steel"},{"shears","shears"},{"carrotOnAStick","carrot_on_a_stick"}})if(legacy.equals(tool[0]))return tool[1];
        }return item==null?"":item.toString().replace("minecraft:","");
    }
    private void durability(List<String> rows,Object stack,String key){if(stack==null)return;Double max=number(call(stack,"stack.max","getMaxDamage,func_77958_k")),damage=number(call(stack,"stack.damage","getDamageValue,getDamage,getItemDamage,func_77952_i"));if(max!=null&&damage!=null&&max>0)numeric(rows,key,Math.max(0,max-damage));}
    private void light(List<String> rows,Object world,Object pos){
        if(pos==null)return;
        try{for(Method method:world.getClass().getMethods())if(Arrays.asList(names("world.brightness","getBrightness,getLightLevel")).contains(method.getName())&&method.getParameterTypes().length==2&&method.getParameterTypes()[0].isEnum()&&method.getParameterTypes()[1].isInstance(pos)){
            Object[] constants=method.getParameterTypes()[0].getEnumConstants();if(constants==null||constants.length==0)return;
            for(String type:new String[]{"block","sky"}){Object channel=field(constants[0],"light."+type,type.toUpperCase(Locale.ROOT));if(channel!=null)numeric(rows,type+"_light",method.invoke(world,channel,pos));}return;
        }}catch(Exception ignored){}
    }
    private String playerMetrics(Object player,Object world,Object client,boolean restricted){
        List<String> rows=new ArrayList<String>();
        String[][] getters={{"health","player.health","getHealth"},{"max_health","player.max_health","getMaxHealth"},{"absorption","player.absorption","getAbsorptionAmount"},{"armor_points","player.armor_points","getArmorValue,getArmor"},{"oxygen","entity.air","getAirSupply,getAir"},{"fire_ticks","entity.fire","getRemainingFireTicks"},{"pitch","entity.pitch","getXRot,getPitch"},{"yaw","entity.yaw","getYRot,getYaw"},{"score","player.score","getScore"},{"sprinting","entity.sprinting","isSprinting"},{"sneaking","entity.sneaking","isCrouching,isSneaking"},{"swimming","entity.swimming","isSwimming"},{"on_ground","entity.on_ground","onGround,isOnGround"}};
        for(String[] getter:getters){if(restricted&&(getter[0].equals("pitch")||getter[0].equals("yaw")))continue;numeric(rows,getter[0],call(player,getter[1],getter[2]));}
        for(String[] f:new String[][]{{"xp_level","experienceLevel"},{"xp_total","totalExperience"},{"xp_progress","experienceProgress"},{"fall_distance","fallDistance"},{"hurt_ticks","hurtTime"}})numeric(rows,f[0],field(player,"player."+f[0],f[1]));
        Object food=call(player,"player.food","getFoodData,getHungerManager");for(String[] getter:new String[][]{{"food","food.level","getFoodLevel"},{"saturation","food.saturation","getSaturationLevel,getSaturation"},{"exhaustion","food.exhaustion","getExhaustionLevel,getExhaustion"}})numeric(rows,getter[0],call(food,getter[1],getter[2]));
        Object effects=call(player,"player.effects","getActiveEffects,getStatusEffects");if(effects instanceof Collection)numeric(rows,"effect_count",((Collection<?>)effects).size());
        Object main=call(player,"player.mainhand","getMainHandItem,getMainHandStack,getHeldItem"),off=call(player,"player.offhand","getOffhandItem,getOffHandStack");durability(rows,main,"main_remaining");durability(rows,off,"off_remaining");
        Object inventory=call(player,"player.inventory_method","getInventory");if(inventory==null)inventory=field(player,"player.inventory","inventory,field_71071_by");
        Double size=number(call(inventory,"inventory.size","getContainerSize,size,getSizeInventory"));
        if(size!=null&&size>=36&&size<=256){int free=0,total=0;Map<String,Integer> counts=new HashMap<String,Integer>();boolean complete=true;for(int i=0;i<36;i++){Object stack=invoke(inventory,"inventory.stack","getItem,getStack,getStackInSlot",new Class<?>[]{int.class},i);if(stack==null){complete=false;break;}if(Boolean.TRUE.equals(call(stack,"stack.empty","isEmpty,func_190926_b"))){free++;continue;}Double count=number(call(stack,"stack.count","getCount"));if(count==null)count=number(field(stack,"stack.count_field","stackSize"));if(count==null){complete=false;break;}total+=count.intValue();String id=itemId(stack);counts.put(id,(counts.containsKey(id)?counts.get(id):0)+count.intValue());}if(complete){numeric(rows,"inventory_free",free);numeric(rows,"inventory_items",total);for(String id:new String[]{"arrow","spectral_arrow","tipped_arrow","ender_pearl","golden_apple","enchanted_golden_apple","totem_of_undying","firework_rocket","mushroom_stew","suspicious_stew","splash_potion","potion","experience_bottle","tnt","obsidian","cobblestone"})numeric(rows,"count_"+id,counts.containsKey(id)?counts.get(id):0);}}
        if(!restricted)light(rows,world,call(player,"entity.block_pos","blockPosition,getBlockPos,getPosition"));
        Object difficulty=call(world,"world.difficulty","getDifficulty");if(difficulty instanceof Enum)numeric(rows,"difficulty",((Enum<?>)difficulty).ordinal());
        numeric(rows,"rain",invoke(world,"world.rain","getRainLevel,getRainGradient",new Class<?>[]{float.class},1f));numeric(rows,"thunder",invoke(world,"world.thunder","getThunderLevel,getThunderGradient",new Class<?>[]{float.class},1f));
        Object options=field(client,"client.options","options,gameSettings,field_71474_y");for(String[] option:new String[][]{{"view_distance","renderDistance,getViewDistance"},{"fov","fov,getFov"},{"gamma","gamma,getGamma"}}){Object value=call(options,"options."+(option[0].equals("view_distance")?"chunks":option[0]),option[1]);numeric(rows,option[0],call(value,"option.get","get,getValue"));}
        Object connection=call(client,"client.connection","getConnection,getNetworkHandler,getNetHandler");Object players=call(connection,"connection.players","getOnlinePlayers,getPlayerList");if(players instanceof Collection)numeric(rows,"online_players",((Collection<?>)players).size());
        Object velocity=call(player,"entity.velocity","getDeltaMovement,getVelocity");Double vx=number(field(velocity,"vec.x","x")),vy=number(field(velocity,"vec.y","y")),vz=number(field(velocity,"vec.z","z"));if(vx!=null&&vz!=null)rows.add("\"speed\":"+(Math.hypot(vx,vz)*20));if(vy!=null)rows.add("\"vertical_speed\":"+(vy*20));
        return "{"+join(rows)+"}";
    }
    private String base() { return "{\"schema_version\":1,\"pid\":" + pid + ",\"timestamp_ms\":" + System.currentTimeMillis() + ",\"source\":\"TP Bridge\",\"metrics\":{}"; }
    private String waiting() {double[] render=RenderHook.metrics();return base() + ",\"has_world\":false,\"game_fps\":"+(render==null?"null":Double.toString(render[0]))+",\"armor\":[],\"capabilities\":"+(render==null?"[]":"[\"fps\"]")+",\"status\":\"Conectando con el mundo.\"}"; }
    private void publish(String json) throws IOException {
        if (stopping || json.getBytes(StandardCharsets.UTF_8).length > 16384) return;
        if(transport!=null)try{byte[] packet=("{\"token\":"+quote(transportToken)+",\"payload\":"+json+"}").getBytes(StandardCharsets.UTF_8);transport.send(new DatagramPacket(packet,packet.length,InetAddress.getByName("127.0.0.1"),port));}catch(Exception ignored){}
        Files.createDirectories(output.getParent());
        Files.write(pending, json.getBytes(StandardCharsets.UTF_8));
        try { Files.move(pending, output, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING); }
        catch (AtomicMoveNotSupportedException ignored) { Files.move(pending, output, StandardCopyOption.REPLACE_EXISTING); }
    }
    private void removeFiles() { try { Files.deleteIfExists(output); Files.deleteIfExists(pending); } catch (IOException ignored) { } }
    private static String join(Collection<String> values) { StringBuilder b = new StringBuilder(); for (String value : values) { if (b.length() > 0) b.append(','); b.append(value); } return b.toString(); }
    private static String quote(String value) {
        value = new String(value.getBytes(StandardCharsets.UTF_8),StandardCharsets.UTF_8);
        StringBuilder b = new StringBuilder("\"");
        for (int i = 0; i < value.length(); i++) {
            char c = value.charAt(i);
            if (c == '"' || c == '\\') b.append('\\').append(c);
            else if (c < 32 || Character.isSurrogate(c)) { String hex = Integer.toHexString(c); b.append("\\u"); for (int p = hex.length(); p < 4; p++) b.append('0'); b.append(hex); }
            else b.append(c);
        }
        return b.append('"').toString();
    }
}
