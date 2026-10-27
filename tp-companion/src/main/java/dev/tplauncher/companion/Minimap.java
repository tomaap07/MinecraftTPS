package dev.tplauncher.companion;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.block.MapColor;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.Heightmap;
import java.util.Arrays;


public final class Minimap {
    public static com.google.gson.JsonObject bridge(){
        if(world==null)return null;byte[] rgb=new byte[24*24*3];int at=0;
        for(int color:colors){rgb[at++]=(byte)(color>>>16);rgb[at++]=(byte)(color>>>8);rgb[at++]=(byte)color;}
        var data=new com.google.gson.JsonObject();data.addProperty("pixels",java.util.Base64.getEncoder().encodeToString(rgb));data.addProperty("origin_x",originX);data.addProperty("origin_z",originZ);return data;
    }
    private static final int SIDE=24,STEP=4;
    private static final int[] colors=new int[SIDE*SIDE];
    private static Object world;
    private static int originX,originZ;
    public static void update(MinecraftClient client){
        if(client.world==null||client.player==null){world=null;Arrays.fill(colors,0);return;}
        if(!Profile.get().enabled("minimap")){world=null;return;}
        world=client.world;originX=client.player.getBlockX()-SIDE*STEP/2;originZ=client.player.getBlockZ()-SIDE*STEP/2;
        BlockPos.Mutable pos=new BlockPos.Mutable();
        for(int z=0;z<SIDE;z++)for(int x=0;x<SIDE;x++){
            int bx=originX+x*STEP,bz=originZ+z*STEP,index=z*SIDE+x;colors[index]=0xFF24212D;
            if(!client.world.isChunkLoaded(bx>>4,bz>>4))continue;
            int top=client.world.getTopY(Heightmap.Type.WORLD_SURFACE,bx,bz)-1;
            pos.set(bx,top,bz);
            int abgr=client.world.getBlockState(pos).getMapColor(client.world,pos).getRenderColor(MapColor.Brightness.NORMAL);
            colors[index]=0xFF000000|(abgr&0xFF00)|((abgr&0xFF)<<16)|((abgr>>>16)&0xFF);
        }
    }
    public static void render(DrawContext context,MinecraftClient client,int x,int y){
        if(world!=client.world||client.player==null)return;
        for(int row=0;row<SIDE;row++)for(int col=0;col<SIDE;col++)context.fill(x+col*4,y+row*4,x+col*4+4,y+row*4+4,colors[row*SIDE+col]);
        int px=Math.clamp((int)(client.player.getX()-originX),1,94),pz=Math.clamp((int)(client.player.getZ()-originZ),1,94);
        context.fill(x+px-2,y+pz-2,x+px+3,y+pz+3,0xFF0B0B0F);
        context.fill(x+px-1,y+pz-1,x+px+2,y+pz+2,0xFFF4DFFF);
        double yaw=Math.toRadians(client.player.getYaw());
        int dx=(int)Math.round(-Math.sin(yaw)*5),dz=(int)Math.round(Math.cos(yaw)*5);
        context.fill(x+px+dx-1,y+pz+dz-1,x+px+dx+1,y+pz+dz+1,0xFFFFA6EE);
    }
}
