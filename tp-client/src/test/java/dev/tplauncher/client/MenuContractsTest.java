package dev.tplauncher.client;

import dev.tplauncher.client.menu.*;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import java.nio.file.*;
import java.awt.image.BufferedImage;
import javax.imageio.ImageIO;
import java.io.IOException;
import static org.junit.jupiter.api.Assertions.*;

final class MenuContractsTest {
    @TempDir Path folder;
    @Test void titleButtonsStayWithinScreenAtSmallAndWideGuiScales(){
        for(int[] resolution:new int[][]{{320,180},{320,240},{360,230},{400,300},{426,240},{640,360},{800,600},{1280,720},{1680,720}}){
            for(String preset:new String[]{"balanced","center"}){
                MenuLayout l=MenuLayout.of(resolution[0],resolution[1],true,preset);
                assertTrue(l.x()>=0);assertTrue(l.x()+l.width()<=resolution[0]);
                for(int index=0;index<8;index++){assertTrue(l.y(index)>=0);assertTrue(l.y(index)+l.row()<=resolution[1]-24,"Bottom row clipped at "+resolution[0]+"x"+resolution[1]);}
                if(l.preview()){assertTrue(l.previewX()>=l.x()+l.width());assertTrue(l.previewX()+l.previewWidth()<=resolution[0]);assertTrue(l.previewY()>=0);assertTrue(l.previewY()+l.previewHeight()<=resolution[1]);}
            }
        }
    }
    @Test void unknownLayoutAndVolumeAreNormalized() throws Exception {
        MenuConfig config=MenuConfig.parse("{\"schemaVersion\":1,\"layout\":\"../../escape\",\"musicVolume\":42}");
        assertEquals("balanced",config.layout);assertEquals(1,config.musicVolume);
        assertThrows(IOException.class,()->MenuConfig.parse("{\"schemaVersion\":999}"));
        assertThrows(IOException.class,()->MenuConfig.parse("{broken"));
    }
    @Test void corruptSettingsAreBackedUpWithoutOverwritingOriginal() throws Exception {
        Path file=folder.resolve("menu.json");Files.writeString(file,"{broken");
        assertTrue(MenuConfig.load(file).enabled);assertEquals("{broken",Files.readString(file));
        try(var files=Files.list(folder)){assertTrue(files.anyMatch(p->p.getFileName().toString().startsWith("menu.json.invalid-")));}
    }
    @Test void safeConfigRoundTripRetainsAccessibilityAndAudioPreferences() throws Exception {
        MenuConfig config=new MenuConfig();config.reducedMotion=true;config.lowPower=true;config.musicVolume=.42;config.playerMotion=false;
        Path file=folder.resolve("menu.json");config.save(file);var read=MenuConfig.load(file);
        assertFalse(read.motion());assertFalse(read.playerMotion);assertEquals(.42,read.musicVolume);
    }
    @Test void backgroundRejectsUnsupportedFormatsAndExcessiveDimensions() throws Exception {
        assertFalse(MenuBackground.validDimensions(100000,10));assertFalse(MenuBackground.validDimensions(4096,4096));assertFalse(MenuBackground.validDimensions(0,0));assertTrue(MenuBackground.validDimensions(1920,1080));
        var image=new BufferedImage(64,32,BufferedImage.TYPE_INT_RGB);Path gif=folder.resolve("renamed.png");ImageIO.write(image,"GIF",gif.toFile());assertThrows(IOException.class,()->MenuBackground.normalize(gif));
        Path png=folder.resolve("source.png");ImageIO.write(image,"PNG",png.toFile());byte[] clean=MenuBackground.normalize(png);assertTrue(clean.length>32);assertEquals(64,ImageIO.read(new java.io.ByteArrayInputStream(clean)).getWidth());
    }
    @Test void originalLoopAndSoundDeclarationArePackaged() throws Exception {
        try(var audio=getClass().getResourceAsStream("/assets/tp_client/sounds/menu/endless_dusk.ogg")){assertNotNull(audio);assertArrayEquals(new byte[]{79,103,103,83},audio.readNBytes(4));}
        try(var json=getClass().getResourceAsStream("/assets/tp_client/sounds.json")){assertNotNull(json);assertTrue(new String(json.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8).contains("tp_client:menu/endless_dusk"));}
    }
}
