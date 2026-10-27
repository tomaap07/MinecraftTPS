package dev.tplauncher.hud.runtime;
import java.util.*;

public final class FrameCounter {
    private static volatile boolean fogActive;private static volatile float fogStrength;
    public static void fog(boolean active,float strength){fogActive=active;fogStrength=Math.max(0,Math.min(1,strength));}
    public static float tuneFog(int parameter,float value){if(!fogActive||fogStrength<=0)return value;if(parameter==0x0B62)return value*(1-fogStrength);if(parameter==0x0B63||parameter==0x0B64)return value+fogStrength*(parameter==0x0B63?1000000f:10000000f);return value;}
    private static final Map<Long,Window> WINDOWS=new LinkedHashMap<Long,Window>();
    private static final class Window {long last;final ArrayDeque<Long> times=new ArrayDeque<Long>();}
    public static synchronized void frame(long handle){
        try{long now=System.nanoTime();Window w=WINDOWS.get(handle);if(w==null){if(WINDOWS.size()>=8)WINDOWS.remove(WINDOWS.keySet().iterator().next());w=new Window();WINDOWS.put(handle,w);}
            w.last=now;w.times.add(now);while(!w.times.isEmpty()&&now-w.times.peek()>1000000000L)w.times.remove();
        }catch(Throwable ignored){ }
    }
    public static synchronized double[] snapshot(){
        long now=System.nanoTime();Window best=null;for(Window w:WINDOWS.values())if(now-w.last<2000000000L&&(best==null||w.times.size()>best.times.size()))best=w;
        if(best==null||best.times.size()<2)return null;
        while(!best.times.isEmpty()&&now-best.times.peek()>1000000000L)best.times.remove();if(best.times.size()<2)return null;
        double fps=(best.times.size()-1)*1000000000.0/(best.times.peekLast()-best.times.peekFirst());
        return new double[]{fps};
    }
}
