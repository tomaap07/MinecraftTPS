<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import configured from './data/community_campaigns.json';
  import UiIcon from './UiIcon.svelte';
  import { isGameRunning } from './game_session';
  type Campaign={id:string;active:boolean;topic:'minecraft';placement:'servers'|'home'|'community';sponsor:string;title:string;description:string;url:string;ends_at?:string};
  let {placement='servers'}:{placement?:Campaign['placement']}=$props();
  let dismissed=$state<string[]>([]),message=$state('');
  function https(value:string){try{return new URL(value).protocol==='https:';}catch{return false;}}
  let campaign=$derived((configured as Campaign[]).find(item=>item.active&&item.topic==='minecraft'&&item.placement===placement
    &&https(item.url)&&!dismissed.includes(item.id)&&(!item.ends_at||Date.parse(item.ends_at)>Date.now())));
  async function visit(url:string){try{await openUrl(url);}catch{message='No se pudo abrir el enlace del patrocinador.';}}
</script>

{#if campaign && !$isGameRunning}
  <aside class="community-spotlight" aria-label="Contenido patrocinado">
    <span class="sponsor-icon"><UiIcon name="info" size={25}/></span>
    <div><small>CONTENIDO PATROCINADO · {campaign.sponsor}</small><strong>{campaign.title}</strong><p>{campaign.description}</p>{#if message}<p role="status">{message}</p>{/if}</div>
    <button class="sponsor-link" onclick={()=>void visit(campaign!.url)}>Conocer más<UiIcon name="external" size={14}/></button>
    <button class="dismiss" title="Ocultar este anuncio" aria-label="Ocultar este anuncio" onclick={()=>{dismissed=[...dismissed,campaign!.id];message='';}}><UiIcon name="close" size={16}/></button>
  </aside>
{/if}

<style>
  .community-spotlight{display:flex;align-items:center;gap:13px;border:1px solid #ffffff16;background:rgba(var(--bg-base-rgb,12,14,24),.7);padding:13px 15px;color:#dce5f4;min-width:0}.sponsor-icon{display:grid;place-items:center;padding:8px;background:#0003}.community-spotlight>div{min-width:0;flex:1;display:grid;gap:5px}small{color:#acb5c6;font-size:.51rem;letter-spacing:.06em}strong{font-size:.8rem}p{font-size:.7rem;color:#99a9c2;line-height:1.6;margin:0}button{font:inherit;color:#c8d7ee;background:none;border:1px solid #ffffff22;padding:8px 10px;display:flex;align-items:center;gap:7px;cursor:pointer;font-size:.68rem}button:hover{border-color:var(--primary,#c084fc)}button:focus-visible{outline:2px solid var(--primary,#c084fc);outline-offset:3px}.dismiss{border:0;padding:5px;align-self:start}.sponsor-link{white-space:nowrap}@media(max-width:800px){.community-spotlight{flex-wrap:wrap}.sponsor-link{margin-left:54px}}
</style>
