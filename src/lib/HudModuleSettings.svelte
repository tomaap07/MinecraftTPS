<script lang="ts">
 import {HUD_ANCHORS,LAUNCHER_PLUGINS,isHudWidget,type HudProfile,type HudModuleId} from './launcher_plugins';
 let {profile=$bindable(),selected,onchange=()=>{}}:{profile:HudProfile;selected:HudModuleId;onchange?:()=>void}=$props();
 let module=$derived(profile.modules.find(m=>m.id===selected)!);
 let plugin=$derived(LAUNCHER_PLUGINS.find(p=>p.id===selected)!);
 function range(event:Event,key:'scale'|'opacity'){module[key]=Number((event.currentTarget as HTMLInputElement).value);onchange();}
 function reset(){module.scale=null;module.opacity=null;module.background=null;module.x=null;module.y=null;onchange();}
</script>
<section class="individual" aria-label="Ajustes del módulo seleccionado">
 <small>MÓDULO SELECCIONADO</small><h4>{plugin.name}</h4><p>{plugin.description}</p>
 {#if plugin.delivery==='installer'}<p>Mod externo: se prepara al iniciar una versión compatible. Sus opciones internas se ajustan en Minecraft.</p>
 {:else if !isHudWidget(selected)}
 {#if selected==='optical_zoom'}<p class="shortcut">Mantén <kbd>C</kbd> para hacer zoom y suéltala para recuperar la vista normal. Funciona cuando el adaptador de esa versión está conectado.</p>{:else}<p>Se aplica dentro del juego cuando el conector o mod de esta edición lo permite. No es un widget que se coloque en pantalla.</p>{/if}
 {:else}
 <label>Tamaño de este widget <output>{Math.round((module.scale??profile.scale)*100)}%</output><input aria-label="Tamaño de este widget" type="range" min=".5" max="2.5" step=".05" value={module.scale??profile.scale} oninput={event=>range(event,'scale')}/></label>
 <label>Visibilidad de este widget <output>{Math.round((module.opacity??profile.opacity)*100)}%</output><input aria-label="Visibilidad de este widget" type="range" min=".1" max="1" step=".05" value={module.opacity??profile.opacity} oninput={event=>range(event,'opacity')}/></label>
 <label class="check"><input type="checkbox" checked={module.background??profile.background} onchange={event=>{module.background=event.currentTarget.checked;onchange();}}/>Fondo de este widget</label>
 <label>Ubicación<select bind:value={module.anchor} onchange={()=>{module.x=null;module.y=null;onchange();}}>{#each HUD_ANCHORS as a}<option value={a.id}>{a.label}</option>{/each}</select></label>
 <button onclick={reset}>Restablecer este widget</button><small>Estos controles solo cambian {plugin.name}.</small>
 {/if}
</section>
<style>
 .individual{display:grid;gap:14px}.individual>small{font-size:9px;color:#a98cbd;line-height:1.6}.individual h4{margin:0;font-size:13px;color:#dfbaff}.individual p{font-size:11px;color:#bdafc7;line-height:1.7;margin:0}.individual label{display:grid;grid-template-columns:1fr auto;gap:9px;font-size:11px;color:#c9b7d6}.individual input[type=range],.individual select{grid-column:1/-1;width:100%;box-sizing:border-box;accent-color:#a855f7}.individual .check{display:flex;align-items:center;gap:7px}.individual select,.individual button{padding:9px;background:#19121f;color:#eee4f8;border:1px solid #ffffff25;border-radius:4px;font:11px 'Segoe UI',sans-serif;color-scheme:dark}.individual button{cursor:pointer}.individual button:hover{border-color:#a855f7}
 kbd{padding:3px 6px;border:1px solid #b980dc;border-bottom-width:3px;border-radius:3px;color:#f0d5ff;font:12px Consolas,monospace;background:#251332}
</style>
