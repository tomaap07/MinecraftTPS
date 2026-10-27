package fixture;
public final class RenderOnlyClient {
    public static void main(String[] args)throws Exception{long end=System.currentTimeMillis()+10000;while(System.currentTimeMillis()<end){org.lwjgl.glfw.GLFW.glfwSwapBuffers(7);Thread.sleep(5);}}
}
