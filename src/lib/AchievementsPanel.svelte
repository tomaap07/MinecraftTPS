<script lang="ts">
  import UiIcon from "./UiIcon.svelte";
  import { onMount } from 'svelte';
  import { EASTER_EGGS, subscribeSecretState, type EasterEgg } from './easter_eggs';
  import { ACHIEVEMENT_CATEGORIES, SECRET_DIFFICULTIES, secretCipher, secretDifficulty, secretGameIcon, secretHint, secretIcon, type SecretDifficulty } from './achievement_catalog';
  import SecretIcon from './SecretIcon.svelte';
  import GameIcon from './GameIcon.svelte';

  type DiscoveryFilter = 'all' | 'discovered' | 'pending';
  const PAGE_SIZE = 36;
  const catalogue = EASTER_EGGS.map(egg => ({ egg, difficulty: secretDifficulty(egg), icon: secretIcon(egg) }));
  let discovered = $state<ReadonlySet<string>>(new Set());
  let discoveryFilter = $state<DiscoveryFilter>('all');
  let difficultyFilter = $state<SecretDifficulty | 'all'>('all');
  let categoryFilter = $state('all');
  let search = $state('');
  let page = $state(1);
  const normalize = (value: string) => value.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLocaleLowerCase('es');
  const label = (difficulty: SecretDifficulty) => SECRET_DIFFICULTIES.find(item => item.id === difficulty)!.label;
  const title = (egg: EasterEgg) => egg.title.charAt(0).toLocaleUpperCase('es') + egg.title.slice(1);
  const isHidden = (egg: EasterEgg, difficulty: SecretDifficulty) => !discovered.has(egg.id) && (difficulty === 'hard' || difficulty === 'legendary');

  let foundCount = $derived(discovered.size);
  let progress = $derived(Math.round(foundCount / EASTER_EGGS.length * 1000) / 10);
  let rarityFound = $derived(catalogue.filter(item => item.difficulty === 'legendary' && discovered.has(item.egg.id)).length);
  let rarityTotal = catalogue.filter(item => item.difficulty === 'legendary').length;
  let filtered = $derived.by(() => {
    const query = normalize(search.trim());
    return catalogue.filter(({ egg, difficulty }) => {
      const found = discovered.has(egg.id);
      if (discoveryFilter === 'discovered' && !found || discoveryFilter === 'pending' && found) return false;
      if (difficultyFilter !== 'all' && difficulty !== difficultyFilter) return false;
      if (categoryFilter !== 'all' && egg.category !== categoryFilter) return false;
      if (!query) return true;

      const visibleText = isHidden(egg, difficulty)
        ? `???? Secreto por descubrir ${label(difficulty)}`
        : `${egg.title} ${egg.text} ${egg.category} ${secretHint(egg)} ${label(difficulty)}`;
      return normalize(visibleText).includes(query);
    });
  });
  let pageCount = $derived(Math.max(1, Math.ceil(filtered.length / PAGE_SIZE)));
  let currentPage = $derived(Math.min(page, pageCount));
  let pageEntries = $derived(filtered.slice((currentPage - 1) * PAGE_SIZE, currentPage * PAGE_SIZE));
  let pageButtons = $derived.by(() => {
    const buttons: number[] = [];
    for (let number = 1; number <= pageCount; number++) {
      if (number === 1 || number === pageCount || Math.abs(number - currentPage) <= 2) buttons.push(number);
    }
    return buttons;
  });

  function resetFilters() { search = ''; discoveryFilter = 'all'; difficultyFilter = 'all'; categoryFilter = 'all'; page = 1; }
  function changeDiscovery(filter: DiscoveryFilter) { discoveryFilter = filter; page = 1; }

  onMount(() => subscribeSecretState(state => { discovered = new Set(state.discovered); }));
</script>

<section class="achievements-panel" aria-labelledby="achievements-heading">
  <header class="panel-heading">
    <div><span class="eyebrow">TU CUADERNO DE DESCUBRIMIENTOS</span><h2 id="achievements-heading">Logros<span>.</span></h2><p>Un recuerdo, una visita inesperada, una historia más en tu colección.</p></div>
    <div class="collection-emblem"><GameIcon name="diamond" fallback="trophy" size={38}/><span>COLECCIÓN<br/><strong>TP LAUNCHER</strong></span></div>
  </header>

  <div class="collection-overview">
    <div class="progress-overview"><div><span>Tu colección</span><strong>{progress.toLocaleString('es')}<small>%</small></strong></div><progress max={EASTER_EGGS.length} value={foundCount} aria-label="Progreso de la colección"></progress><p>{foundCount.toLocaleString('es')} de {EASTER_EGGS.length.toLocaleString('es')} secretos descubiertos</p></div>
    <dl class="collection-stats"><div><dt>Descubiertos</dt><dd>{foundCount.toLocaleString('es')}</dd></div><div><dt>Por descubrir</dt><dd>{(EASTER_EGGS.length - foundCount).toLocaleString('es')}</dd></div><div class="legendary-stat"><dt><SecretIcon name="spark" size={14}/>Legendarios</dt><dd>{rarityFound}<small> / {rarityTotal}</small></dd></div></dl>
  </div>

  <div class="discovery-note"><SecretIcon name="lock" size={18}/><p>Los secretos <strong>difíciles y legendarios</strong> ocultan su nombre y su diseño hasta que los descubres. Los logros se registran automáticamente en este dispositivo.</p></div>

  <div class="collection-controls">
    <div class="filter-tabs" role="group" aria-label="Estado de los logros">
      <button class:active={discoveryFilter === 'all'} aria-pressed={discoveryFilter === 'all'} onclick={() => changeDiscovery('all')}><UiIcon name="grid" size={15}/>Todos <span>{EASTER_EGGS.length.toLocaleString('es')}</span></button>
      <button class:active={discoveryFilter === 'discovered'} aria-pressed={discoveryFilter === 'discovered'} onclick={() => changeDiscovery('discovered')}><UiIcon name="check" size={15}/>Descubiertos <span>{foundCount.toLocaleString('es')}</span></button>
      <button class:active={discoveryFilter === 'pending'} aria-pressed={discoveryFilter === 'pending'} onclick={() => changeDiscovery('pending')}><UiIcon name="search" size={15}/>Por descubrir <span>{(EASTER_EGGS.length - foundCount).toLocaleString('es')}</span></button>
    </div>
    <div class="filter-fields">
      <label class="search-field"><span>Buscar en la colección</span><div><SecretIcon name="compass" size={17}/><input type="search" bind:value={search} oninput={() => { page = 1; }} placeholder="Nombres, recuerdos y pistas visibles…"/></div></label>
      <label><span>Dificultad</span><select bind:value={difficultyFilter} onchange={() => { page = 1; }}><option value="all">Todas las dificultades</option>{#each SECRET_DIFFICULTIES as difficulty}<option value={difficulty.id}>{difficulty.label}</option>{/each}</select></label>
      <label><span>Categoría</span><select bind:value={categoryFilter} onchange={() => { page = 1; }}><option value="all">Todas las categorías</option>{#each ACHIEVEMENT_CATEGORIES as category}<option value={category}>{category}</option>{/each}</select></label>
    </div>
    <div class="results-summary" aria-live="polite"><span>{filtered.length.toLocaleString('es')} {filtered.length === 1 ? 'logro' : 'logros'}{#if filtered.length} · {(currentPage - 1) * PAGE_SIZE + 1}–{Math.min(currentPage * PAGE_SIZE, filtered.length)}{/if}</span><span>Los nombres ocultos no aparecen en la búsqueda.</span></div>
  </div>

  {#if pageEntries.length}
    <div class="achievement-grid">
      {#each pageEntries as { egg, difficulty, icon } (egg.id)}
        {@const found = discovered.has(egg.id)}
        {@const hidden = isHidden(egg, difficulty)}
        <article class="achievement-card" class:discovered={found} class:concealed={hidden} class:legendary={difficulty === 'legendary'}>
          <div class="card-top"><div class="secret-illustration" class:obscured={hidden}>{#if hidden}<SecretIcon name="lock" size={32}/>{:else}<GameIcon name={secretGameIcon(egg)} fallback={icon} size={34}/>{/if}</div><div class="card-badges"><span class="difficulty" class:easy={difficulty === 'easy'} class:normal={difficulty === 'normal'} class:hard={difficulty === 'hard'} class:rare={difficulty === 'legendary'}>{label(difficulty)}</span><span class="status" class:found>{#if found}<svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="m3 8 3 3 7-7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>Descubierto{:else}Por descubrir{/if}</span></div></div>
          <div class="card-content">
            <span class="card-category">{hidden ? 'SECRETO SIN IDENTIFICAR' : egg.category}</span>
            <h3>{hidden ? '????' : title(egg)}</h3>
            {#if hidden}<p class="cipher" aria-hidden="true">{secretCipher(egg.id)}</p><p class="concealed-description">Una historia que todavía no se ha revelado.</p>
            {:else}<p class="secret-description">{egg.text}</p>{/if}
          </div>
          {#if !hidden}<details class="hint"><summary><SecretIcon name="compass" size={14}/>Pista para descubrirlo<span>+</span></summary><p>{secretHint(egg)}</p></details>{:else}<div class="locked-hint"><SecretIcon name="lock" size={13}/><span>La pista se revela con el descubrimiento</span></div>{/if}
        </article>
      {/each}
    </div>
    {#if pageCount > 1}
      <nav class="pagination" aria-label="Páginas de logros">
        <button class="page-direction" disabled={currentPage === 1} onclick={() => { page = currentPage - 1; }} aria-label="Página anterior"><UiIcon name="left" size={15}/><span>Anterior</span></button>
        <div class="page-numbers">{#each pageButtons as number, index}{#if index > 0 && number - pageButtons[index - 1] > 1}<span class="ellipsis" aria-hidden="true">…</span>{/if}<button class:active={currentPage === number} aria-current={currentPage === number ? 'page' : undefined} aria-label={`Página ${number}`} onclick={() => { page = number; }}>{number}</button>{/each}</div>
        <button class="page-direction" disabled={currentPage === pageCount} onclick={() => { page = currentPage + 1; }} aria-label="Página siguiente"><span>Siguiente</span><UiIcon name="arrow" size={15}/></button>
      </nav>
    {/if}
  {:else}
    <div class="empty-state"><SecretIcon name={discoveryFilter === 'discovered' && !foundCount ? 'trophy' : 'compass'} size={44}/><h3>{discoveryFilter === 'discovered' && !foundCount ? 'Tu primera historia está por llegar' : 'No hay logros con estos filtros'}</h3><p>{discoveryFilter === 'discovered' && !foundCount ? 'Explora el launcher y observa al personaje de bienvenida. Cada descubrimiento aparecerá aquí.' : 'Prueba otra dificultad, categoría o un texto visible en las tarjetas.'}</p><button onclick={resetFilters}><UiIcon name="grid" size={16}/>Ver toda la colección</button></div>
  {/if}

  <footer class="collection-footer"><SecretIcon name="book" size={16}/><p>Esta colección reúne los {EASTER_EGGS.length.toLocaleString('es')} secretos del launcher. Un mismo tema puede tener varios recuerdos distintos; cada uno cuenta por separado.</p></footer>
</section>

<style>
  .achievements-panel{--panel-border:#ffffff16;--panel-text:#ecf0f7;--panel-muted:#9caac0;min-width:0;color:var(--panel-text);padding-bottom:14px}.panel-heading{display:flex;align-items:center;justify-content:space-between;gap:25px;margin-bottom:25px}.eyebrow{display:block;font-size:.61rem;font-weight:700;letter-spacing:.16em;color:var(--primary);margin-bottom:10px}h2{margin:0 0 10px;font-size:2.1rem;line-height:1.15;letter-spacing:-.045em}h2>span{color:var(--primary)}.panel-heading p{font-size:.82rem;line-height:1.7;color:var(--panel-muted);margin:0}.collection-emblem{display:flex;align-items:center;gap:13px;flex-shrink:0;color:var(--primary);padding:16px 20px;border:1px solid rgba(var(--primary-rgb),.22);border-radius:13px;background:rgba(var(--primary-rgb),.045)}.collection-emblem>span{font-size:.53rem;line-height:1.8;letter-spacing:.13em;color:var(--panel-muted)}.collection-emblem strong{color:var(--panel-text);font-size:.63rem;letter-spacing:.05em}.collection-overview{display:grid;grid-template-columns:minmax(220px,1fr) minmax(380px,1.7fr);gap:36px;padding:27px;border:1px solid var(--panel-border);border-radius:16px;background:linear-gradient(110deg,rgba(var(--primary-rgb),.095),#0c142063);margin-bottom:15px}.progress-overview>div{display:flex;align-items:center;justify-content:space-between;gap:15px}.progress-overview>div>span{font-size:.76rem;color:#c7d1e1}.progress-overview strong{font-size:1.7rem;font-weight:650;letter-spacing:-.04em;color:var(--primary)}.progress-overview strong small{font-size:.85rem;margin-left:3px}.progress-overview progress{appearance:none;display:block;width:100%;height:7px;border:0;border-radius:5px;overflow:hidden;margin:13px 0 11px;background:#ffffff0b}.progress-overview progress::-webkit-progress-bar{background:#ffffff0b;border-radius:5px}.progress-overview progress::-webkit-progress-value{background:var(--primary);border-radius:5px}.progress-overview progress::-moz-progress-bar{background:var(--primary);border-radius:5px}.progress-overview p{font-size:.67rem;color:var(--panel-muted);margin:0}.collection-stats{margin:0;display:grid;grid-template-columns:repeat(3,1fr);align-items:center}.collection-stats>div{padding:6px 20px;border-left:1px solid var(--panel-border)}dt{display:flex;align-items:center;gap:6px;font-size:.67rem;color:var(--panel-muted);margin-bottom:9px}dd{font-size:1.75rem;letter-spacing:-.04em;margin:0;font-weight:650}dd small{font-size:.95rem;font-weight:450;color:var(--panel-muted)}.legendary-stat dd,.legendary-stat :global(svg){color:#d1b579}.discovery-note{display:flex;align-items:flex-start;gap:10px;padding:13px 16px;color:#96a7be;margin-bottom:24px;background:#0a13234d;border-radius:9px;border:1px solid #ffffff08}.discovery-note> :global(svg){flex-shrink:0;margin-top:1px;color:var(--primary)}.discovery-note p{margin:0;font-size:.69rem;line-height:1.7}.discovery-note strong{color:#c9d3e3;font-weight:550}button,input,select{font:inherit}button{cursor:pointer;display:inline-flex;align-items:center;justify-content:center;gap:7px;border:1px solid #ffffff19;border-radius:8px;background:#111b2bd9;color:#c6d0e3;padding:10px 14px;font-size:.72rem}button:hover:not(:disabled){border-color:rgba(var(--primary-rgb),.7);color:var(--panel-text);background:rgba(var(--primary-rgb),.1)}button:disabled{opacity:.35;cursor:default}button:focus-visible,input:focus-visible,select:focus-visible,summary:focus-visible{outline:2px solid var(--primary);outline-offset:3px}.collection-controls{margin-bottom:17px}.filter-tabs{display:flex;flex-wrap:wrap;gap:6px;padding-bottom:18px}.filter-tabs button{background:#101929b0;border-color:#ffffff10;padding:10px 13px}.filter-tabs button.active{border-color:rgba(var(--primary-rgb),.45);color:var(--primary);background:rgba(var(--primary-rgb),.1)}.filter-tabs button>span{font-size:.6rem;border-radius:4px;background:#ffffff0a;color:inherit;padding:2px 6px}.filter-fields{display:grid;grid-template-columns:minmax(220px,1.5fr) minmax(160px,.8fr) minmax(190px,1fr);gap:13px}.filter-fields label{display:flex;flex-direction:column;gap:8px;min-width:0}.filter-fields label>span{font-size:.65rem;color:var(--panel-muted)}.filter-fields input,.filter-fields select{box-sizing:border-box;width:100%;height:41px;background:#0b1322;color:#dfe6f3;border:1px solid #ffffff1b;border-radius:8px;padding:0 11px;font-size:.73rem;min-width:0;color-scheme:dark}.filter-fields input::placeholder{color:#728299}.filter-fields select{cursor:pointer}.filter-fields option{background:#101929;color:#dfe6f3}.search-field>div{position:relative}.search-field>div> :global(svg){position:absolute;left:12px;top:12px;color:#71839b;pointer-events:none}.search-field input{padding-left:38px}.results-summary{display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:10px;margin-top:17px;font-size:.64rem;color:#95a5bb}.results-summary>span:last-child{color:#71839c}.achievement-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:14px}.achievement-card{display:flex;flex-direction:column;min-width:0;overflow:hidden;padding:19px;border:1px solid var(--panel-border);border-radius:13px;background:linear-gradient(150deg,#142033b8,#0c1421e8);position:relative}.achievement-card.discovered{border-color:rgba(var(--primary-rgb),.27);background:linear-gradient(150deg,rgba(var(--primary-rgb),.075),#0c1421ed)}.achievement-card.legendary.discovered{border-color:#d1b57948;background:linear-gradient(150deg,#d1b57912,#0c1421ed)}.card-top{display:flex;justify-content:space-between;align-items:center;gap:10px;margin-bottom:18px}.secret-illustration{display:grid;place-items:center;flex-shrink:0;width:56px;height:56px;border-radius:12px;background:rgba(var(--primary-rgb),.07);color:var(--primary);border:1px solid rgba(var(--primary-rgb),.15)}.discovered .secret-illustration{background:rgba(var(--primary-rgb),.12)}.legendary.discovered .secret-illustration{background:#d1b57910;border-color:#d1b57930;color:#d1b579}.secret-illustration.obscured{background:#030609;border:1px solid #28303d;color:#080b11;box-shadow:inset 0 0 18px #0009}.secret-illustration.obscured> :global(svg){stroke:#46505f;fill:#020407}.card-badges{display:flex;flex-direction:column;gap:9px;align-items:flex-end}.difficulty{font-size:.54rem;line-height:1.2;border-radius:4px;padding:4px 6px;border:1px solid #ffffff16;letter-spacing:.035em}.difficulty.easy{color:#9bc4ac;background:#9bc4ac09;border-color:#9bc4ac24}.difficulty.normal{color:#9cb7db;background:#9cb7db09;border-color:#9cb7db24}.difficulty.hard{color:#c6a4df;background:#c6a4df09;border-color:#c6a4df24}.difficulty.rare{color:#d1b579;background:#d1b57909;border-color:#d1b57924}.status{display:flex;align-items:center;gap:4px;font-size:.55rem;color:#728299}.status.found{color:var(--primary)}.card-content{flex:1;min-width:0}.card-category{display:block;font-size:.49rem;line-height:1.5;letter-spacing:.085em;text-transform:uppercase;color:#8495ae;margin-bottom:9px}h3{font-size:.91rem;line-height:1.45;letter-spacing:-.015em;margin:0 0 10px;font-weight:650;overflow-wrap:anywhere}.secret-description{font-size:.7rem;line-height:1.8;color:#a3b2c8;margin:0 0 19px;overflow-wrap:anywhere}.concealed{background:linear-gradient(150deg,#101722dd,#080e18f0)}.concealed h3{color:#738096;letter-spacing:.17em;font-size:1.25rem}.concealed .card-category{color:#5c6a7f}.cipher{font-family:monospace;color:#4c596b;font-size:.95rem;letter-spacing:.21em;margin:4px 0 13px;user-select:none}.concealed-description{font-size:.69rem;color:#62728a;line-height:1.7;margin:0 0 20px}.hint{border-top:1px solid #ffffff0d;padding-top:12px;color:#92a7c5;margin-top:auto}.hint summary{list-style:none;display:flex;align-items:center;gap:7px;font-size:.62rem;cursor:pointer;min-height:22px;border-radius:3px}.hint summary::-webkit-details-marker{display:none}.hint summary>span{margin-left:auto;font-size:.95rem;color:#8396b1}.hint[open] summary>span{transform:rotate(45deg)}.hint p{font-size:.66rem;line-height:1.8;margin:10px 0 1px;color:#92a7c5}.locked-hint{display:flex;align-items:center;gap:7px;font-size:.57rem;line-height:1.7;color:#56657a;padding-top:14px;border-top:1px solid #ffffff08}.locked-hint :global(svg){flex-shrink:0}.pagination{display:flex;align-items:center;justify-content:space-between;gap:15px;margin-top:24px}.page-numbers{display:flex;align-items:center;justify-content:center;flex-wrap:wrap;gap:5px}.page-numbers>button{width:34px;min-height:34px;padding:5px}.page-numbers>button.active{color:var(--primary);border-color:rgba(var(--primary-rgb),.5);background:rgba(var(--primary-rgb),.1)}.ellipsis{font-size:.7rem;color:#697b94;padding:0 3px}.page-direction{flex-shrink:0}.empty-state{display:flex;align-items:center;flex-direction:column;gap:15px;text-align:center;padding:65px 20px;border:1px dashed #ffffff1d;border-radius:14px;background:#10192866}.empty-state> :global(svg){color:var(--primary)}.empty-state h3{margin:0;font-size:1rem}.empty-state p{font-size:.76rem;line-height:1.8;color:var(--panel-muted);max-width:480px;margin:0 0 7px}.collection-footer{display:flex;align-items:flex-start;gap:9px;margin-top:25px;padding-top:18px;border-top:1px solid #ffffff0e;color:#687c96}.collection-footer> :global(svg){flex-shrink:0;margin-top:2px}.collection-footer p{font-size:.64rem;line-height:1.7;margin:0}@media(min-width:1600px){.achievement-grid{grid-template-columns:repeat(4,minmax(0,1fr))}}@media(max-width:1120px){.achievement-grid{grid-template-columns:repeat(2,minmax(0,1fr))}.collection-overview{grid-template-columns:1fr;gap:22px}.collection-stats>div:first-child{border-left:0;padding-left:0}.filter-fields{grid-template-columns:1fr 1fr}.search-field{grid-column:1/-1}}@media(max-width:620px){.panel-heading{align-items:flex-start}.collection-emblem{display:none}h2{font-size:1.8rem}.collection-overview{padding:20px;gap:20px}.collection-stats>div{padding:5px 12px}dt{font-size:.57rem}dd{font-size:1.4rem}.collection-stats .legendary-stat dt{gap:4px}.achievement-grid{grid-template-columns:1fr}.filter-fields{grid-template-columns:1fr}.search-field{grid-column:auto}.pagination{gap:8px}.page-direction{padding:10px}.page-direction>span{display:none}.page-numbers{gap:3px}.page-numbers>button{width:30px;min-height:32px;padding:3px}.filter-tabs{gap:5px}.filter-tabs button{font-size:.65rem;padding:9px 10px}.filter-tabs button>span{font-size:.55rem}.results-summary>span:last-child{display:none}.panel-heading p{font-size:.75rem}.discovery-note p{font-size:.64rem}}@media(prefers-reduced-motion:reduce){.hint[open] summary>span{transform:none}}
</style>
