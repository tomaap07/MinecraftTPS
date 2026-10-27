package dev.tplauncher.bridge;
import java.io.*;import java.nio.file.*;import java.util.zip.GZIPInputStream;

final class WorldTime{
 static Long read(Path world){try{
  Path file=world.resolve("level.dat");if(!Files.isRegularFile(file)||Files.isSymbolicLink(file)||Files.size(file)>8*1024*1024||System.currentTimeMillis()-Files.getLastModifiedTime(file).toMillis()>15*60*1000)return null;
  ByteArrayOutputStream out=new ByteArrayOutputStream();try(InputStream in=new GZIPInputStream(Files.newInputStream(file))){byte[] chunk=new byte[4096];int n;while((n=in.read(chunk))!=-1){if(out.size()+n>8*1024*1024)return null;out.write(chunk,0,n);}}
  WorldTime parser=new WorldTime();DataInputStream in=new DataInputStream(new ByteArrayInputStream(out.toByteArray()));int tag=in.readUnsignedByte();if(tag!=10)return null;in.readUTF();parser.tag(in,tag,"",0,false);return parser.day!=null?parser.day:parser.time;
 }catch(Exception ignored){return null;}}
 private Long day,time;private int tags;
 private void tag(DataInputStream in,int type,String key,int depth,boolean data)throws IOException{
  if(depth>32||++tags>100000)throw new IOException("NBT bounds");
  switch(type){case 1:in.readByte();break;case 2:in.readShort();break;case 3:in.readInt();break;case 4:{long value=in.readLong();if(data&&depth==2&&value>=0){if(key.equals("DayTime"))day=value;else if(key.equals("Time"))time=value;}break;}case 5:in.readFloat();break;case 6:in.readDouble();break;
   case 7:skip(in,size(in,1));break;case 8:in.readUTF();break;case 9:{int child=in.readUnsignedByte(),n=size(in,1);if(n>100000)throw new IOException();for(int i=0;i<n;i++)tag(in,child,"",depth+1,false);break;}
   case 10:while(true){int child=in.readUnsignedByte();if(child==0)break;String name=in.readUTF();tag(in,child,name,depth+1,depth==0&&name.equals("Data")||data);}break;
   case 11:skip(in,size(in,4));break;case 12:skip(in,size(in,8));break;default:throw new IOException("NBT type");
  }
 }
 private static int size(DataInputStream in,int width)throws IOException{int count=in.readInt();if(count<0||count>8*1024*1024/width)throw new IOException();return count*width;}
 private static void skip(DataInputStream in,int bytes)throws IOException{if(in.skipBytes(bytes)!=bytes)throw new EOFException();}
}
