import { derived, writable } from 'svelte/store';
export type GameSessionSnapshot={running:boolean;active_games:number;username:string;version:string;server:string};
const state=writable<GameSessionSnapshot>({running:false,active_games:0,username:'',version:'',server:''});
const preparing=writable(false);
export const gameSession={subscribe:state.subscribe};
export const isGameRunning=derived([state,preparing],([$state,$preparing])=>$state.running||$preparing);
export const setGameSessionState=(snapshot:GameSessionSnapshot)=>state.set(snapshot);
export const setGamePreparing=(value:boolean)=>preparing.set(value);
