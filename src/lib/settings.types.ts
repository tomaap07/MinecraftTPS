export type LauncherSettings = {
  language: string;
  launch_behavior: 'keep' | 'minimize' | 'tray' | 'close';
  data_root: string;
  discord_presence: boolean;
  discord_application_id: string;
  automatic_updates: boolean;
  first_run_completed: boolean;
  ui_scale_percent: number;
  launcher_width: number;
  launcher_height: number;
  launcher_maximized: boolean;
  resolution_width: number;
  resolution_height: number;
  fullscreen: boolean;
  ram_gb: number;
  fps_boost: boolean;
  gc_mode?: 'g1gc' | 'zgc';
};
export type ThemePreset = { id: string; name: string; primary: string; secondary: string; bgBase: string };
export type AppearanceSettings = {
  selectedThemeId: string;
  primaryColor: string;
  secondaryColor: string;
  bgBaseColor: string;
  transparencyPreset: 'fantasma' | 'equilibrado' | 'lectura' | 'solido';
  bgOpacity: number;
  menuOpacity: number;
  barOpacity: number;
  cardOpacity: number;
  blurPx: number;
  backgroundUrl?: string;
};
export type JavaRuntime = { major: number; installed: boolean; enabled: boolean; path: string };
export type AccountSummary = { id: string; name: string; kind: 'microsoft' | 'offline'; active: boolean; avatar_url?: string; status?: string };
export type QuickFolder = 'root' | 'screenshots' | 'saves' | 'resourcepacks' | 'shaderpacks';
export type SettingsTab = 'general' | 'display' | 'language' | 'appearance' | 'minecraft' | 'java' | 'accounts';
export type Result = void | string | Promise<void | string>;
export const DEFAULT_LAUNCHER_SETTINGS: LauncherSettings = {
  language: 'es-419', launch_behavior: 'keep', data_root: '', discord_presence: false,
  discord_application_id: '', automatic_updates: true, first_run_completed: false,
  ui_scale_percent: 100, launcher_width: 1280, launcher_height: 820, launcher_maximized: false,
  resolution_width: 1280, resolution_height: 720, fullscreen: false, ram_gb: 4, fps_boost: false, gc_mode: 'g1gc'
};
