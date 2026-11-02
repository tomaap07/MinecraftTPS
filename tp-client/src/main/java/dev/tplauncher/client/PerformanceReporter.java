package dev.tplauncher.client;

import com.google.gson.JsonObject;
import net.minecraft.client.MinecraftClient;
import java.lang.management.ManagementFactory;
import java.nio.file.*;
import java.util.concurrent.*;

public final class PerformanceReporter {
    private static final ExecutorService WRITER=Executors.newSingleThreadExecutor(r->{Thread t=new Thread(r,"TP Client local metrics");t.setDaemon(true);return t;});
    private static final java.util.concurrent.atomic.AtomicBoolean busy=new java.util.concurrent.atomic.AtomicBoolean();
    private static long previousGc=-1;
    private static Object measuredWorld;
    private static long worldStarted;
    static long worldElapsed(Object world,long now){
        if(world!=measuredWorld){measuredWorld=world;worldStarted=now;previousGc=-1;}
        return world==null?0:Math.max(0,now-worldStarted)/1_000_000_000L;
    }
    public static void sample(MinecraftClient c){
        long elapsed=worldElapsed(c.player==null?null:c.world,System.nanoTime());
        if(!TpClient.config.metricsEnabled||!busy.compareAndSet(false,true))return;
        Runtime r=Runtime.getRuntime();JsonObject data=new JsonObject();
        data.addProperty("schema_version",1);data.addProperty("timestamp_ms",System.currentTimeMillis());data.addProperty("pid",ProcessHandle.current().pid());
        data.addProperty("elapsed_seconds",elapsed);data.addProperty("has_world",c.world!=null&&c.player!=null);data.addProperty("focused",c.isWindowFocused());
        data.addProperty("fps",c.getCurrentFps());if(TpClient.frames.count()>0)data.addProperty("frame_time_ms",TpClient.frames.average());
        data.addProperty("heap_used_mb",(r.totalMemory()-r.freeMemory())/1048576);data.addProperty("heap_max_mb",r.maxMemory()/1048576);
        data.addProperty("fps_limit",c.options.getMaxFps().getValue());
        long gc=ManagementFactory.getGarbageCollectorMXBeans().stream().mapToLong(bean->Math.max(0,bean.getCollectionTime())).sum();if(previousGc>=0)data.addProperty("gc_pause_ms",Math.max(0,gc-previousGc));previousGc=gc;
        try{WRITER.execute(()->{try{ConfigStore.writeText(TpClient.configDirectory.resolve("tp-client/performance.json"),data.toString());}catch(Exception ignored){}finally{busy.set(false);}});}catch(RejectedExecutionException ignored){busy.set(false);}
    }
    public static void close(){WRITER.shutdownNow();}
    private PerformanceReporter(){}
}
