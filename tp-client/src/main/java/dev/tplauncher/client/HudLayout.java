package dev.tplauncher.client;

import java.util.*;

public final class HudLayout {
    public record Rect(int x,int y,int width,int height) {
        public boolean contains(double px,double py){return px>=x&&py>=y&&px<x+width&&py<y+height;}
        public boolean intersects(Rect other,int gap){return x<other.x+other.width+gap&&x+width+gap>other.x&&y<other.y+other.height+gap&&y+height+gap>other.y;}
    }
    public record Request(String id,String anchor,double x,double y,int width,int height,double scale,boolean priority){}
    public record Placement(Rect bounds,double scale){}
    public record Result(Map<String,Placement> placements,int hidden,double compression){}
    private static final int MARGIN=6,GAP=4;

    public static Result arrange(List<Request> requests,int screenWidth,int screenHeight,List<Rect> protectedAreas){
        int width=Math.max(1,screenWidth),height=Math.max(1,screenHeight);
        List<Request> ordered=new ArrayList<>(requests);ordered.sort(Comparator.comparing((Request r)->!r.priority));
        Result best=new Result(Map.of(),requests.size(),1);
        for(double compression:new double[]{1,0.9,0.8,0.7,0.65}){
            List<Rect> occupied=new ArrayList<>(protectedAreas);Map<String,Placement> placements=new LinkedHashMap<>();
            for(Request request:ordered){
                double scale=Math.min(request.scale*compression,Math.max(0.1,(width-MARGIN*2.0)/Math.max(1,request.width)));
                int boxWidth=Math.max(1,(int)Math.ceil(request.width*scale)),boxHeight=Math.max(1,(int)Math.ceil(request.height*scale));
                int right=width-MARGIN-boxWidth,bottom=height-MARGIN-boxHeight;
                if(right<MARGIN||bottom<MARGIN)continue;
                double baseX=request.anchor.endsWith("right")?right:request.anchor.endsWith("center")?(width-boxWidth)/2.0:MARGIN;
                double baseY=request.anchor.startsWith("bottom")?bottom:MARGIN;
                int desiredX=HudMath.safeCoordinate(baseX+request.x,boxWidth,width,MARGIN),desiredY=HudMath.safeCoordinate(baseY+request.y,boxHeight,height,MARGIN);
                Rect found=findSpace(desiredX,desiredY,boxWidth,boxHeight,right,bottom,occupied);
                if(found!=null){placements.put(request.id,new Placement(found,scale));occupied.add(found);}
            }
            Result result=new Result(Collections.unmodifiableMap(placements),requests.size()-placements.size(),compression);
            if(result.hidden<best.hidden||best.placements.isEmpty())best=result;
            if(result.hidden==0)return result;
        }
        return best;
    }
    private static Rect findSpace(int x,int y,int width,int height,int right,int bottom,List<Rect> occupied){
        Rect desired=new Rect(x,y,width,height);if(free(desired,occupied))return desired;
        TreeSet<Integer> xs=new TreeSet<>(List.of(MARGIN,right,x)),ys=new TreeSet<>(List.of(MARGIN,bottom,y));
        for(Rect used:occupied){
            for(int px:new int[]{used.x-width-GAP,used.x+used.width+GAP})if(px>=MARGIN&&px<=right)xs.add(px);
            for(int py:new int[]{used.y-height-GAP,used.y+used.height+GAP})if(py>=MARGIN&&py<=bottom)ys.add(py);
        }
        Rect best=null;long cost=Long.MAX_VALUE;
        for(int py:ys)for(int px:xs){long next=(long)(px-x)*(px-x)+(long)(py-y)*(py-y);if(next>=cost)continue;Rect candidate=new Rect(px,py,width,height);if(free(candidate,occupied)){best=candidate;cost=next;}}
        return best;
    }
    private static boolean free(Rect candidate,List<Rect> occupied){for(Rect used:occupied)if(candidate.intersects(used,GAP))return false;return true;}
    public static Rect notificationArea(int width){
        int available=width-176;
        if(available<48)return new Rect(0,0,0,0);
        int panelWidth=Math.min(280,available);
        int x=Math.max(6,Math.min((width-panelWidth)/2,width-170-panelWidth));
        return new Rect(x,8,panelWidth,34);
    }
    public static List<Rect> safeAreas(int width,int height,boolean chatOpen){
        List<Rect> areas=new ArrayList<>();
        Rect notification=notificationArea(width);
        if(notification.width>0)areas.add(new Rect(notification.x,0,notification.width,48));
        areas.add(new Rect(Math.max(0,width-164),0,Math.min(164,width),56));
        areas.add(new Rect(Math.max(0,(width-210)/2),Math.max(0,height-65),Math.min(210,width),65));
        int chatHeight=chatOpen?155:72;
        areas.add(new Rect(0,Math.max(0,height-chatHeight),Math.min(chatOpen?330:width/2,width),chatHeight));
        return areas;
    }
    private HudLayout(){}
}
