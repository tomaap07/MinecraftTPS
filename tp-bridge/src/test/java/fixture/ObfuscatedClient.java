package fixture;

public final class ObfuscatedClient {
    private static final ObfuscatedClient s=new ObfuscatedClient();
    private static int f=83;
    private Player p=new Player(); private Object w=new Object();
    public static final class Player {
        public double a(){return 20.25;} public double b(){return -4.5;} public double c(){return -22.25;}
    }
    public static void main(String[] args)throws Exception{long until=System.currentTimeMillis()+12000;while(System.currentTimeMillis()<until)Thread.sleep(25);}
}
