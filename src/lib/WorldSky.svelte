<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from './i18n';

  let { compact = false }: { compact?: boolean } = $props();
  let night = $state(false);
  let localTime = $state('--:--');
  let timeZone = $state('…');
  let utcOffset = $state('');
  const skyDescription = $derived(`${night ? $t('Noche') : $t('Día')} · ${localTime} · ${timeZone}${utcOffset ? ` · ${utcOffset}` : ''}`);

  function updateSystemClock(): void {
    const now = new Date();
    const offset = -now.getTimezoneOffset();
    const absoluteOffset = Math.abs(offset);
    utcOffset = `UTC${offset < 0 ? '−' : '+'}${String(Math.floor(absoluteOffset / 60)).padStart(2, '0')}:${String(absoluteOffset % 60).padStart(2, '0')}`;



    try {
      const formatter = new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' });
      localTime = formatter.format(now);
      timeZone = formatter.resolvedOptions().timeZone || utcOffset;
    } catch {
      localTime = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
      timeZone = utcOffset;
    }


    night = now.getHours() < 6 || now.getHours() >= 18;
  }

  onMount(() => {
    const update = () => {
      if (!document.hidden) updateSystemClock();
    };
    updateSystemClock();
    const timer = window.setInterval(update, 15_000);
    document.addEventListener('visibilitychange', update);
    window.addEventListener('focus', update);
    return () => {
      window.clearInterval(timer);
      document.removeEventListener('visibilitychange', update);
      window.removeEventListener('focus', update);
    };
  });
</script>

<div class="world-sky" class:night class:compact title={skyDescription} aria-label={skyDescription}>
  <div class="sky-window" aria-hidden="true">
    <div class="horizon-glow"></div>
    <svg class="stars" viewBox="0 0 160 66" fill="currentColor">
      <path d="M15 13h2v2h-2zM40 27h2v2h-2zM65 8h2v2h-2zM111 12h2v2h-2zM139 24h2v2h-2zM121 37h2v2h-2zM24 40h2v2h-2zM91 21h1v1h-1zM151 8h1v1h-1z" />
    </svg>
    <div class="orbital-track"></div>
    <div class="celestial sun">
      <svg viewBox="0 0 32 32" fill="none"><path stroke="#e6b665" stroke-width="2" d="M16 1v4m0 22v4M1 16h4m22 0h4M5 5l3 3m16 16 3 3M5 27l3-3M24 8l3-3"/><path fill="#ffe2a0" d="m16 7 8 4v10l-8 4-8-4V11z"/><path fill="#fff3ca" d="m16 11 4 2v6l-4 2-4-2v-6z"/></svg>
    </div>
    <div class="celestial moon">
      <svg viewBox="0 0 32 32"><path fill="#d8def6" d="M22 3A13 13 0 1 0 29 22C12 26 7 10 22 3Z"/><path fill="#a855f7" d="m25 3 1 3 3 1-3 1-1 3-1-3-3-1 3-1z"/></svg>
    </div>
    <svg class="mountains" viewBox="0 0 160 20" preserveAspectRatio="none"><path d="M0 20V14H13V9H25V5H35V10H46V14H64V8H75V2H88V9H100V14H114V10H126V4H138V10H149V15H160V20Z"/></svg>
    <div class="horizon-line"></div>
  </div>
  <div class="clock-info" data-no-translate>
    <div class="clock-line"><span class="phase-label"><i aria-hidden="true"></i>{night ? $t('Noche') : $t('Día')}</span><time>{localTime}</time></div>
    <span class="time-zone">{timeZone.replaceAll('_', ' ')}</span>
    <span class="utc-offset">{utcOffset}</span>
  </div>
</div>

<style>
  .world-sky{--sky-warm:#e7bc77;--sky-cool:#b8c9ec;position:relative;display:flex;align-items:center;gap:13px;width:292px;height:72px;flex-shrink:0;color:var(--text-primary,#e8edf4);isolation:isolate}
  .sky-window{position:relative;flex:0 0 125px;height:66px;overflow:hidden;contain:paint}
  .horizon-glow{position:absolute;inset:2px 0 0;background:radial-gradient(ellipse at 50% 96%,#e7bc7719,transparent 71%);transition:background 1.8s}
  .night .horizon-glow{background:radial-gradient(ellipse at 50% 96%,#b8c9ec12,transparent 71%)}
  .orbital-track{position:absolute;left:9px;right:9px;top:8px;height:108px;border:1px dashed currentColor;border-radius:50%;opacity:.12}
  .celestial{position:absolute;left:calc(50% - 18px);top:15px;width:36px;height:36px;transition:transform 1.8s cubic-bezier(.35,.01,.2,1),opacity 1.1s;transform-origin:18px 73px}
  .celestial svg{display:block;width:100%;height:100%}
  .sun{filter:drop-shadow(0 0 13px #e7bc773e);transform:rotate(-10deg);opacity:1}
  .moon{filter:drop-shadow(0 0 11px #b8c9ec29);transform:rotate(-105deg);opacity:0}
  .night .sun{transform:rotate(105deg);opacity:0}
  .night .moon{transform:rotate(-10deg);opacity:1}
  .stars{position:absolute;inset:0;width:100%;height:100%;color:var(--sky-cool);opacity:0;transition:opacity 1.5s}
  .night .stars{opacity:.65}
  .mountains{position:absolute;bottom:3px;left:0;width:100%;height:18px;fill:currentColor;opacity:.12}
  .horizon-line{position:absolute;bottom:2px;left:1px;right:1px;height:1px;background:linear-gradient(90deg,transparent,currentColor,transparent);opacity:.21}
  .phase-label{display:flex;align-items:center;gap:6px;font-size:.64rem;letter-spacing:.06em;white-space:nowrap;font-weight:600;color:var(--text-secondary,#a8b3c7)}
  .phase-label i{width:5px;height:5px;flex-shrink:0;background:var(--sky-warm);box-shadow:0 0 9px #e7bc7740;transition:background 1.8s}
  .night .phase-label i{background:var(--sky-cool);box-shadow:0 0 9px #b8c9ec40}
  .clock-info{min-width:0;flex:1;display:flex;flex-direction:column;gap:5px}
  .clock-line{display:flex;align-items:center;gap:9px;white-space:nowrap}
  .clock-line time{font-size:.7rem;font-weight:700;font-variant-numeric:tabular-nums;letter-spacing:.025em}
  .time-zone{font-size:.62rem;color:var(--text-secondary,#a8b3c7);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
  .utc-offset{font-size:.52rem;color:var(--text-muted,#71829a);font-variant-numeric:tabular-nums;letter-spacing:.04em}
  .compact{width:232px;height:60px;gap:8px}
  .compact .sky-window{flex-basis:84px;height:58px}
  .compact .orbital-track{left:3px;right:3px;top:9px}
  .compact .celestial{width:29px;height:29px;left:calc(50% - 14.5px);top:14px;transform-origin:14.5px 65px}
  .compact .phase-label{font-size:.58rem}
  .compact .clock-info{gap:4px}
  .compact .clock-line{gap:7px}
  .compact .clock-line time{font-size:.64rem}
  .compact .time-zone{font-size:.57rem}
  @media(max-width:760px){.compact{width:192px}.compact .sky-window{flex-basis:56px}.compact .clock-line time{font-size:.6rem}.compact .phase-label{font-size:.53rem}}
  @media(max-width:480px){.compact{width:155px;gap:5px}.compact .sky-window{flex-basis:36px}.compact .clock-line{gap:5px}.compact .phase-label i{display:none}.compact .time-zone{font-size:.52rem}}
  @media(prefers-reduced-motion:reduce){.celestial,.stars,.phase-label i,.horizon-glow{transition:none}.celestial{transform:none!important}}
</style>
