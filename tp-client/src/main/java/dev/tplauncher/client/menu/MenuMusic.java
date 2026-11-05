package dev.tplauncher.client.menu;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.sound.MovingSoundInstance;
import net.minecraft.client.sound.SoundInstance;
import net.minecraft.sound.SoundCategory;
import net.minecraft.sound.SoundEvent;
import net.minecraft.util.Identifier;

public final class MenuMusic {
    private static Track track;
    public static void tick(MinecraftClient client,boolean active){
        boolean desired=active&&MenuController.config.musicEnabled&&MenuController.config.musicVolume>0&&client.options.getSoundVolume(SoundCategory.MASTER)>0&&client.options.getSoundVolume(SoundCategory.MUSIC)>0;
        if(desired){client.getMusicTracker().stop();if(track==null||track.isDone()){track=new Track();client.getSoundManager().play(track);}}
        if(track!=null){track.target=desired?(float)MenuController.config.musicVolume:0;if(!desired&&track.getVolume()<0.002f){client.getSoundManager().stop(track);track=null;}}
    }
    public static void close(){if(track!=null){MinecraftClient.getInstance().getSoundManager().stop(track);track=null;}}
    private static final class Track extends MovingSoundInstance {
        private float target;
        Track(){super(SoundEvent.of(Identifier.of("tp_client","endless_dusk")),SoundCategory.MUSIC,SoundInstance.createRandom());volume=0.001f;repeat=true;repeatDelay=0;relative=true;attenuationType=SoundInstance.AttenuationType.NONE;}
        @Override public void tick(){volume+=(target-volume)*0.08f;if(!MinecraftClient.getInstance().isRunning())setDone();}
    }
    private MenuMusic(){}
}
