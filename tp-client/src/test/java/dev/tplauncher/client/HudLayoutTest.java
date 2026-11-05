package dev.tplauncher.client;

import org.junit.jupiter.api.Test;
import java.util.*;
import static org.junit.jupiter.api.Assertions.*;

final class HudLayoutTest {
    private static List<HudLayout.Request> allWidgets(double scale){
        List<HudLayout.Request> result=new ArrayList<>();
        for(int i=0;i<20;i++)result.add(new HudLayout.Request("w"+i,i%3==0?"top_right":"top_left",0,0,i==3?106:110,i==3?103:i==4?105:49,scale,false));
        return result;
    }
    private static void valid(HudLayout.Result result,int width,int height,List<HudLayout.Rect> reserved){
        List<HudLayout.Rect> boxes=new ArrayList<>();
        for(var placement:result.placements().values()){
            var box=placement.bounds();assertTrue(box.x()>=6);assertTrue(box.y()>=6);assertTrue(box.x()+box.width()<=width-6);assertTrue(box.y()+box.height()<=height-6);
            for(var other:boxes)assertFalse(box.intersects(other,0),"Widgets overlap: "+box+" / "+other);
            for(var area:reserved)assertFalse(box.intersects(area,0),"Widget covers protected game interface");
            boxes.add(box);assertTrue(placement.scale()>0&&Double.isFinite(placement.scale()));
        }
        assertEquals(20,result.placements().size()+result.hidden());
    }
    @Test void enablingEveryWidgetNeverOverlapsAtDifferentGuiScales(){
        for(int[] size:new int[][]{{1920,1080},{1280,720},{960,540},{640,360},{480,270},{320,240},{256,180}}){
            for(boolean chat:new boolean[]{false,true}){
                var reserved=HudLayout.safeAreas(size[0],size[1],chat);var result=HudLayout.arrange(allWidgets(1),size[0],size[1],reserved);
                valid(result,size[0],size[1],reserved);
                if(size[0]>=960)assertEquals(0,result.hidden(),"Full viewport can fit all widgets");
            }
        }
    }
    @Test void oversizedAndDraggedOffscreenWidgetsDegradeWithoutCoveringOthers(){
        var requests=allWidgets(3);requests.set(2,new HudLayout.Request("w2","top_left",10000,-10000,110,60,3,true));
        var reserved=HudLayout.safeAreas(480,270,false);var first=HudLayout.arrange(requests,480,270,reserved);
        valid(first,480,270,reserved);assertEquals(first,HudLayout.arrange(requests,480,270,reserved));
    }
    @Test void remainingVisibleAfterResizingDoesNotRequireEditingSavedCoordinates(){
        var requests=allWidgets(1);var before=List.copyOf(requests);
        for(int[] size:new int[][]{{640,360},{320,240},{1280,720}})valid(HudLayout.arrange(requests,size[0],size[1],HudLayout.safeAreas(size[0],size[1],false)),size[0],size[1],HudLayout.safeAreas(size[0],size[1],false));
        assertEquals(before,requests);
    }
    @Test void noticesUseTheReservedSpaceAndNeverCoverMinecraftToasts(){
        for(int width:new int[]{180,224,256,320,428,640,960,1920}){
            var notice=HudLayout.notificationArea(width);
            if(notice.width()==0)continue;
            assertTrue(notice.x()>=6);assertTrue(notice.width()<=280);
            var toast=new HudLayout.Rect(Math.max(0,width-164),0,Math.min(164,width),56);
            assertFalse(notice.intersects(toast,4));
            assertTrue(HudLayout.safeAreas(width,240,false).stream().anyMatch(area->area.contains(notice.x(),notice.y())&&area.contains(notice.x()+notice.width()-1,notice.y()+notice.height()-1)));
        }
    }
}
