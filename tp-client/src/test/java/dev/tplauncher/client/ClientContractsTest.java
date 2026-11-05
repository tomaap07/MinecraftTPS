package dev.tplauncher.client;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import java.nio.file.*;
import java.nio.ByteBuffer;
import static org.junit.jupiter.api.Assertions.*;

final class ClientContractsTest {
    @TempDir Path root;
    @Test void configRoundTripPreservesIndividualWidgetSettings() throws Exception {
        ClientConfig c=new ClientConfig();var armor=c.widgets.get("armor");armor.scale=1.7;armor.opacity=0.4;armor.x=87;armor.locked=true;
        Path file=root.resolve("tp-client.json");ConfigStore.write(file,c);ClientConfig loaded=ConfigStore.read(file);
        assertEquals(1.7,loaded.widgets.get("armor").scale);assertEquals(0.4,loaded.widgets.get("armor").opacity);
        assertEquals(87,loaded.widgets.get("armor").x);assertTrue(loaded.widgets.get("armor").locked);assertEquals(1,loaded.widgets.get("fps").scale);
    }
    @Test void partialLauncherConfigReceivesDefaultsWithoutEnablingCosmetics() throws Exception {
        Path file=root.resolve("tp-client.json");Files.writeString(file,"{\"schemaVersion\":1,\"mode\":\"minimal\"}");
        ClientConfig loaded=ConfigStore.read(file);assertEquals(20,loaded.widgets.size());assertFalse(loaded.cosmeticsEnabled);assertFalse(loaded.metricsEnabled);
    }
    @Test void corruptConfigIsPreservedAndGameGetsSafeDefaults() throws Exception {
        Path file=root.resolve("tp-client.json");Files.writeString(file,"{broken");ClientConfig c=ConfigStore.loadSafely(file);
        assertEquals(20,c.widgets.size());assertEquals("{broken",Files.readString(file));try(var paths=Files.list(root)){assertTrue(paths.anyMatch(p->p.toString().contains(".invalid-")));}
    }
    @Test void futureSchemaNeverSilentlyParses() throws Exception {
        Path file=root.resolve("future.json");Files.writeString(file,"{\"schemaVersion\":99}");assertThrows(java.io.IOException.class,()->ConfigStore.read(file));
    }
    @Test void normalizationRejectsPoisonedNumbersAndUnknownWidgets(){
        ClientConfig c=new ClientConfig();var w=c.widgets.get("fps");w.scale=Double.NaN;w.opacity=-40;w.x=Double.POSITIVE_INFINITY;w.anchor="../../";c.widgets.put("not-a-module",new ClientConfig.Widget());c.normalize();
        assertEquals(1,w.scale);assertEquals(0.1,w.opacity);assertEquals(0,w.x);assertEquals("top_left",w.anchor);assertFalse(c.widgets.containsKey("not-a-module"));
    }
    @Test void presetsAreDistinctAndStreamingHidesSensitiveCoordinates(){
        ClientConfig c=new ClientConfig();c.preset("PvP");assertTrue(c.widgets.get("cps").enabled);assertFalse(c.widgets.get("coordinates").enabled);
        c.preset("Streaming");assertTrue(c.streamer);assertTrue(c.hideCoordinates);assertFalse(c.widgets.get("target").enabled);
    }
    @Test void profileFileNamesCannotEscapeDirectory() throws Exception {
        for(String name:new String[]{"../outside","a/b","C:\\outside","","a".repeat(49)})assertThrows(java.io.IOException.class,()->ConfigStore.profilePath(root,name));
        assertEquals(root.resolve("tp-client-profiles/my-hud_1.json"),ConfigStore.profilePath(root,"my-hud_1"));
    }
    @Test void oversizedProfileIsRejectedBeforeParsing() throws Exception {
        Path file=root.resolve("large.json");Files.write(file,new byte[ConfigStore.MAX_BYTES+1]);assertThrows(java.io.IOException.class,()->ConfigStore.read(file));
    }
    @Test void widgetBoundsStayOnScreenAfterResize(){
        assertEquals(4,HudMath.safeCoordinate(-200,100,320,4));assertEquals(216,HudMath.safeCoordinate(900,100,320,4));assertEquals(4,HudMath.safeCoordinate(200,900,320,4));assertEquals(16,HudMath.snap(15,4));
    }
    @Test void durabilityReportsRemainingPercentageWithoutOverflow(){assertEquals(100,HudMath.percentage(100,0));assertEquals(75,HudMath.percentage(100,25));assertEquals(0,HudMath.percentage(100,120));assertEquals(0,HudMath.percentage(0,0));}
    @Test void frameMetricsBoundHistoryAndRejectLargePauseSamples(){
        FrameMetrics f=new FrameMetrics();long start=1_000_000_000L;for(int i=0;i<500;i++)f.frame(start+i*10_000_000L);assertEquals(240,f.count());assertEquals(10,f.average(),0.01);assertEquals(10,f.percentile(.95),0.01);
        f.frame(start+20_000_000_000L);assertEquals(10,f.average(),0.01);f.reset();assertEquals(0,f.count());
    }
    @Test void cosmeticConfigRejectsRemoteAndEscapingCapeReferences() throws Exception {
        for(String name:new String[]{"../private.png","https://example.com/a.png","C:\\a.png","a/b.png"})assertThrows(java.io.IOException.class,()->Cosmetics.parseState("{\"schemaVersion\":1,\"equippedCape\":"+new com.google.gson.Gson().toJson(name)+"}"));
        assertEquals("own.png",Cosmetics.parseState("{\"schemaVersion\":1,\"equippedCape\":\"own.png\"}").equippedCape);
    }
    @Test void capePngDimensionsAreCheckedBeforeDecode(){
        byte[] png=new byte[40];byte[] magic={(byte)137,80,78,71,13,10,26,10};System.arraycopy(magic,0,png,0,8);ByteBuffer b=ByteBuffer.wrap(png);b.putInt(8,13).putInt(12,0x49484452).putInt(16,64).putInt(20,32);assertTrue(Cosmetics.validCape(png));b.putInt(16,2147483647);assertFalse(Cosmetics.validCape(png));assertFalse(Cosmetics.validCape(new byte[262145]));
    }
}
