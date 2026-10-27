<script lang="ts">
  import { onDestroy, untrack } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import ServiceLogo from './ServiceLogo.svelte';
  import UiIcon from './UiIcon.svelte';
  import TpCosmeticsPreview from './TpCosmeticsPreview.svelte';
  import {defaultSocialProfile,type SocialProfileData,type SocialLinks} from './launcher_extensions';
  let {playerId,name,skinUrl,fallbackSkinUrl="",capeUrl,model='auto-detect',paused=false}:
    {playerId:string;name:string;skinUrl?:string;fallbackSkinUrl?:string;capeUrl?:string;model?:'default'|'slim'|'auto-detect';paused?:boolean}=$props();
  let profile=$state<SocialProfileData>(defaultSocialProfile()),saved=$state(''),loading=$state(true),saving=$state(false),message=$state(''),failed=$state(false);
  let sequence=0,disposed=false;
  const networks:{id:keyof SocialLinks;name:string;placeholder:string;hosts:string[]}[]=[
    {id:'discord',name:'Discord',placeholder:'usuario o https://discord.com/users/ID',hosts:['discord.com']},
    {id:'youtube',name:'YouTube',placeholder:'https://youtube.com/@tu-canal',hosts:['youtube.com','youtu.be']},
    {id:'twitch',name:'Twitch',placeholder:'https://twitch.tv/tu-canal',hosts:['twitch.tv']},
    {id:'twitter',name:'X / Twitter',placeholder:'https://x.com/tu-usuario',hosts:['x.com','twitter.com']},
    {id:'github',name:'GitHub',placeholder:'https://github.com/tu-usuario',hosts:['github.com']},
  ];
  let dirty=$derived(JSON.stringify(profile)!==saved);
  $effect(()=>{const id=playerId;untrack(()=>void load(id));});
  async function load(id:string){const generation=++sequence;loading=true;saving=false;message='';failed=false;
    try{const next=await invoke<SocialProfileData>('get_social_profile',{playerId:id});if(!disposed&&generation===sequence){profile=next;saved=JSON.stringify(next);}}
    catch(problem){if(!disposed&&generation===sequence){profile=defaultSocialProfile();saved=JSON.stringify(profile);failed=true;message=String(problem);}}
    finally{if(!disposed&&generation===sequence)loading=false;}
  }
  async function save(){if(saving||loading)return;const generation=sequence;saving=true;failed=false;message='';
    try{const next=await invoke<SocialProfileData>('save_social_profile',{playerId,profile:$state.snapshot(profile)});if(!disposed&&generation===sequence){profile=next;saved=JSON.stringify(next);message='Perfil y cosméticos guardados en este equipo.';}}
    catch(problem){if(!disposed&&generation===sequence){failed=true;message=String(problem);}}
    finally{if(!disposed&&generation===sequence)saving=false;}
  }
  async function visit(network:typeof networks[number]){try{const url=new URL(profile.links[network.id]);if(url.protocol!=='https:'||!network.hosts.includes(url.hostname.replace(/^www\./,''))||url.username||url.password)throw new Error('Revisa el enlace antes de abrirlo.');await openUrl(url.toString());}catch{failed=true;message='Usa un enlace HTTPS válido a esa red social. Los nombres de Discord se muestran como texto.';}}
  onDestroy(()=>{disposed=true;sequence++;});
</script>

<section class="social-profile" aria-labelledby="social-title" aria-busy={loading}>
  <header><div><span class="eyebrow">TU IDENTIDAD EN TP</span><h3 id="social-title">Perfil social y cosméticos</h3><p>Enlaces de {name}, guardados localmente. No se conectan cuentas ni se verifica su titularidad.</p></div><button class="save" disabled={loading||saving||!dirty} onclick={save}><UiIcon name="save" size={16}/>{saving?'Guardando…':'Guardar perfil'}</button></header>
  {#if message}<p class="profile-message" class:failed role="status">{message}</p>{/if}
  <div class="profile-columns"><div class="links-panel"><h4>Mis redes</h4><label class="visibility"><input type="checkbox" bind:checked={profile.show_links} disabled={loading||saving}/>Mostrar insignias en mi tarjeta del launcher</label>
    {#each networks as network}<label class="network-field"><span>{#if network.id==='discord'||network.id==='github'}<ServiceLogo name={network.id} size={18}/>{:else}<UiIcon name={network.id==='youtube'?'play':network.id==='twitch'?'message':'link'} size={18}/>{/if}{network.name}</span><input type="text" bind:value={profile.links[network.id]} placeholder={network.placeholder} maxlength="512" disabled={loading||saving} spellcheck="false"/></label>{/each}
    {#if profile.show_links}<div class="social-badges">{#each networks.filter(network=>profile.links[network.id]) as network}{#if profile.links[network.id].startsWith('https://')}<button onclick={()=>void visit(network)}><UiIcon name="external" size={13}/>{network.name}</button>{:else}<span>{network.name}: {profile.links[network.id]}</span>{/if}{/each}</div>{/if}
  </div><div class="cosmetics-panel"><h4>Cosméticos TP</h4><TpCosmeticsPreview {fallbackSkinUrl} {skinUrl} {capeUrl} {model} cosmetics={profile.cosmetics} {paused}/><div class="cosmetic-controls"><label>Capa<select bind:value={profile.cosmetics.cape} disabled={loading||saving}><option value="none">Tu capa oficial / sin capa TP</option><option value="end">Capa TP · Cristal violeta</option></select></label><label class="toggle-line"><input type="checkbox" bind:checked={profile.cosmetics.wings} disabled={loading||saving}/>Alas geométricas TP</label><label class="toggle-line"><input type="checkbox" bind:checked={profile.cosmetics.aura} disabled={loading||saving}/>Aura de partículas moradas</label></div><p class="cosmetics-note">Diseños originales de TP. Esta es una previsualización del launcher; verlos dentro de Minecraft o desde otros clientes requiere un complemento del juego.</p></div></div>
</section>

<style>
  .social-profile{display:grid;gap:18px;padding:23px;border:1px solid #a855f733;background:rgba(var(--bg-base-rgb,11,11,15),.75);color:#e8e1f2;min-width:0}header{display:flex;align-items:center;justify-content:space-between;gap:20px}.eyebrow{font-size:.56rem;letter-spacing:.12em;color:var(--primary,#a855f7)}h3{font-size:1.15rem;margin:8px 0}h4{font-size:.86rem;margin:0 0 16px}p{font-size:.74rem;color:#9e92b2;line-height:1.7;margin:0}.profile-columns{display:grid;grid-template-columns:1fr 1fr;gap:25px;min-width:0}.links-panel,.cosmetics-panel{min-width:0}.network-field{display:grid;gap:8px;margin-top:14px}.network-field>span{display:flex;align-items:center;gap:8px;font-size:.73rem;color:#b5a9c8}input:not([type=checkbox]),select{width:100%;box-sizing:border-box;min-width:0;padding:11px;background:#0b0b0f;color:#e8e1f2;border:1px solid #ffffff20;font:inherit;font-size:.75rem;color-scheme:dark}.visibility,.toggle-line{display:flex;align-items:center;gap:8px;font-size:.7rem;line-height:1.6;color:#b5a9c8}input[type=checkbox]{accent-color:var(--primary,#a855f7)}button{display:inline-flex;align-items:center;justify-content:center;gap:7px;font:inherit;font-size:.71rem;padding:10px 13px;color:#d8c8ea;background:#a855f710;border:1px solid #a855f73b;cursor:pointer}button:disabled{opacity:.4;cursor:default}button:focus-visible,input:focus-visible,select:focus-visible{outline:2px solid var(--primary,#a855f7);outline-offset:3px}.social-badges{display:flex;gap:7px;flex-wrap:wrap;margin-top:18px}.social-badges button,.social-badges span{font-size:.65rem;padding:7px 9px;border:1px solid #a855f736;color:#c9b4df;overflow-wrap:anywhere}.social-badges span{background:#a855f709}.cosmetic-controls{display:grid;gap:13px;margin-top:17px}.cosmetic-controls>label:first-child{font-size:.72rem;display:grid;gap:8px}.cosmetics-note{font-size:.65rem;margin-top:15px}.profile-message{padding:12px 14px;background:#7dbba70a;border:1px solid #7dbba738;color:#b1d6c9}.profile-message.failed{color:#d5bf99;border-color:#cfaa7044;background:#cfaa7008}@media(max-width:1100px){.profile-columns{grid-template-columns:1fr}.social-profile{padding:18px}header{align-items:start;flex-wrap:wrap}.save{width:100%}}
</style>
