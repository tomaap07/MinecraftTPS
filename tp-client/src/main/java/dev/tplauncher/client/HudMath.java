package dev.tplauncher.client;

public final class HudMath {
    public static double finite(double value, double fallback, double min, double max) { return Double.isFinite(value) ? Math.max(min, Math.min(max, value)) : fallback; }
    public static int safeCoordinate(double value, int extent, int screen, int margin) { return (int)Math.max(margin, Math.min(Math.max(margin, screen-extent-margin), value)); }
    public static int snap(double value, int grid) { return (int)Math.round(value / Math.max(1, grid)) * Math.max(1, grid); }
    public static int percentage(int maximum, int damage) { return maximum<=0?0:(int)Math.round(100.0*Math.max(0,Math.min(maximum,maximum-damage))/maximum); }
    private HudMath() {}
}
