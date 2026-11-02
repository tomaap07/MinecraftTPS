package dev.tplauncher.client;

import java.util.Arrays;

public final class FrameMetrics {
    private final double[] frames=new double[240];private int size,index;private long last;
    public void frame(long now) {if(last!=0){double ms=(now-last)/1_000_000.0;if(ms>0&&ms<=1000){frames[index++%frames.length]=ms;size=Math.min(size+1,frames.length);}}last=now;}
    public double average(){if(size==0)return 0;double sum=0;for(int i=0;i<size;i++)sum+=frames[i];return sum/size;}
    public double percentile(double p){if(size==0)return 0;double[] copy=Arrays.copyOf(frames,size);Arrays.sort(copy);return copy[(int)Math.max(0,Math.min(size-1,Math.ceil(p*size)-1))];}
    public int count(){return size;}
    public void reset(){last=0;size=0;index=0;}
}
