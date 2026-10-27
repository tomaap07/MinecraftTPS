<script lang="ts">
 import {observeSize} from "./observe_size";
 import {invoke} from '@tauri-apps/api/core';
 import {isHudWidget,type HudProfile} from './launcher_plugins';
 import type {OverlaySnapshot} from './launcher_extensions';
 import HudEquipmentIcon from './HudEquipmentIcon.svelte';
 import {durabilityPercent,equipmentKind,equipmentRows,PREVIEW_EQUIPMENT} from './hud_equipment';
 import {metricValue,METRIC_MODULES} from './hud_metrics';
 import {hudMetricState} from './hud_data';
 import {visibleWidgetPosition,type HudSize} from './hud_widget_bounds';
 let {profile,frame=null,preview=false,editing=false,onchange=()=>{}}:{profile:HudProfile;frame?:OverlaySnapshot|null;preview?:boolean;editing?:boolean;onchange?:()=>void}=$props();
 let internal=$derived(frame?.telemetry?.has_world?frame.telemetry:null);
 let icons=$state<Record<string,string>>({});
 let brokenIcons=$state<Record<string,boolean>>({});
 let iconKey='';
 $effect(()=>{const pid=frame?.pid,ids=[...new Set(internal?.armor.map(i=>i.item_id).filter((v):v is string=>!!v&&v.startsWith('minecraft:'))??[])];const key=pid+':'+ids.join(',');if(!pid||!ids.length||preview||key===iconKey)return;iconKey=key;icons={};brokenIcons={};void invoke<Record<string,string>>('hud_item_icons',{pid,items:ids}).then(value=>{if(iconKey===key)icons=value;}).catch(()=>{});});
 let equipment=$derived(equipmentRows(preview?PREVIEW_EQUIPMENT:internal?.armor??[]));

 const directions:Record<string,number>={north:0,east:90,south:180,west:270};
 let day=$derived(internal?.world_time!=null?Math.max(1,Math.floor(internal.world_time/24000)+1+(profile.day_offset||0)):null);
 let clock=$derived(new Date(frame?.updated_at||Date.now()).toLocaleTimeString([],{hour:'2-digit',minute:'2-digit'}));
 function paintMap(node:HTMLCanvasElement,encoded:string){let previous='';function draw(value:string){if(value===previous||value.length!==2304)return;try{const rgb=atob(value);if(rgb.length!==1728)return;const context=node.getContext('2d');if(!context)return;node.width=24;node.height=24;const pixels=context.createImageData(24,24);for(let p=0;p<576;p++){pixels.data[p*4]=rgb.charCodeAt(p*3);pixels.data[p*4+1]=rgb.charCodeAt(p*3+1);pixels.data[p*4+2]=rgb.charCodeAt(p*3+2);pixels.data[p*4+3]=255;}context.putImageData(pixels,0,0);previous=value;}catch{}}draw(encoded);return{update:draw};}

 type Module=HudProfile['modules'][number];
 let scene:HTMLDivElement;let sizes=$state<Record<string,HudSize>>({});let viewport=$state<HudSize>({width:0,height:0});let drag=$state<{id:string;pointer:number;startX:number;startY:number;left:number;top:number;width:number;height:number}|null>(null);
 let visible=$derived(profile.modules.filter(m=>m.enabled&&isHudWidget(m.id)&&(preview||hudMetricState(m.id,frame).available)&&!(m.id==='days'&&profile.days_with_fps&&(preview||hudMetricState('fps',frame).available)&&profile.modules.some(p=>p.id==='fps'&&p.enabled))));
 function measure(node:HTMLElement,id:string){return{destroy:observeSize(node,(width,height)=>{sizes[id]={width,height};},'offset')};}
 function measureViewport(node:HTMLElement){return{destroy:observeSize(node,(width,height)=>{viewport={width,height};})};}
 function position(module:Module){
  const scale=module.scale??profile.scale;
  if(module.x!=null&&module.y!=null){const visible=visibleWidgetPosition({x:module.x,y:module.y},viewport,sizes[module.id]??{width:0,height:0},scale);return `left:${visible.x*100}%;top:${visible.y*100}%;transform:scale(${scale});transform-origin:top left`;}
  const previous=visible.slice(0,visible.indexOf(module)).filter(m=>m.anchor===module.anchor&&m.x==null);const offset=14+previous.reduce((sum,m)=>sum+(sizes[m.id]?.height??32)*(m.scale??profile.scale)+7,0);
  const vertical=module.anchor.startsWith('bottom')?`bottom:${offset}px;transform-origin:bottom `:`top:${offset}px;transform-origin:top `;
  return vertical+(module.anchor.endsWith('right')?`right;right:14px;transform:scale(${scale})`:module.anchor.endsWith('center')?`center;left:50%;transform:translateX(-50%) scale(${scale})`:`left;left:14px;transform:scale(${scale})`);
 }
 function begin(event:PointerEvent,module:Module){if(!editing||event.button!==0)return;event.preventDefault();event.stopPropagation();const rect=event.currentTarget instanceof HTMLElement?event.currentTarget.getBoundingClientRect():null;if(!rect)return;const area=scene.getBoundingClientRect();drag={id:module.id,pointer:event.pointerId,startX:event.clientX,startY:event.clientY,left:rect.left-area.left,top:rect.top-area.top,width:rect.width,height:rect.height};module.x=drag.left/area.width;module.y=drag.top/area.height;scene.setPointerCapture(event.pointerId);}
 function moving(event:PointerEvent){if(!drag||event.pointerId!==drag.pointer)return;const area=scene.getBoundingClientRect(),module=profile.modules.find(m=>m.id===drag!.id)!;const clamp=(v:number,max:number)=>Math.max(0,Math.min(max,v));const snap=(v:number,targets:number[])=>targets.find(target=>Math.abs(target-v)<.008)??v;module.x=clamp(snap((drag.left+event.clientX-drag.startX)/area.width,[.02,.5-drag.width/area.width/2,1-drag.width/area.width-.02]),Math.max(0,1-drag.width/area.width));module.y=clamp(snap((drag.top+event.clientY-drag.startY)/area.height,[.02,.5-drag.height/area.height/2,1-drag.height/area.height-.02]),Math.max(0,1-drag.height/area.height));}
 function finish(){if(drag){drag=null;onchange();}}
 function keyboard(event:KeyboardEvent,module:Module){if(!editing||!['ArrowLeft','ArrowRight','ArrowUp','ArrowDown'].includes(event.key))return;event.preventDefault();const rect=(event.currentTarget as HTMLElement).getBoundingClientRect(),area=scene.getBoundingClientRect();if(!area.width||!area.height)return;module.x=Math.max(0,Math.min(Math.max(0,1-rect.width/area.width),(rect.left-area.left+(event.key==='ArrowLeft'?-5:event.key==='ArrowRight'?5:0))/area.width));module.y=Math.max(0,Math.min(Math.max(0,1-rect.height/area.height),(rect.top-area.top+(event.key==='ArrowUp'?-5:event.key==='ArrowDown'?5:0))/area.height));onchange();}

</script>
<div bind:this={scene} use:measureViewport class="hud-layer" class:editing class:chroma={profile.chroma} style:--hud-accent={profile.accent} style:--hud-text={profile.text_color} style:--hud-bg={profile.background_color} role="group" onpointermove={moving} onpointerup={finish} onpointercancel={finish} style={'--scale:'+profile.scale+';--opacity:'+profile.opacity} class:uniform={profile.modules.some(m=>m.id==='custom_font'&&m.enabled)}  aria-label="Widgets del HUD">
 {#each visible as module (module.id)}{@const state=hudMetricState(module.id,frame,preview)}<div class="widget-position" style={position(module)} style:--opacity={module.opacity??profile.opacity} class:no-background={!(module.background??profile.background)}>
 <div class="widget" use:measure={module.id} role="button" tabindex={editing?0:-1} aria-disabled={!editing} aria-label={`Mover ${module.id}`} onpointerdown={(event)=>begin(event,module)} onkeydown={(event)=>keyboard(event,module)} class:unavailable={!state.available} class:keys={module.id==='keystrokes'} title={module.id==='days'||module.id==='fps'?state.detail+' '+(internal?.world_time_source||''):state.detail}>
 {#if !state.available}<span>{({fps:'FPS',coordinates:'XYZ',direction:'DIRECCIÓN',armor:'ARMADURA',ping:'PING',minimap:'MINIMAPA'} as Record<string,string>)[module.id]}</span><b>{state.short}</b>
 {:else if module.id==='fps'}<b>{preview?'144':Math.round(frame?.game_fps??0)}</b><span>FPS</span>{#if profile.days_with_fps&&profile.modules.some(m=>m.id==='days'&&m.enabled)&&(preview||day!=null)}<i class="divider"></i><b>Día {preview?1:day}</b>{/if}
 {:else if module.id==='days'}<span>MUNDO</span><b>Día {preview?1:day}</b>
 {:else if module.id==='biome'}<span>BIOMA</span><b>{preview?'Plains':internal?.biome?.replace('minecraft:','').replaceAll('_',' ')}</b>
 {:else if module.id.startsWith('m_')}<span>{METRIC_MODULES.find(p=>p.id===module.id)?.name}</span><b>{metricValue(module.id,frame,preview)}</b>
 {:else if module.id==='keystrokes'}
 <div class="movement"><span class:pressed={!preview&&frame?.keys.w}>W</span><div><span class:pressed={!preview&&frame?.keys.a}>A</span><span class:pressed={!preview&&frame?.keys.s}>S</span><span class:pressed={!preview&&frame?.keys.d}>D</span></div></div>
 <div class="mouse"><span class:pressed={!preview&&frame?.keys.lmb}>LMB<small>{preview?0:frame?.left_cps??0} CPS</small></span><span class:pressed={!preview&&frame?.keys.rmb}>RMB<small>{preview?0:frame?.right_cps??0} CPS</small></span></div>
 {:else if module.id==='coordinates'}<span>XYZ</span><b>{preview?'128 / 64 / −256':internal?`${Math.floor(internal.x)} / ${Math.floor(internal.y)} / ${Math.floor(internal.z)}`:'Requiere integración del juego'}</b>
 {:else if module.id==='direction'}<span>DIRECCIÓN</span><b>{preview?'N · NORTE':internal?.direction?.toUpperCase()||'—'}</b>
 {:else if module.id==='armor'}<div class="equipment">{#each equipment as item}{@const percent=durabilityPercent(item)}<div title={`${item.name} · ${percent!=null?`${item.remaining} de ${item.maximum} · ${percent}%`:'Sin desgaste'}`} class:low={percent!=null&&percent<=15}>{#if item.item_id&&icons[item.item_id]&&!brokenIcons[item.item_id]}<img src={icons[item.item_id]} alt={item.name} onerror={()=>{if(item.item_id)brokenIcons[item.item_id]=true;}}/>{:else}<HudEquipmentIcon kind={equipmentKind(item)}/>{/if}<b>{percent!=null?`${percent}%`:'—'}</b><span>{item.name}</span></div>{/each}{#if !equipment.length}<span>Sin equipamiento</span>{/if}</div>
 {:else if module.id==='ping'}<span>MS</span><b>{preview?'28':internal?.ping!=null?Math.round(internal.ping):'—'}</b>
 {:else if module.id==='minimap'}<span>MINIMAPA</span>{#if !preview&&internal?.map}<div class="map"><canvas use:paintMap={internal.map.pixels} aria-label="Superficie del mundo recibida desde Minecraft"></canvas><span>N</span><i style:left={`${Math.max(2,Math.min(94,internal.x-internal.map.origin_x))}px`} style:top={`${Math.max(2,Math.min(94,internal.z-internal.map.origin_z))}px`} style:transform={`rotate(${directions[internal.direction]??0}deg)`}></i></div>{:else}<b>{preview?'Superficie · TP Companion':'Requiere integración del juego'}</b>{/if}
 {:else}<span>HORA LOCAL</span><b>{preview?'12:45':clock}</b>{/if}
 </div>
 </div>{/each}{#if drag}<i class="guide vertical"></i><i class="guide horizontal"></i>{/if}
</div>
<style>
.equipment{display:grid;gap:5px}.equipment>div{display:grid;grid-template-columns:24px auto;align-items:center;gap:2px 9px}.equipment img{width:24px;height:24px;object-fit:contain;image-rendering:pixelated}.equipment b{font-weight:500;text-align:right}.equipment span{grid-column:1/-1;font:9px/1.2 Segoe UI,sans-serif;color:inherit;opacity:.7;max-width:170px;overflow:hidden;text-overflow:ellipsis}.equipment .low{color:#ff8e8e}

.hud-layer.editing{pointer-events:auto}.editing .widget{cursor:grab;touch-action:none;outline:1px dashed #ffffff50}.editing .widget:active{cursor:grabbing}.guide{position:absolute;background:#a855f780;pointer-events:none}.guide.vertical{top:0;bottom:0;width:1px;left:50%}.guide.horizontal{left:0;right:0;height:1px;top:50%}.chroma .widget{animation:hud-chroma 8s linear infinite}@keyframes hud-chroma{0%,100%{border-color:#a855f7}25%{border-color:#06b6d4}50%{border-color:#10b981}75%{border-color:#ef4444}}

.divider{height:16px;width:1px;background:currentColor;opacity:.35}
@font-face{font-family:TPHudPixel;src:url("/fonts/Monocraft.ttf") format("truetype");font-display:swap}
 .widget.unavailable{border-color:#b38c6045;color:#d9be99;font-size:10px}.widget.unavailable>span{color:#bda784}
 .hud-layer{position:absolute;inset:0;pointer-events:none;user-select:none;color:var(--hud-text,#f3edf8);font-family:var(--font-display,TPHudPixel,monospace)}.widget-position{position:absolute;max-width:80%}.widget{display:flex;align-items:center;gap:10px;max-width:360px;font-size:12px;line-height:1.7;padding:7px 10px;background:color-mix(in srgb,var(--hud-bg,#0b0b0f) 85%,transparent);border:1px solid color-mix(in srgb,var(--hud-accent,#a855f7) 45%,transparent);box-shadow:0 2px 6px #0004;opacity:var(--opacity);text-shadow:1px 1px #000;white-space:nowrap}.widget>span{font-size:9px;color:inherit;opacity:.8}.widget>b{font-weight:500}.no-background .widget{background:transparent;border-color:transparent;box-shadow:none}.keys{display:grid;gap:4px;padding:5px}.movement{display:grid;justify-items:center;gap:4px}.movement>div{display:flex;gap:4px}.movement span,.mouse>span{display:grid;place-content:center;min-width:30px;min-height:28px;border:1px solid #ffffff28;background:#0006;color:#eee4f8;transition:background .04s,color .04s}.mouse{display:flex;gap:4px}.mouse>span{min-width:48px;padding:3px}.mouse small{font:9px/1.7 'Segoe UI',sans-serif;color:inherit}.movement .pressed,.mouse>.pressed{background:var(--hud-accent,#b46df3);color:#100817;border-color:#efbfff;text-shadow:none;box-shadow:0 0 10px #a855f744}
.map{position:relative;width:96px;height:96px;overflow:hidden;border:1px solid #c299e555}.map canvas{width:96px;height:96px;image-rendering:pixelated}.map>span{position:absolute;top:2px;left:44px;font-size:9px;color:white;text-shadow:1px 1px black}.map>i{position:absolute;width:0;height:0;border-left:3px solid transparent;border-right:3px solid transparent;border-bottom:6px solid #ffb6fb;filter:drop-shadow(0 1px 1px #000);transform-origin:center}.hud-layer.uniform{font-family:Consolas,monospace}.no-background .movement span,.no-background .mouse>span{background:transparent}.no-background .movement .pressed,.no-background .mouse>.pressed{background:#b46df3}</style>
