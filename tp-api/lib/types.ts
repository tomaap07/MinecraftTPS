export type MotdSpan={text:string;color:string;bold:boolean;italic:boolean;underlined:boolean;strikethrough:boolean};
export type ServerState={is_online:boolean;players_online:number;players_max:number;version_text:string;motd_clean:string;motd:MotdSpan[];favicon_url:string;latency_ms:number;protocol:number};
export type ServerRow=ServerState&{id:string;name:string;ip:string;port:number;category:string;region:string;language:string;is_featured:boolean;featured_until:string|null;sponsor_url:string|null;last_pinged_at:string|null;created_at:string;priority_tier:number;fail_streak:number;lease_token?:string};
export type PublicServer=Omit<ServerRow,'priority_tier'|'fail_streak'|'lease_token'>;
export const PUBLIC_COLUMNS='id,name,ip,port,category,region,language,version_text,is_online,players_online,players_max,motd_clean,motd,favicon_url,latency_ms,protocol,is_featured,featured_until,sponsor_url,last_pinged_at,created_at';
