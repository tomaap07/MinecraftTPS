package fixture;

public final class LegacyClient {
    private static final LegacyClient singleton=new LegacyClient();
    private final String debug="73 fps, 0 chunk updates";
    private final Player p=new Player(); private final Object w=new Object();
    public static final class Player { private double a=-10.5,b=65,c=40.75; private float d=-90; }
    public static void main(String[] args)throws Exception{long end=System.currentTimeMillis()+12000;while(System.currentTimeMillis()<end)Thread.sleep(25);}
}
