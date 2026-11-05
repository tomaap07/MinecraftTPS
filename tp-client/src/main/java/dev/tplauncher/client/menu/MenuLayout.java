package dev.tplauncher.client.menu;

public record MenuLayout(int x,int width,int top,int row,int gap,int previewX,int previewY,int previewWidth,int previewHeight,boolean preview) {
    public static MenuLayout of(int width,int height,boolean preview,String layout){
        boolean show=preview&&width>=360&&height>=230;
        int gap=height<220?2:3;
        int row=Math.max(9,Math.min(23,(height-68-gap*7)/8));
        int buttonWidth=Math.max(100,Math.min(208,show?(width-60)/2:width-36));
        int center=width/2-(show&&"balanced".equals(layout)?Math.min(70,width/9):0);
        int x=Math.max(12,Math.min(width-buttonWidth-12,center-buttonWidth/2));
        int controls=row*8+gap*7;
        int top=Math.min(Math.max(32,(height-controls)/2+8),Math.max(6,height-controls-28));
        int previewWidth=Math.max(84,Math.min(150,width-x-buttonWidth-24));
        int previewHeight=Math.max(130,Math.min(height-70,240));
        int previewX=Math.min(width-previewWidth-10,x+buttonWidth+12);
        return new MenuLayout(x,buttonWidth,top,row,gap,previewX,(height-previewHeight)/2-4,previewWidth,previewHeight,show);
    }
    public int y(int index){return top+index*(row+gap);}
}
