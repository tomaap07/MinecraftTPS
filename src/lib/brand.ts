import { writable } from 'svelte/store';

export const BRAND_NAME = 'TP LAUNCHER';
const resting = { active: false, text: BRAND_NAME, hue: 280 };
const effect = writable(resting);
export const logoEffect = { subscribe: effect.subscribe };
let frameTimer: ReturnType<typeof setInterval> | undefined;

export function stopLogoEffect(): void {
  clearInterval(frameTimer);
  frameTimer = undefined;
  effect.set(resting);
}


export function startLogoEffect(): void {
  stopLogoEffect();
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!?%#@<>/';
  const reducedMotion = globalThis.matchMedia?.('(prefers-reduced-motion: reduce)').matches;
  let frame = 0;
  const update = () => {
    const text = [...BRAND_NAME].map((letter, index) => letter === ' ' ? ' '
      : (frame + index) % 5 === 0 ? letter : alphabet[Math.floor(Math.random() * alphabet.length)]).join('');
    effect.set({ active: true, text, hue: (280 + frame * 17) % 360 });
    frame++;
  };
  update();
  if (!reducedMotion) frameTimer = setInterval(update, 140);
}
