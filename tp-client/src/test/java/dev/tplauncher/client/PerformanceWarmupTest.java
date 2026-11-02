package dev.tplauncher.client;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

final class PerformanceWarmupTest {
    @Test void timeInMenusNeverCountsAsWorldWarmupAndChangingWorldResets(){
        Object first=new Object(),second=new Object();
        assertEquals(0,PerformanceReporter.worldElapsed(null,300_000_000_000L));
        assertEquals(0,PerformanceReporter.worldElapsed(first,310_000_000_000L));
        assertEquals(30,PerformanceReporter.worldElapsed(first,340_000_000_000L));
        assertEquals(0,PerformanceReporter.worldElapsed(second,350_000_000_000L));
        assertEquals(60,PerformanceReporter.worldElapsed(second,410_000_000_000L));
        assertEquals(0,PerformanceReporter.worldElapsed(null,420_000_000_000L));
    }
}
