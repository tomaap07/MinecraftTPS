package org.lwjgl.glfw;

public final class GLFW {public static volatile boolean focused=true,c;public static void glfwSwapBuffers(long handle){}public static int glfwGetWindowAttrib(long handle,int attribute){return focused?1:0;}public static int glfwGetKey(long handle,int key){return key==67&&c?1:0;} }
