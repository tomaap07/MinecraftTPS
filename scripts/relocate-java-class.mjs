
export function relocateClass(input){
 if(input.readUInt32BE(0)!==0xcafebabe)throw new Error('Invalid Java class');
 const parts=[input.subarray(0,10)];let offset=10;
 for(let index=1;index<input.readUInt16BE(8);index++){
  const start=offset,tag=input[offset++];
  if(tag===1){const size=input.readUInt16BE(offset);offset+=2;let value=input.subarray(offset,offset+size).toString('latin1');offset+=size;
   value=value.replaceAll('org/objectweb/asm','dev/tplauncher/shaded/asm').replaceAll('org.objectweb.asm','dev.tplauncher.shaded.asm');const bytes=Buffer.from(value,'latin1');if(bytes.length>65535)throw new Error('Constant too large');const header=Buffer.alloc(3);header[0]=1;header.writeUInt16BE(bytes.length,1);parts.push(header,bytes);
  }else{const size=({3:4,4:4,5:8,6:8,7:2,8:2,9:4,10:4,11:4,12:4,15:3,16:2,17:4,18:4,19:2,20:2})[tag];if(!size)throw new Error('Unknown Java constant tag '+tag);offset+=size;if(tag===5||tag===6)index++;parts.push(input.subarray(start,offset));}
 }parts.push(input.subarray(offset));return Buffer.concat(parts);
}
