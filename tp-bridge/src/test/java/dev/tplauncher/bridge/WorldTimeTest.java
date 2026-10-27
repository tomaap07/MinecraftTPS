package dev.tplauncher.bridge;
import java.io.*;import java.nio.file.*;import java.util.zip.GZIPOutputStream;

public final class WorldTimeTest{
 public static void main(String[] args)throws Exception{
  Path folder=Paths.get(args[0]);Files.createDirectories(folder);Path file=folder.resolve("level.dat");
  try(DataOutputStream out=new DataOutputStream(new GZIPOutputStream(Files.newOutputStream(file)))){out.writeByte(10);out.writeUTF("");out.writeByte(10);out.writeUTF("Data");out.writeByte(4);out.writeUTF("Time");out.writeLong(96000);out.writeByte(4);out.writeUTF("DayTime");out.writeLong(72000);out.writeByte(10);out.writeUTF("Nested");out.writeByte(4);out.writeUTF("DayTime");out.writeLong(1);out.writeByte(0);out.writeByte(0);out.writeByte(0);}
  if(!Long.valueOf(72000).equals(WorldTime.read(folder)))throw new AssertionError("Wrong world/day or nested metadata override");
  Files.setLastModifiedTime(file,java.nio.file.attribute.FileTime.fromMillis(System.currentTimeMillis()-16*60*1000));if(WorldTime.read(folder)!=null)throw new AssertionError("Stale save was accepted");
  Files.write(file,new byte[]{1,2,3});if(WorldTime.read(folder)!=null)throw new AssertionError("Invalid save accepted");
  try(OutputStream out=new GZIPOutputStream(Files.newOutputStream(file))){byte[] bytes=new byte[8192];for(int i=0;i<1100;i++)out.write(bytes);}if(WorldTime.read(folder)!=null)throw new AssertionError("Decompression limit ignored");
  System.out.println("Active-save fallback: NBT selection, stale/malformed saves and decompression bounds verified.");
 }
}
