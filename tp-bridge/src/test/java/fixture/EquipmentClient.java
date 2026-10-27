package fixture;

public final class EquipmentClient {
    private static final class Holder { private static final EquipmentClient value=new EquipmentClient(); }
    public static EquipmentClient getInstance(){return Holder.value;}
    private static int fps=91;
    private final Player player=new Player(); private final Object level=new Object();
    public enum Slot{CAP,CHEST,LEGS,FEET,MAINHAND,OFFHAND,BODY}
    public static final class Player {
        public double getX(){return 1;} public double getY(){return 70;} public double getZ(){return 2;}
        public Stack e(Slot slot){return new Stack(slot==Slot.CAP||slot==Slot.MAINHAND||slot==Slot.BODY);}
        public Stack getMainHandItem(){return new Stack(true);}
    }
    public static final class Stack {
        private final boolean present;Stack(boolean value){present=value;}
        public boolean isEmpty(){return !present;}public int getMaxDamage(){return 80;}public int getDamageValue(){return 7;}
        public String getDisplayName(){return "Fixture armor";}
        public Item getItem(){return new Item();}
    }
    public static final class Item {public String getUnlocalizedName(){return "item.swordDiamond";}}
    public static void main(String[] args)throws Exception{getInstance();long end=System.currentTimeMillis()+12000;while(System.currentTimeMillis()<end)Thread.sleep(25);}
}
