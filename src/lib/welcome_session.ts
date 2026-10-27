import { POSES, type Pose } from "./poses_catalog";
import { discoverSecret, randomFraction, rollStartupEgg, type EasterEgg } from "./easter_eggs";
import { equipmentForWelcome, type EquippedItems } from "./held_items";

export const HEROBRINE_SKIN = "/skins/tp-spectral.png";
export type WelcomeSession = Readonly<{ pose: Pose; equipment:EquippedItems; easterEgg: boolean; egg: EasterEgg | null }>;
let session: WelcomeSession | undefined;


export function getWelcomeSession(random: () => number = randomFraction): WelcomeSession {
  if (!session) {
    session = createWelcomeSession(random(), random(), random(), random(), random());
    if(session.egg)discoverSecret(session.egg.id);
  }
  return session;
}

export function createWelcomeSession(poseSample: number, appearanceSample: number, categorySample = .5, entrySample = .5, equipmentSample = .5): WelcomeSession {
  const index = Math.min(POSES.length - 1, Math.max(0, Math.floor((Number.isFinite(poseSample)?poseSample:.5) * POSES.length)));
  const gate=Number.isFinite(appearanceSample)&&appearanceSample>=0&&appearanceSample<=1?appearanceSample:1;
  const samples=[gate,categorySample,entrySample];let sampleIndex=0;
  const egg=rollStartupEgg(()=>samples[sampleIndex++]??.5);
  return Object.freeze({ pose: POSES[index], equipment:equipmentForWelcome(POSES[index],equipmentSample), easterEgg: egg?.variant==='herobrine', egg });
}
