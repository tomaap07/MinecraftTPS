package fixture;
public final class LegacyRenderOnlyClient {
    public static void main(String[] args)throws Exception{long end=System.currentTimeMillis()+10000;while(System.currentTimeMillis()<end){org.lwjgl.opengl.Display.swapBuffers();Thread.sleep(8);}}
}
