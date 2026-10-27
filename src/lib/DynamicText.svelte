<script lang="ts">
  import {invoke} from '@tauri-apps/api/core';
  import {locale,t} from './i18n';
  import {dynamicTranslationEnabled} from './dynamic_translation';
  import {onMount,untrack} from 'svelte';
  let{text,automatic=true}:{text:string;automatic?:boolean}=$props();
  let value=$state(''),translated=$state(false),busy=$state(false),status=$state(''),visible=$state(false);
  let element:HTMLSpanElement;let request=0;
  onMount(()=>{const observer=new IntersectionObserver(entries=>{if(entries.some(entry=>entry.isIntersecting))visible=true;},{rootMargin:'100px'});observer.observe(element);return()=>{observer.disconnect();request++;};});
  $effect(()=>{text;$locale;request++;value=text;translated=false;status='';busy=false;});
  $effect(()=>{const source=text,language=$locale;if(source&&visible&&automatic&&$dynamicTranslationEnabled&&!language.startsWith('en'))untrack(()=>void translate());});
  async function translate(){
    if(busy)return;const id=++request,source=text,language=$locale;busy=true;status='';
    try{const result=await invoke<{text:string;translated:boolean;status:string}>('translate_public_text',{text:source,targetLanguage:language});if(id===request){value=result.text;translated=result.translated;status=result.status;}}
    catch{if(id===request)status='unavailable';}finally{if(id===request)busy=false;}
  }
</script>
<span bind:this={element} class="dynamic-text" data-no-translate><span>{value||text}</span>{#if !$locale.startsWith('en')}<span class="translation-actions"><button disabled={busy} onclick={event=>{event.preventDefault();event.stopPropagation();if(translated){value=text;translated=false;status='';}else void translate();}}>{$t(busy?'Traduciendo…':translated?'Mostrar original':'Traducir')}</button>{#if translated}<small>MyMemory</small>{:else if status==='limit'}<small>{$t('Límite gratuito alcanzado; se muestra el original.')}</small>{:else if status==='unavailable'}<small>{$t('Traducción no disponible; se muestra el original.')}</small>{/if}</span>{/if}</span>
<style>.dynamic-text{display:block}.translation-actions{display:flex;align-items:center;gap:8px;margin-top:7px;flex-wrap:wrap}.translation-actions button{font:inherit;font-size:.67rem;padding:0;border:0;background:none;color:#b8a3d3;cursor:pointer;text-decoration:underline;text-underline-offset:3px}.translation-actions button:disabled{opacity:.5;cursor:wait}.translation-actions small{font-size:.58rem;color:#8f98aa;line-height:1.4}</style>
