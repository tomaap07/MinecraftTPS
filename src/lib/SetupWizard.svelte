<script lang="ts">
  import JavaRuntimeIcon from "./JavaRuntimeIcon.svelte";
  import BrandWordmark from "./BrandWordmark.svelte";
  import ServiceLogo from "./ServiceLogo.svelte";
  import { onMount } from 'svelte';
  import UiIcon from './UiIcon.svelte';
  import { LANGUAGES, detectSystemLanguage, setLocale, t } from './i18n';
  import type { LauncherSettings, AppearanceSettings, ThemePreset, JavaRuntime, Result } from './settings.types';
  let {
    settings, onchange, appearance, onappearance, themes, runtimes = [], totalMemoryGb = 16,
    onscanjava, oninstalljava, onconnectmicrosoft, oncomplete, accountName = ''
  }: {
    settings: LauncherSettings; onchange: (patch: Partial<LauncherSettings>) => Result;
    appearance: AppearanceSettings; onappearance: (patch: Partial<AppearanceSettings>) => Result;
    themes: ThemePreset[]; runtimes?: JavaRuntime[]; totalMemoryGb?: number;
    onscanjava: () => Result; oninstalljava: (major: number) => Result;
    onconnectmicrosoft: () => Result; oncomplete: () => Result; accountName?: string;
  } = $props();
  let step = $state(1);
  let search = $state('');
  let busy = $state('');
  let message = $state('');
  let failed = $state(false);
  let scanned = $state(false);
  let shell: HTMLDivElement;
  let filteredLanguages = $derived(LANGUAGES.filter((language) => `${language.name} ${language.code} ${language.region}`.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase())));
  let recommendedRam = $derived(Math.max(2, Math.min(8, Math.floor((totalMemoryGb || 8) / 2))));
  let memoryLimit = $derived(Math.max(2, Math.min(128, Math.floor(totalMemoryGb || 16))));
  const requestedMajors = [8, 17, 21, 25];
  const steps = ['Selecciona tu idioma', 'Java y memoria', 'Tu estilo y tu cuenta'];
  async function perform(key: string, action: () => Result): Promise<boolean> {
    if (busy) return false;
    busy = key; message = ''; failed = false;
    try { const result = await action(); if (typeof result === 'string') message = result; return true; }
    catch (error) { failed = true; message = String(error); return false; }
    finally { busy = ''; }
  }
  async function selectLanguage(code: string) {
    setLocale(code);
    await perform('language', () => onchange({ language: code }));
  }
  async function scan() {
    scanned = true;
    await perform('scan', onscanjava);
  }
  async function advance() {
    if (busy) return;
    message = '';
    if (step < 3) { step += 1; if (step === 2 && !scanned) await scan(); }
    else await perform('complete', oncomplete);
  }
  async function installMissing() {
    await perform('install', async () => {
      for (const major of requestedMajors) {
        if (!runtimes.some((runtime) => runtime.major === major && runtime.installed)) {
          message = `${$t('Descargar runtime')} · Java ${major}`;
          await oninstalljava(major);
        }
      }
      await onscanjava();
      return $t('Configuración lista');
    });
  }
  async function chooseTheme(theme: ThemePreset) {
    await perform('theme', () => onappearance({ selectedThemeId: theme.id, primaryColor: theme.primary, secondaryColor: theme.secondary, bgBaseColor: theme.bgBase }));
  }
  function trapFocus(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;
    const focusable = [...shell.querySelectorAll<HTMLElement>('button:not(:disabled),input:not(:disabled),select:not(:disabled),[tabindex="0"]')].filter((element) => element.getClientRects().length > 0);
    const first = focusable[0], last = focusable[focusable.length - 1];
    if (!first) { event.preventDefault(); shell.focus(); return; }
    if (event.shiftKey && (document.activeElement === first || document.activeElement === shell)) { event.preventDefault(); last.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  }
  onMount(() => {
    const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    shell.querySelector<HTMLInputElement>('input')?.focus();
    if (!settings.language) void selectLanguage(detectSystemLanguage());
    return () => previousFocus?.focus();
  });
</script>

<div class="wizard-shell" bind:this={shell} role="dialog" aria-modal="true" aria-labelledby="wizard-heading" tabindex="-1" onkeydown={trapFocus}>
  <div class="ambient-glow"></div>
  <div class="wizard-window">
    <aside class="wizard-rail"><div class="brand"><BrandWordmark/></div><div class="rail-copy"><span class="eyebrow">{$t('Bienvenido a TP LAUNCHER')}</span><h1>{$t('Tu próxima aventura empieza aquí.')}</h1><p>{$t('Prepara tu próxima aventura en tres pasos.')}</p></div><ol>{#each steps as title, index}<li class:current={step === index + 1} class:done={step > index + 1}><span class="step-number">{index + 1}</span><div><small>{$t('Paso')} 0{index + 1}</small><strong>{$t(title)}</strong></div></li>{/each}</ol><small class="rail-foot">{$t('Puedes ajustar estas opciones más tarde en Ajustes.')}</small></aside>
    <section class="wizard-body"><header><span class="step-count">{$t('Paso')} {step} / 3</span><h2 id="wizard-heading">{$t(steps[step - 1])}</h2><div class="step-track">{#each steps as _, index}<span class:active={index < step}></span>{/each}</div></header>
      <div class="step-content">
        {#if step === 1}
          <div class="language-tools"><input type="search" bind:value={search} placeholder={$t('Buscar idioma o región…')} aria-label={$t('Buscar idioma o región…')} /><button disabled={!!busy} onclick={() => selectLanguage(detectSystemLanguage())}><UiIcon name="refresh" size={15} />{$t('Detectar idioma del sistema')}</button></div>
          <div class="language-table"><table><thead><tr><th>{$t('Idioma')}</th><th>{$t('Región / Código')}</th><th>{$t('Completado')}</th></tr></thead><tbody>{#each filteredLanguages as language}<tr class:chosen={settings.language === language.code}><td><button disabled={!!busy} class="language-button" onclick={() => selectLanguage(language.code)}><strong data-no-translate>{language.name}</strong>{#if settings.language === language.code}<UiIcon name="arrow" size={15} />{/if}</button></td><td data-no-translate>{language.region}<code>{language.code}</code></td><td><div class="coverage"><span><i style:width={`${language.completed}%`}></i></span><strong>{language.completed}%</strong></div></td></tr>{/each}</tbody></table>{#if !filteredLanguages.length}<p class="empty">{$t('Sin resultados')}</p>{/if}</div><small class="catalog-note">{$t('Cobertura del catálogo traducible. Los textos pendientes se muestran en español.')}</small>
        {:else if step === 2}
          <div class="section-top"><div><h3>{$t('Motores Java')}</h3><p>{$t('El launcher selecciona un runtime compatible según el manifiesto de cada versión.')}</p></div><button disabled={!!busy} onclick={scan}><UiIcon name="refresh" size={15} />{$t(busy === 'scan' ? 'Escaneando…' : 'Detectar Java')}</button></div>
          <div class="runtime-grid">{#each requestedMajors as major}{@const runtime = runtimes.find((item) => item.major === major)}<div class="runtime-card" class:installed={runtime?.installed}><span class="java-number"><JavaRuntimeIcon {major}/></span><div><strong>Java {major}</strong><small>{$t(runtime?.installed ? 'Disponible' : 'No instalado')}</small></div>{#if runtime?.installed}<UiIcon name="terminal" size={17} />{:else}<button disabled={!!busy} title={`${$t('Descargar runtime')} Java ${major}`} aria-label={`${$t('Descargar runtime')} Java ${major}`} onclick={() => perform(`java-${major}`, () => oninstalljava(major))}><UiIcon name="download" size={16} /></button>{/if}</div>{/each}</div><div class="java-actions"><button disabled={!!busy || requestedMajors.every((major) => runtimes.some((item) => item.major === major && item.installed))} onclick={installMissing}><UiIcon name="download" size={16} />{$t(busy === 'install' ? 'Preparando…' : 'Instalar los runtimes faltantes')}</button><small>{$t('Las versiones faltantes también se pueden instalar al jugar.')}</small></div>
          <div class="memory-card"><div><h3>{$t('Memoria global')}</h3><span class="memory-amount">{settings.ram_gb}<small>GB</small></span></div><input type="range" min="1" max={memoryLimit} step="1" value={settings.ram_gb} oninput={(event) => onchange({ ram_gb: +event.currentTarget.value })} aria-label={$t('Memoria global')} /><div class="memory-foot"><small>{$t('Memoria del equipo')}: {totalMemoryGb.toFixed(1)} GB · {$t('Memoria recomendada')}: {recommendedRam} GB</small><button disabled={!!busy || settings.ram_gb === recommendedRam} onclick={() => perform('ram', () => onchange({ ram_gb: recommendedRam }))}>{$t('Usar recomendación')}</button></div></div>
        {:else}
          <h3>{$t('Galería de Temas Oficiales')}</h3><div class="wizard-themes">{#each themes.filter((theme) => theme.id !== 'custom') as theme}<button class:chosen={appearance.selectedThemeId === theme.id} style={`--theme-color:${theme.primary};--theme-secondary:${theme.secondary}`} disabled={!!busy} onclick={() => chooseTheme(theme)}>{$t(theme.name)}{#if appearance.selectedThemeId === theme.id}<span>{$t('Activado')}</span>{/if}</button>{/each}</div><div class="connect-card"><div class="account-symbol"><ServiceLogo name="microsoft" size={28}/></div><div><h3>{$t(accountName ? 'Cuenta activa' : 'Conectar cuenta')}</h3>{#if accountName}<p data-no-translate>{accountName}</p>{:else}<p>{$t('Sincroniza tu perfil de Minecraft, tu skin y tus capas oficiales.')}</p>{/if}</div><button disabled={!!busy} onclick={() => perform('microsoft', onconnectmicrosoft)}><ServiceLogo name="microsoft" size={17}/>{$t(busy === 'microsoft' ? 'Conectando…' : 'Conectar Microsoft')}</button></div><small class="account-note">{$t('También puedes continuar con Steve y añadir cuentas desde Ajustes.')}</small>
        {/if}
        {#if message}<div class="wizard-message" class:failed role="status" data-diagnostic>{message}</div>{/if}
      </div>
      <footer><button class="back" disabled={step === 1 || !!busy} onclick={() => { step -= 1; message = ''; }}><UiIcon name="arrow" size={15} />{$t('Volver')}</button><div>{#if step === 3 && !accountName}<span>{$t('Continuar sin cuenta')}</span>{/if}<button class="primary" disabled={!!busy} onclick={advance}>{$t(busy === 'complete' ? 'Guardando…' : step === 3 ? 'Empezar a jugar' : 'Continuar')}<UiIcon name="arrow" size={16} /></button></div></footer>
    </section>
  </div>
</div>

<style>
  .wizard-shell{position:fixed;inset:0;z-index:10000;display:grid;place-items:center;padding:25px;box-sizing:border-box;background:rgba(5,9,22,.95);backdrop-filter:blur(30px);color:#e7ecfa;overflow:auto}.ambient-glow{position:absolute;inset:0;pointer-events:none;background:radial-gradient(ellipse at 12% 20%,rgba(var(--primary-rgb,129,140,248),.15),transparent 54%),radial-gradient(ellipse at 92% 94%,rgba(var(--primary-rgb,129,140,248),.1),transparent 50%)}.wizard-window{position:relative;display:grid;grid-template-columns:290px minmax(0,1fr);width:min(1130px,100%);height:min(810px,calc(100dvh - 50px));min-height:490px;border:1px solid rgba(var(--primary-rgb,129,140,248),.28);border-radius:22px;box-shadow:0 30px 110px #0008;background:#0b1124bf;overflow:hidden}.wizard-rail{padding:35px 30px;display:flex;flex-direction:column;border-right:1px solid #ffffff0b;background:linear-gradient(155deg,rgba(var(--primary-rgb,129,140,248),.13),transparent 60%)}.brand{font-size:1.17rem;font-weight:900;letter-spacing:.02em;color:#c6ccff;display:flex;gap:8px;align-items:center}.rail-copy{margin:55px 0 36px}.eyebrow{font-size:.6rem;letter-spacing:.11em;text-transform:uppercase;color:#a1a9d5;line-height:1.5}h1{font-size:1.9rem;line-height:1.2;letter-spacing:-.06em;margin:15px 0}p{font-size:.79rem;line-height:1.7;color:#929fba;margin:10px 0}.wizard-rail ol{list-style:none;padding:0;display:flex;flex-direction:column;gap:26px;margin:0}.wizard-rail li{display:flex;align-items:center;gap:13px;opacity:.45}.wizard-rail li.current,.wizard-rail li.done{opacity:1}.step-number{width:31px;height:31px;display:grid;place-items:center;border:1px solid #ffffff26;border-radius:9px;flex-shrink:0;font-size:.75rem;color:#a4b1cf}.current .step-number{background:var(--secondary,#4f46e5);border-color:var(--primary,#818cf8);color:#fff;box-shadow:0 0 25px rgba(var(--primary-rgb,129,140,248),.2)}.done .step-number{background:#40a37d20;color:#81c8ac;border-color:#81c8ac45}.wizard-rail li small{display:block;font-size:.58rem;text-transform:uppercase;letter-spacing:.12em;color:#9ba6bf;margin-bottom:5px}.wizard-rail li strong{font-size:.74rem;font-weight:650}.rail-foot{margin-top:auto;padding-top:35px;color:#6f7d9a;font-size:.65rem;line-height:1.65}.wizard-body{display:flex;flex-direction:column;min-width:0;min-height:0;padding:30px 32px 0}.wizard-body header{padding-bottom:22px}.step-count{font-size:.63rem;text-transform:uppercase;letter-spacing:.12em;color:var(--primary,#a5b4fc)}h2{margin:9px 0 19px;font-size:1.65rem;font-weight:720;letter-spacing:-.04em}.step-track{display:flex;gap:6px}.step-track>span{height:3px;flex:1;background:#ffffff09;border-radius:3px}.step-track>span.active{background:var(--primary,#818cf8)}.step-content{min-height:0;flex:1;overflow:auto;padding:0 4px 20px 0;scrollbar-width:thin;scrollbar-color:#46527c transparent}.language-tools{display:flex;gap:10px;margin:0 0 15px;align-items:center}.language-tools input{flex:1}button{display:inline-flex;align-items:center;justify-content:center;gap:8px;border:1px solid #58648e70;border-radius:8px;background:#131c34;color:#d7def4;padding:10px 12px;font:inherit;font-size:.73rem;cursor:pointer;line-height:1.4;transition:background .18s,border-color .18s}button:hover{border-color:var(--primary,#818cf8);background:rgba(var(--primary-rgb,129,140,248),.16)}button:disabled{opacity:.45;cursor:default}button:focus-visible,input:focus-visible{outline:2px solid var(--primary,#818cf8);outline-offset:3px}input[type=search]{background:#090f21;border:1px solid #333f60;color:#dfe6f7;padding:12px;font:inherit;font-size:.75rem;min-width:0;border-radius:8px;width:100%;box-sizing:border-box}.language-tools button{font-size:.65rem;max-width:180px;padding:9px;line-height:1.5}.language-table{border:1px solid #ffffff10;border-radius:10px;overflow:auto;max-height:calc(100dvh - 360px);min-height:150px}table{border-collapse:collapse;width:100%;font-size:.72rem}thead{position:sticky;top:0;z-index:1;background:#141d34}th{font-size:.62rem;color:#8b9ab9;text-align:left;font-weight:650;padding:12px 13px;letter-spacing:.03em}td{border-top:1px solid #ffffff08;padding:10px 13px}td:nth-child(2){color:#8e9cba;font-size:.64rem}td code{display:block;font-size:.59rem;color:#657693;margin-top:4px}.language-button{border:0;background:transparent;padding:2px 0;width:100%;text-align:left;justify-content:space-between;font-size:.73rem}.language-button strong{font-weight:550}.language-button:hover{background:none}.chosen td{background:rgba(var(--primary-rgb,129,140,248),.08)}.chosen .language-button{color:var(--primary,#a5b4fc)}.coverage{display:flex;align-items:center;gap:9px;min-width:86px}.coverage>span{flex:1;display:block;height:4px;background:#ffffff0b;border-radius:4px;overflow:hidden}.coverage i{height:100%;display:block;background:var(--primary,#818cf8);border-radius:inherit}.coverage strong{font-size:.61rem;font-weight:500;min-width:29px;text-align:right}.catalog-note{display:block;color:#7786a4;font-size:.63rem;line-height:1.6;margin:11px 0}.section-top{display:flex;align-items:start;gap:12px;justify-content:space-between}h3{font-size:.9rem;margin:0 0 9px;letter-spacing:-.01em}.section-top p{font-size:.73rem;margin-bottom:15px}.section-top>button{flex-shrink:0;font-size:.64rem;padding:8px}.runtime-grid{display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-top:9px}.runtime-card{display:flex;align-items:center;gap:11px;border:1px solid #ffffff0e;background:#080f2066;border-radius:10px;padding:14px}.java-number{min-width:66px;min-height:40px;display:grid;place-items:center;flex-shrink:0}.runtime-card.installed .java-number{min-width:66px;min-height:40px;display:grid;place-items:center;flex-shrink:0}.runtime-card strong{font-size:.74rem}.runtime-card small{display:block;font-size:.61rem;margin-top:5px;color:#6f809f}.runtime-card.installed small{color:#7dc7a4}.runtime-card>button,.runtime-card>:global(svg){margin-left:auto}.runtime-card>button{padding:7px}.runtime-card>:global(svg){color:#7dc7a4}.java-actions{display:flex;gap:12px;align-items:center;margin:15px 0 24px}.java-actions button{font-size:.65rem;padding:8px 10px}.java-actions small{font-size:.61rem;color:#7d8ba9;line-height:1.6;max-width:190px}.memory-card{border-top:1px solid #ffffff10;padding-top:23px}.memory-card>div:first-child{display:flex;align-items:center;justify-content:space-between}.memory-amount{font-size:1.75rem;font-weight:750;color:var(--primary,#a5b4fc)}.memory-amount small{font-size:.69rem;font-weight:500;margin-left:6px;color:#96a4c0}.memory-card input{width:100%;margin:18px 0 11px;accent-color:var(--primary,#818cf8)}.memory-foot{display:flex;justify-content:space-between;align-items:center;gap:12px}.memory-foot small{font-size:.64rem;color:#8595b3;line-height:1.7}.memory-foot button{font-size:.6rem;padding:7px 9px}.wizard-themes{display:grid;grid-template-columns:repeat(3,1fr);gap:9px;margin:15px 0 25px}.wizard-themes button{position:relative;min-height:51px;font-size:.66rem;color:var(--theme-color);border-color:color-mix(in srgb,var(--theme-color) 22%,transparent);background:#091020;overflow:hidden;display:flex;flex-direction:column;gap:3px;padding:9px}.wizard-themes button:after{content:'';position:absolute;bottom:0;left:15%;right:15%;height:2px;background:linear-gradient(90deg,transparent,var(--theme-color),var(--theme-secondary),transparent);box-shadow:0 -1px 12px var(--theme-color)}.wizard-themes button.chosen{background:color-mix(in srgb,var(--theme-color) 11%,#091020);border-color:var(--theme-color)}.wizard-themes button span{font-size:.51rem;opacity:.7}.connect-card{display:flex;align-items:center;gap:14px;border:1px solid rgba(var(--primary-rgb,129,140,248),.25);background:rgba(var(--primary-rgb,129,140,248),.06);padding:19px;border-radius:12px}.account-symbol{display:grid;place-items:center;height:43px;width:43px;flex-shrink:0;border-radius:11px;background:rgba(var(--primary-rgb,129,140,248),.12);color:var(--primary,#a5b4fc)}.connect-card h3{font-size:.8rem;margin:0}.connect-card p{font-size:.65rem;margin:5px 0 0}.connect-card>button{flex-shrink:0;margin-left:auto;font-size:.65rem;padding:10px}.account-note{font-size:.62rem;color:#7f8fac;display:block;margin-top:11px;line-height:1.6}.wizard-message{border:1px solid #6bd4a848;background:#19362835;padding:12px;color:#a8e5c6;border-radius:8px;margin-top:18px;font-size:.72rem;white-space:pre-wrap;overflow-wrap:anywhere}.wizard-message.failed{border-color:#d4777748;background:#57262630;color:#efa9a9}footer{display:flex;justify-content:space-between;gap:16px;align-items:center;padding:21px 0;border-top:1px solid #ffffff0d;margin-top:auto}footer>div{display:flex;align-items:center;gap:14px}footer>div>span{font-size:.62rem;color:#7d8caa}.back{padding:10px 12px}.back :global(svg){transform:rotate(180deg)}.primary{background:var(--secondary,#4f46e5);border-color:var(--primary,#818cf8);font-size:.76rem;font-weight:650;padding:12px 18px;box-shadow:0 3px 20px rgba(var(--primary-rgb,129,140,248),.15)}.primary:hover{background:color-mix(in srgb,var(--secondary,#4f46e5) 75%,var(--primary,#818cf8))}.empty{text-align:center;padding:25px}
  @media(max-width:950px){.wizard-rail{padding:28px 22px}.wizard-window{grid-template-columns:230px minmax(0,1fr)}.wizard-body{padding:27px 24px 0}.rail-copy{margin-top:40px}h1{font-size:1.65rem}.connect-card{flex-wrap:wrap}.connect-card>button{margin-left:57px}.wizard-themes{grid-template-columns:repeat(3,1fr)}}@media(max-width:720px){.wizard-shell{padding:12px}.wizard-window{height:calc(100dvh - 24px);min-height:450px;grid-template-columns:1fr}.wizard-rail{padding:14px 20px;border-right:0;border-bottom:1px solid #ffffff0c;flex-direction:row;align-items:center;gap:20px}.brand{font-size:.9rem}.rail-copy,.rail-foot{display:none}.wizard-rail ol{flex-direction:row;gap:10px;margin-left:auto}.wizard-rail li{gap:0}.wizard-rail li>div{display:none}.step-number{width:26px;height:26px;font-size:.65rem}.wizard-body{padding:20px 18px 0;min-height:0}.wizard-body header{padding-bottom:15px}h2{font-size:1.3rem;margin-bottom:14px}.wizard-window{grid-template-rows:auto minmax(0,1fr)}.language-table{max-height:calc(100dvh - 310px)}.language-tools{gap:7px}.language-tools button{font-size:.58rem;max-width:130px}.language-tools input{font-size:.68rem}td,th{padding:10px 8px}.language-button{font-size:.66rem}.runtime-grid{gap:8px}.runtime-card{padding:11px;gap:9px}.runtime-card strong{font-size:.69rem}.runtime-card small{font-size:.55rem}.java-number{min-width:66px;min-height:40px;display:grid;place-items:center;flex-shrink:0}.java-actions{flex-wrap:wrap;margin-bottom:20px}.java-actions small{max-width:none}.wizard-themes{gap:7px;margin-bottom:20px}.wizard-themes button{font-size:.58rem;min-height:49px;padding:6px}.connect-card{padding:13px;gap:10px}.connect-card>button{margin-left:0}.connect-card>div:nth-child(2){flex:1;min-width:130px}footer{padding:17px 0}footer>div>span{display:none}.primary{padding:11px 14px;font-size:.69rem}.memory-foot{flex-wrap:wrap}}
</style>
