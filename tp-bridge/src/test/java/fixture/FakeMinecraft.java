package fixture;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;

public final class FakeMinecraft implements java.util.concurrent.Executor {
    private static final FakeMinecraft instance=new FakeMinecraft();
    private static int fps=137;
    private Player player=new Player();
    private World level=new World();
    private boolean local;
    private static volatile boolean modernOptions;
    private final Options options=new Options();
    private Object screen;
    public static final class Window{public long getWindow(){return 1;}}
    public Window getWindow(){return new Window();}
    private final java.util.Queue<Runnable> tasks=new java.util.concurrent.ConcurrentLinkedQueue<Runnable>();
    public void execute(Runnable runnable){tasks.add(runnable);}
    public boolean isLocalServer(){return local;}
    public static final class Options{public boolean pauseOnLostFocus=true;public float fovSetting=70;public int saved;private final Option<Integer> fov=new Option<Integer>(70);private final Option<Double> volume=new Option<Double>(1.0);public Option<Integer> getFov(){return modernOptions?fov:null;}public Option<Double> getSoundVolumeOption(Audio category){return volume;}public void saveOptions(){saved++;}}
    public enum Audio{MASTER,MUSIC}
    public static final class Option<T>{private T value;public Option(T value){this.value=value;}public void setValue(T next){if(next.getClass()!=value.getClass())throw new IllegalArgumentException("Wrong numeric option type");value=next;}public T getValue(){return value;}}
    public static final class Key{private final String id;public Key(String id){this.id=id;}public String getValue(){return id;}}
    public static final class Position{}
    public enum Light{BLOCK,SKY}
    public static final class World{public long getDayTime(){return 48000;}public long getGameTime(){return 98001;}public int getBrightness(Light channel,Position pos){return channel==Light.BLOCK?7:15;}public Key dimension(){return new Key("minecraft:overworld");}public Key getBiome(Position pos){return new Key("minecraft:plains");}}

    private boolean restricted;
    public static final class Player {
        public double getX(){return -12.75;} public double getY(){return 64.5;} public double getZ(){return 45.125;}
        public Position blockPosition(){return new Position();}public boolean isInWater(){return false;}public boolean isInLava(){return false;}
        public float getYRot(){return 90;}
        public UUID getUUID(){return new UUID(0,1);}
        public Inventory getInventory(){return new Inventory();}
    }
    public static final class Inventory { public final List<Stack> armor=Arrays.asList(new Stack()); }
    public static final class Stack {
        public boolean isEmpty(){return false;} public int getMaxDamage(){return 100;} public int getDamageValue(){return 13;}
        public String getDisplayName(){return "Casco \\\" local · 漢字 · 🛡";}
        public Item getItem(){return new Item();}
    }
    public static final class Item{public String getDescriptionId(){return "item.minecraft.diamond_helmet";}}
    public Connection getConnection(){return new Connection();}
    public boolean showOnlyReducedInfo(){return restricted;}
    public static final class Connection {public Info getPlayerInfo(UUID uuid){return new Info();}}
    public static final class Info {public int getLatency(){return 42;}}
    public static void main(String[] args)throws Exception{
        Path phase=Paths.get(args[0]);long until=System.currentTimeMillis()+30000;
        while(System.currentTimeMillis()<until){
            String value=Files.exists(phase)?new String(Files.readAllBytes(phase),StandardCharsets.UTF_8):"world";
            instance.restricted=value.equals("restricted");instance.local=value.equals("local");modernOptions=value.equals("modern-options")||value.startsWith("zoom-modern");
            org.lwjgl.glfw.GLFW.c=value.startsWith("zoom");org.lwjgl.glfw.GLFW.focused=!value.endsWith("unfocused");instance.screen=value.endsWith("screen")?new Object():null;
            Runnable task;while((task=instance.tasks.poll())!=null)task.run();
            org.lwjgl.opengl.GL11.glFogf(0x0B64,100);
            org.lwjgl.glfw.GLFW.glfwSwapBuffers(1);
            Path snapshot=phase.resolveSibling("settings.json"),pending=phase.resolveSibling("settings.pending");Files.write(pending,("{\"pause\":"+instance.options.pauseOnLostFocus+",\"fov\":"+(modernOptions?instance.options.fov.getValue():instance.options.fovSetting)+",\"volume\":"+instance.options.volume.getValue()+",\"saved\":"+instance.options.saved+",\"fog\":"+org.lwjgl.opengl.GL11.lastFog+"}").getBytes(StandardCharsets.UTF_8));try{Files.move(pending,snapshot,StandardCopyOption.REPLACE_EXISTING,StandardCopyOption.ATOMIC_MOVE);}catch(java.nio.file.AccessDeniedException reading){ }
            instance.player=value.equals("menu")?null:new Player();instance.level=value.equals("menu")?null:new World();
            Thread.sleep(25);
        }
    }
}
