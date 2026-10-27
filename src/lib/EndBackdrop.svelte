<script lang="ts">
  let {paused=false}:{paused?:boolean}=$props();
  const particles=Array.from({length:14},(_,index)=>({
    left:((index*37+11)%94)+3,top:((index*23+17)%81)+8,
    delay:-(index*2.7),duration:24+(index%5)*4,drift:((index%3)-1)*32,size:index%4===0?3:2,
  }));
</script>

{#if !paused}
  <div class="end-backdrop" aria-hidden="true"><div class="end-haze"></div>{#each particles as particle}<span style={`left:${particle.left}%;top:${particle.top}%;--delay:${particle.delay}s;--duration:${particle.duration}s;--drift:${particle.drift}px;--size:${particle.size}px`}></span>{/each}</div>
{/if}

<style>
  .end-backdrop{position:absolute;inset:0;z-index:0;overflow:hidden;pointer-events:none;contain:strict}.end-haze{position:absolute;inset:0;background:radial-gradient(ellipse at 85% 25%,rgba(var(--primary-rgb,168,85,247),.065),transparent 55%),radial-gradient(ellipse at 20% 90%,rgba(217,70,239,.035),transparent 55%)}span{position:absolute;width:var(--size);height:var(--size);background:var(--primary,#A855F7);box-shadow:0 0 7px rgba(var(--primary-rgb,168,85,247),.35);opacity:0;animation:teleport-drift var(--duration) var(--delay) ease-in-out infinite}span:nth-of-type(3n){background:var(--secondary,#D946EF)}@keyframes teleport-drift{0%,100%{opacity:0;transform:translate3d(0,65px,0)}25%{opacity:.32}65%{opacity:.2}90%{opacity:0;transform:translate3d(var(--drift),-85px,0)}}@media(prefers-reduced-motion:reduce){span{animation:none;opacity:.12}}
</style>
