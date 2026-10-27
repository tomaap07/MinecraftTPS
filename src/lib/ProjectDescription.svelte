<script lang="ts">
  import { browser } from '$app/environment';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import { openUrl } from '@tauri-apps/plugin-opener';
  let { source }: { source:string } = $props();
  let message = $state('');
  let markup = $derived(browser ? DOMPurify.sanitize(marked.parse(source,{async:false,gfm:true,breaks:true}),{
    ALLOWED_TAGS:['p','br','strong','em','del','h1','h2','h3','h4','h5','h6','ul','ol','li','blockquote','pre','code','a','img','table','thead','tbody','tr','th','td','hr','details','summary'],
    ALLOWED_ATTR:['href','src','alt','title','colspan','rowspan'],
    ALLOWED_URI_REGEXP:/^https?:\/\//i,
    ALLOW_DATA_ATTR:false,
    ALLOW_ARIA_ATTR:false
  }):'');
  function externalLinks(node:HTMLElement) {
    const click=(event:MouseEvent)=>{
      const anchor=(event.target as Element)?.closest('a');
      if(!anchor) return;
      event.preventDefault();
      const url=anchor.getAttribute('href');
      if(url&&/^https?:\/\//i.test(url)) void openUrl(url).catch(error=>message=String(error));
    };
    node.addEventListener('click',click);
    return {destroy(){node.removeEventListener('click',click);}};
  }
</script>
<div class="project-markdown" style="white-space:normal" use:externalLinks>{@html markup}</div>
{#if message}<p role="status">{message}</p>{/if}
<style>
  .project-markdown{font-size:.86rem;line-height:1.8;color:#b9c3db;overflow-wrap:anywhere}.project-markdown :global(h1),.project-markdown :global(h2),.project-markdown :global(h3){color:#ede9ff;line-height:1.3;margin:1.4em 0 .6em;font-weight:700}.project-markdown :global(h1){font-size:1.65rem}.project-markdown :global(h2){font-size:1.3rem}.project-markdown :global(h3){font-size:1.1rem}.project-markdown :global(p){margin:.9em 0}.project-markdown :global(ul),.project-markdown :global(ol){padding-left:1.5rem}.project-markdown :global(li){margin:.4em 0}.project-markdown :global(a){color:var(--primary,#c4b5fd);text-underline-offset:3px}.project-markdown :global(img){max-width:100%;height:auto;border-radius:9px}.project-markdown :global(code){font-size:.85em;background:#05081777;color:#d7c7f5;padding:2px 5px;border-radius:4px}.project-markdown :global(pre){padding:16px;border:1px solid #ffffff14;border-radius:10px;overflow-x:auto;background:#080a15}.project-markdown :global(pre code){background:transparent;padding:0}.project-markdown :global(blockquote){border-left:3px solid #9a7bdb;padding:3px 17px;margin:18px 0;background:#aa80e50c}.project-markdown :global(table){border-collapse:collapse;width:100%;display:block;overflow:auto}.project-markdown :global(th),.project-markdown :global(td){border:1px solid #ffffff20;padding:8px 12px;text-align:left}.project-markdown :global(hr){border:0;border-top:1px solid #ffffff20;margin:22px 0}.project-markdown :global(summary){cursor:pointer;color:#ded3f1}p[role=status]{color:#fda4af}
</style>
