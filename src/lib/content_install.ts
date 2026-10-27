export type ContentKind = 'mod' | 'modpack' | 'shader' | 'resourcepack';
export type InstallProject = {provider:string; id:string; title:string; description:string; icon_url:string; project_type:string; url:string};
export type ContentVersion = {id:string; name:string; number:string; release_type:string; game_versions:string[]; loaders:string[]; published:string; filename:string; size:number};
export type InstallTarget = {id:string; name:string; game_version:string; loader:string; version_id:string; pack_id:string|null; shader_loader:string|null};
export type CompatibleTarget = {target:InstallTarget; version:ContentVersion};
const modLoaders = new Set(['fabric','forge','neoforge','quilt']);
export function compatibleVersion(version:ContentVersion, target:InstallTarget, kind:string):boolean {
  if (!version.id || !version.filename || !version.game_versions.includes(target.game_version)) return false;
  const engines = version.loaders.map(value=>value.toLowerCase());
  const shader = target.shader_loader?.toLowerCase();
  const supports = (engine:string) => engine===target.loader || engine===shader || (engine==='iris'&&shader==='oculus');
  if (kind==='mod') return modLoaders.has(target.loader) && engines.includes(target.loader);
  if (kind==='shader') return !!shader && (!engines.length || engines.some(supports));
  if (kind==='resourcepack') return !engines.length || engines.includes('minecraft') || engines.includes('vanilla') || engines.some(supports);
  return false;
}
export function newestVersions<T extends ContentVersion>(versions:T[]):T[] {
  return versions.filter(version=>version.id&&version.filename).map((version,index)=>({version,index,time:Date.parse(version.published)||0}))
    .sort((a,b)=>b.time-a.time||a.index-b.index).map(row=>row.version);
}
export function compatibleTargets(versions:ContentVersion[], targets:InstallTarget[], kind:string, file='', game='all', loader='all'):CompatibleTarget[] {
  const files=newestVersions(versions).filter(version=>(!file||version.id===file)&&(game==='all'||!game||version.game_versions.includes(game))&&(loader==='all'||!loader||version.loaders.includes(loader)));
  return targets.flatMap(target=>{
    if(game&&game!=='all'&&target.game_version!==game)return [];
    const version=files.find(version=>compatibleVersion(version,target,kind));
    return version?[{target,version}]:[];
  });
}
export function installRequest(project:InstallProject, version:ContentVersion, target:InstallTarget|undefined, username:string) {
  if(project.project_type!=='modpack'&&(!target||!compatibleVersion(version,target,project.project_type)))throw new Error('El archivo no es compatible con la instalación seleccionada.');
  return {provider:project.provider,projectId:project.id,fileId:version.id,projectType:project.project_type,packId:project.project_type==='modpack'?null:target!.pack_id,username,versionId:project.project_type==='modpack'?'':target!.version_id};
}
