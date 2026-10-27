package dev.tplauncher.bridge;
import java.io.*;import java.nio.file.*;import java.security.*;import java.lang.instrument.*;import java.util.jar.*;
import org.objectweb.asm.*;

final class RenderHook implements ClassFileTransformer {
    private static JarFile bootstrap;
    private static volatile boolean fogHook;
    private static boolean target(String name){return name.equals("org.lwjgl.glfw.GLFW")||name.equals("org.lwjgl.opengl.Display")||name.equals("com.mojang.blaze3d.systems.RenderSystem")||name.equals("org.lwjgl.opengl.GL11")||name.equals("org.lwjgl.opengl.GL11C");}
    static void install(Instrumentation instrumentation,Path folder)throws Exception{
        Files.createDirectories(folder);byte[] data;
        try(InputStream input=TpBridge.class.getResourceAsStream("/bootstrap/tp-hud-runtime.jar")){if(input==null)throw new IOException("Missing runtime");ByteArrayOutputStream out=new ByteArrayOutputStream();byte[] chunk=new byte[4096];int n;while((n=input.read(chunk))!=-1){if(out.size()+n>65536)throw new IOException("Runtime too large");out.write(chunk,0,n);}data=out.toByteArray();}
        StringBuilder key=new StringBuilder();for(byte b:MessageDigest.getInstance("SHA-256").digest(data))key.append(String.format("%02x",b&255));
        Path jar=folder.resolve("hud-runtime-"+key+".jar");if(!Files.isRegularFile(jar)||!java.util.Arrays.equals(Files.readAllBytes(jar),data))Files.write(jar,data);
        bootstrap=new JarFile(jar.toFile());instrumentation.appendToBootstrapClassLoaderSearch(bootstrap);
        Class.forName("dev.tplauncher.hud.runtime.FrameCounter",true,null);
        RenderHook hook=new RenderHook();instrumentation.addTransformer(hook,true);
        for(Class<?> type:instrumentation.getAllLoadedClasses())if(target(type.getName())&&instrumentation.isModifiableClass(type))try{instrumentation.retransformClasses(type);}catch(Throwable ignored){}
    }
    public byte[] transform(ClassLoader loader,String name,Class<?> type,ProtectionDomain domain,byte[] bytes){
        if(name==null||!target(name.replace('/','.')))return null;
        try{final boolean glfw=name.equals("org/lwjgl/glfw/GLFW");ClassReader reader=new ClassReader(bytes);ClassWriter writer=new ClassWriter(reader,ClassWriter.COMPUTE_MAXS);
            reader.accept(new ClassVisitor(Opcodes.ASM9,writer){
                public MethodVisitor visitMethod(int access,String method,String desc,String signature,String[] exceptions){MethodVisitor next=super.visitMethod(access,method,desc,signature,exceptions);
                    if((access&(Opcodes.ACC_NATIVE|Opcodes.ACC_ABSTRACT))!=0)return next;
                    if(method.equals("glFogf")&&desc.equals("(IF)V")){fogHook=true;return new MethodVisitor(Opcodes.ASM9,next){public void visitCode(){super.visitCode();visitVarInsn(Opcodes.ILOAD,0);visitVarInsn(Opcodes.FLOAD,1);visitMethodInsn(Opcodes.INVOKESTATIC,"dev/tplauncher/hud/runtime/FrameCounter","tuneFog","(IF)F",false);visitVarInsn(Opcodes.FSTORE,1);}};}
                    if((method.equals("setShaderFogStart")||method.equals("setShaderFogEnd"))&&desc.equals("(F)V")){fogHook=true;final int parameter=method.endsWith("Start")?0x0B63:0x0B64;return new MethodVisitor(Opcodes.ASM9,next){public void visitCode(){super.visitCode();visitLdcInsn(parameter);visitVarInsn(Opcodes.FLOAD,0);visitMethodInsn(Opcodes.INVOKESTATIC,"dev/tplauncher/hud/runtime/FrameCounter","tuneFog","(IF)F",false);visitVarInsn(Opcodes.FSTORE,0);}};}
                    if(!method.equals(glfw?"glfwSwapBuffers":"swapBuffers")||!desc.equals(glfw?"(J)V":"()V"))return next;
                    return new MethodVisitor(Opcodes.ASM9,next){public void visitCode(){super.visitCode();if(glfw)visitVarInsn(Opcodes.LLOAD,0);else visitInsn(Opcodes.LCONST_0);visitMethodInsn(Opcodes.INVOKESTATIC,"dev/tplauncher/hud/runtime/FrameCounter","frame","(J)V",false);}};
                }
            },0);return writer.toByteArray();
        }catch(Throwable ignored){return null;}
    }
    static boolean fogAvailable(){return fogHook;}
    static void fog(boolean active,float strength){try{Class.forName("dev.tplauncher.hud.runtime.FrameCounter",false,null).getMethod("fog",boolean.class,float.class).invoke(null,active,strength);}catch(Throwable ignored){}}
    static double[] metrics(){try{return(double[])Class.forName("dev.tplauncher.hud.runtime.FrameCounter",false,null).getMethod("snapshot").invoke(null);}catch(Throwable ignored){return null;}}
}
