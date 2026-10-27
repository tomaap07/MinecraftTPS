import { derived, get, writable } from 'svelte/store';
import { english, partialCatalogs } from './locales/catalog';

export type Language = { code: string; name: string; region: string; completed: number; translated: number; total: number };
const source = Object.fromEntries(Object.keys(english).map((key) => [key, key]));
const catalogs: Record<string, Record<string, string>> = {
  'es-419': source,
  'es-ES': { ...source, 'Carpeta del juego': 'Carpeta del juego' },
  'en-US': english,
  'en-GB': { ...english, 'Modo Boost FPS': 'FPS Boost mode', 'Colores personalizados': 'Custom colours', 'Acento principal': 'Primary accent', 'Acento secundario': 'Secondary accent', 'Optimización JVM': 'JVM optimisation', 'Optimización JVM con G1GC · puede reducir pausas': 'JVM optimisation with G1GC · may reduce pauses' },
  ...partialCatalogs
};
const total = Object.keys(source).length;
export const LANGUAGES: Language[] = [
  ['es-419', 'Español (Latinoamérica)', 'Latinoamérica'], ['es-ES', 'Español (España)', 'España'], ['en-US', 'English (US)', 'United States'], ['en-GB', 'English (UK)', 'United Kingdom'], ['pt-BR', 'Português (Brasil)', 'Brasil'], ['fr-FR', 'Français', 'France'], ['de-DE', 'Deutsch', 'Deutschland'], ['it-IT', 'Italiano', 'Italia'], ['ru-RU', 'Русский', 'Россия'], ['ja-JP', '日本語', '日本'], ['ko-KR', '한국어', '대한민국'], ['zh-CN', '中文（简体）', '中国大陆'], ['ca-ES', 'Català', 'Catalunya'], ['eo-UY', 'Esperanto', 'Internacia'], ['en-PT', 'Pirate Speak', 'The Seven Seas'], ['lol-US', 'LOLCAT', 'Internet']
].map(([code, name, region]) => {
  const translated = Object.keys(source).filter((key) => typeof catalogs[code]?.[key] === 'string' && catalogs[code][key].length > 0).length;
  return { code, name, region, translated, total, completed: Math.floor(translated * 100 / total) };
});

export function normalizeLocale(value: string): string {
  if (catalogs[value]) return value;
  const normalized = value.toLowerCase().replace('_', '-');
  if (normalized.startsWith('es')) return normalized === 'es-es' ? 'es-ES' : 'es-419';
  if (normalized.startsWith('en')) return normalized === 'en-gb' ? 'en-GB' : 'en-US';
  return LANGUAGES.find((language) => language.code.toLowerCase().split('-')[0] === normalized.split('-')[0])?.code ?? 'es-419';
}
export const locale = writable('es-419');
export function setLocale(value: string): void {
  const selected = normalizeLocale(value);
  locale.set(selected);
  if (typeof document !== 'undefined') document.documentElement.lang = selected;
}
export function getLanguage(): string { return get(locale); }
export function detectSystemLanguage(): string {
  return normalizeLocale(typeof navigator === 'undefined' ? 'es-419' : navigator.languages?.[0] ?? navigator.language);
}

export function translate(message: string, code = get(locale), values?: Record<string, string | number>): string {
  let result = catalogs[code]?.[message] ?? message;
  if (values) for (const [key, value] of Object.entries(values)) result = result.replaceAll(`{${key}}`, String(value));
  return result;
}
export const t = derived(locale, (code) => (message: string, values?: Record<string, string | number>) => translate(message, code, values));


export function translateDiagnostic(message: string, code = get(locale)): string {
  let result = translate(message, code);
  if (result !== message || code.startsWith('es')) return result;
  const replacements: Array<[RegExp, string]> = [
    [/^Error: /, `${translate('Error', code)}: `], [/^Advertencia: /, `${translate('Advertencia', code)}: `],
    [/Descargando Java (\d+)(?: Portable)?/g, code.startsWith('en') ? 'Downloading Java $1' : `${translate('Descargar runtime', code)} · Java $1`],
    [/Java (\d+) (?:no está instalado|no encontrado)/g, `Java $1 · ${translate('No instalado', code)}`],
    [/Se requiere Java (\d+)(?: o superior)?/g, code.startsWith('en') ? 'Java $1 or newer is required' : `Java $1 · ${translate('Recomendado', code)}`],
    [/El juego terminó (?:con código|con el código|con exit code) (-?\d+)/g, code.startsWith('en') ? 'The game exited with code $1' : `${translate('Estado', code)}: $1`],
    [/Descargando Minecraft/g, code.startsWith('en') ? 'Downloading Minecraft' : `${translate('Descargar', code)} Minecraft`],
    [/Comprobando biblioteca/g, code.startsWith('en') ? 'Checking library' : translate('Dependencias', code)],
    [/Descargando recursos de Minecraft/g, code.startsWith('en') ? 'Downloading Minecraft assets' : `${translate('Descargar', code)} Minecraft`],
    [/Sesión recuperada automáticamente/g, code.startsWith('en') ? 'Session restored automatically' : translate('Sesión guardada', code)],
    [/No se pudo iniciar el juego/g, translate('No se pudo iniciar Minecraft.', code)],
    [/Error de descarga/g, translate('No se pudo descargar el archivo.', code)],
    [/Error de conexión/g, translate('Conexión de red no disponible.', code)]
  ];
  for (const [pattern, replacement] of replacements) result = result.replace(pattern, replacement);
  return result;
}


export function translateUi(node: HTMLElement): { destroy(): void } {
  const originalText = new WeakMap<Text, { source: string; rendered: string }>();
  const originalAttributes = new WeakMap<Element, Map<string, { source: string; rendered: string }>>();
  let language = get(locale);
  let pending = false;
  let alive = true;
  const excluded = (element: Element | null) => !!element?.closest('script,style,textarea,input,code,pre,[data-no-translate],[contenteditable="true"]');
  const observer = new MutationObserver(() => schedule());
  const run = () => {
    if (!alive) return;
    pending = false;
    observer.disconnect();
    const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT);
    let current = walker.nextNode();
    while (current) {
      const text = current as Text;
      if (!excluded(text.parentElement) && text.data.trim()) {
        let entry = originalText.get(text);
        if (!entry || text.data !== entry.rendered) entry = { source: text.data, rendered: text.data };
        const trimmed = entry.source.trim();
        const translated = text.parentElement?.closest('[data-diagnostic]') ? translateDiagnostic(trimmed, language) : translate(trimmed, language);
        entry.rendered = entry.source.replace(trimmed, translated);
        if (text.data !== entry.rendered) text.data = entry.rendered;
        originalText.set(text, entry);
      }
      current = walker.nextNode();
    }
    for (const element of [node, ...node.querySelectorAll<HTMLElement>('[title],[aria-label],[placeholder]')]) {
      if (element.closest('[data-no-translate]')) continue;
      const attributes = originalAttributes.get(element) ?? new Map();
      for (const name of ['title', 'aria-label', 'placeholder']) {
        const value = element.getAttribute(name);
        if (!value) continue;
        let entry = attributes.get(name);
        if (!entry || value !== entry.rendered) entry = { source: value, rendered: value };
        entry.rendered = translate(entry.source, language);
        if (value !== entry.rendered) element.setAttribute(name, entry.rendered);
        attributes.set(name, entry);
      }
      originalAttributes.set(element, attributes);
    }
    observer.observe(node, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ['title', 'aria-label', 'placeholder'] });
  };
  const schedule = () => { if (!pending && alive) { pending = true; queueMicrotask(run); } };
  const unsubscribe = locale.subscribe((value) => { language = value; schedule(); });
  schedule();
  return { destroy() { alive = false; observer.disconnect(); unsubscribe(); } };
}
