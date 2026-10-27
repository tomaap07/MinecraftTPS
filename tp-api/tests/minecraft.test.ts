import test from 'node:test';
import assert from 'node:assert/strict';
import {isPublicAddress,parseAddress,motdSpans,stateFromJson,varint,readVarint,validFavicon} from '../lib/minecraft.js';
import {categoryFilter} from '../lib/filters.js';

test('server addresses preserve explicit ports and reject URLs or malformed inputs',()=>{
  assert.deepEqual(parseAddress('MC.HYPIXEL.NET'),{host:'mc.hypixel.net',port:25565,explicitPort:false});
  assert.deepEqual(parseAddress('[2606:4700:4700::1111]:25566'),{host:'2606:4700:4700::1111',port:25566,explicitPort:true});
  assert.equal(parseAddress('example.org:25566',25566).port,25566);
  for(const value of ['https://localhost/','127.0.0.1/path','user@example.org','example.org:0','example.org:99999','example.org:-1','local name'])assert.throws(()=>parseAddress(value));
  assert.throws(()=>parseAddress('example.org:25566',25565));
});
test('launcher category slugs map to catalogue category labels',()=>{
  assert.equal(categoryFilter('minigames'),'Minijuegos');assert.equal(categoryFilter('anarchy'),'Anarquía');
  assert.equal(categoryFilter('SkyBlock'),'SkyBlock');assert.equal(categoryFilter('Custom mode'),'Custom mode');
});
test('SSRF policy rejects internal, documentation, mapped and tunneled IPs',()=>{
  for(const value of ['127.0.0.1','10.0.0.5','192.168.1.1','169.254.169.254','172.16.0.1','100.64.1.2','0.0.0.0','203.0.113.1','198.18.0.2','224.0.0.1','::1','::','fe80::1','fc00::1','::ffff:127.0.0.1','2001:db8::1','2002:a00:1::1'])assert.equal(isPublicAddress(value),false,value);
  for(const value of ['1.1.1.1','8.8.8.8','2606:4700:4700::1111','2001:4860:4860::8888'])assert.equal(isPublicAddress(value),true,value);
});
test('VarInt roundtrip and incomplete/malformed packets',()=>{
  for(const value of [0,1,127,128,255,2097151,0xffffffff])assert.equal(readVarint(varint(value))?.value,value);
  assert.equal(readVarint(Buffer.from([128])),null);
  assert.throws(()=>readVarint(Buffer.from([255,255,255,255,255])));
});
test('MOTD retains style and line breaks without accepting arbitrary CSS',()=>{
  const result=motdSpans({text:'Hola ',color:'green',extra:[{text:'Mundo\n',bold:true},{text:'§cPvP',color:'url(evil)'}]});
  assert.equal(result.map(span=>span.text).join(''),'Hola Mundo\nPvP');
  assert.equal(result[0].color,'#55ff55');assert.equal(result[1].bold,true);assert.equal(result[2].color,'#ff5555');
  assert.ok(motdSpans('A'.repeat(5000)).map(span=>span.text).join('').length<=2048);
});
test('only bounded 64x64 PNG favicons are accepted',()=>{
  assert.equal(validFavicon('https://host/icon.png'),'');assert.equal(validFavicon('data:image/png;base64,AAAA'),'');
  const png=Buffer.alloc(33);Buffer.from([137,80,78,71,13,10,26,10]).copy(png);png.write('IHDR',12,'ascii');png.writeUInt32BE(64,16);png.writeUInt32BE(64,20);
  assert.match(validFavicon(`data:image/png;base64,${png.toString('base64')}`),/^data:image\/png;base64,/);
  png.writeUInt32BE(16000,16);assert.equal(validFavicon(`data:image/png;base64,${png.toString('base64')}`),'');
});
test('SLP state requires actual Minecraft fields and bounds server counts',()=>{
  assert.throws(()=>stateFromJson({success:true},2));
  const status=stateFromJson({players:{online:12,max:100},version:{name:'1.21',protocol:767},description:'§aServidor'},24.4);
  assert.equal(status.players_online,12);assert.equal(status.motd_clean,'Servidor');assert.equal(status.latency_ms,24);
});
