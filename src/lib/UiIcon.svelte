<script lang="ts">
  let { name, size = 18 }: { name: string; size?: number } = $props();
  const paths: Record<string,string> = {
    play: 'M8 5v14l11-7z', cube: 'm12 3 9 5v8l-9 5-9-5V8z M3 8l9 5 9-5 M12 13v8',
    archive:'M3 4h18v5H3z M5 9v12h14V9 M9 13h6', flask:'M9 3h6 M10 3v6l-6 10a2 2 0 0 0 2 3h12a2 2 0 0 0 2-3L14 9V3 M7 15h10',
    maximize:'M4 4h16v16H4z',restore:'M8 8h12v12H8z M4 16V4h12',

    creeper: 'M6 5h12v14H6z M9 9h6v6H9z M2 8h4 M2 16h4 M18 8h4 M18 16h4 M9 2v3 M15 19v3',
    enderman: 'm12 2 8 5v10l-8 5-8-5V7z M8 9l4-3 4 3v6l-4 3-4-3z',
    anvil: 'M3 6h18l-4 5h-4v4l5 3v3H5v-3l5-3v-4H7z M3 3h18v3H3z',
    chip: 'M6 6h12v12H6z M9 9h6v6H9z M9 2v4 M15 2v4 M9 18v4 M15 18v4 M2 9h4 M2 15h4 M18 9h4 M18 15h4',
    trophy: 'M8 3h8v5a4 4 0 0 1-8 0V3z M8 5H4v3a4 4 0 0 0 5 4 M16 5h4v3a4 4 0 0 1-5 4 M12 12v6 M8 21h8 M9 18h6v3H9z',
    grid: 'M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z',
    user: 'M20 21v-2a7 7 0 0 0-14 0v2 M12 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8z',
    pose: 'M12 3a2 2 0 1 0 0 4 2 2 0 0 0 0-4z M4 9l8 3 8-3 M12 12v5 M7 22l5-5 5 5',
    terminal: 'm4 6 6 6-6 6 M13 18h7', folder: 'M3 7V5h6l2 2h10v13H3z',
    image: 'M3 3h18v18H3z m0 14 6-6 5 5 3-3 4 4 M15 7h.01', clock: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z M12 7v5l3 2',
    settings: 'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z M9 3h6l1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1z',
    arrow: 'm9 5 7 7-7 7', close: 'm6 6 12 12 M18 6 6 18', download: 'M12 3v12 m-5-5 5 5 5-5 M4 16v5h16v-5',
    copy: 'M8 8h13v13H8z M16 8V3H3v13h5', refresh: 'M20 7v5h-5 M4 17v-5h5 M5 8a8 8 0 0 1 13-3l2 3 M4 16l2 3a8 8 0 0 0 13-3'
    ,trash: 'M3 6h18 M9 6V3h6v3 M5 6l1 15h12l1-15 M10 10v7 M14 10v7'
    ,server: 'M3 3h18v7H3z M3 14h18v7H3z M7 6h.01 M7 17h.01 M11 6h6 M11 17h6'
    ,check: 'm4 12 5 5L20 6'
    ,plus: 'M12 4v16 M4 12h16'
    ,minus: 'M4 12h16'
    ,search: 'M10 3a7 7 0 1 0 0 14 7 7 0 0 0 0-14z m5 12 6 6'
    ,filter: 'M3 4h18l-7 8v7l-4 2v-9z'
    ,globe: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z M3 12h18 M12 3c-5 5-5 13 0 18 M12 3c5 5 5 13 0 18'
    ,external: 'M14 3h7v7 M21 3l-11 11 M10 3H3v18h18v-7'
    ,link: 'm10 14 4-4 M9 16l-2 2a4 4 0 0 1-6-6l5-5a4 4 0 0 1 6 0 M15 8l2-2a4 4 0 0 1 6 6l-5 5a4 4 0 0 1-6 0'
    ,info: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z M12 11v6 M12 7h.01'
    ,eye: 'M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z'
    ,'eye-off': 'm3 3 18 18 M9 5a12 12 0 0 1 3 0c6 0 10 7 10 7a20 20 0 0 1-4 5 M6 6a20 20 0 0 0-4 6s4 7 10 7a12 12 0 0 0 5-1 M10 10a3 3 0 0 0 4 4'
    ,pause: 'M6 4h4v16H6z M14 4h4v16h-4z'
    ,upload: 'M12 15V3 m-5 5 5-5 5 5 M4 16v5h16v-5'
    ,zip: 'M5 3h10l4 4v14H5z M15 3v5h4 M10 3v2 M12 5v2 M10 7v2 M12 9v2 M10 12h3v4h-3z'
    ,save: 'M4 3h14l3 3v15H3V3z M7 3v6h10V3 M7 21v-8h10v8'
    ,shield: 'm12 3 9 4v5c0 5-5 8-9 10-4-2-9-5-9-10V7z m-4 9 3 3 5-6'
    ,sliders: 'M5 3v8 M5 15v6 M3 11h4v4H3z M12 3v3 M12 10v11 M10 6h4v4h-4z M19 3v12 M19 19v2 M17 15h4v4h-4z'
    ,volume: 'M3 9h5l5-5v16l-5-5H3z M17 8a6 6 0 0 1 0 8 M20 5a10 10 0 0 1 0 14'
    ,java: 'M5 10h11v6a4 4 0 0 1-4 4H9a4 4 0 0 1-4-4z M16 11h2a3 3 0 0 1 0 6h-2 M8 2c-3 3 3 3 0 6 M13 2c-3 3 3 3 0 6 M3 23h15'
    ,skin: 'm8 3-6 4 3 6 3-2v10h8V11l3 2 3-6-6-4c-1 3-7 3-8 0z'
    ,users: 'M9 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8z M2 21v-3a7 7 0 0 1 14 0v3 M16 4a4 4 0 0 1 0 8 M18 14a5 5 0 0 1 4 5v2'
    ,bolt: 'm13 2-9 12h7l-1 8L21 9h-8z'
    ,book: 'M3 3h7l2 2 2-2h7v17h-7l-2 2-2-2H3z M12 5v17 M6 7h3 M6 11h3 M15 7h3 M15 11h3'
    ,news: 'M3 3h15v17H3z M18 7h3v13H3 M6 6h9 M6 10h3v4H6z M12 10h3 M12 14h3 M6 17h9'
    ,left: 'm15 5-7 7 7 7'
    ,down: 'm5 9 7 7 7-7'
    ,home: 'm2 11 10-9 10 9 M5 9v13h14V9 M10 22v-8h4v8'
    ,compass: 'M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z m5 5-3 7-7 3 3-7z'
    ,plugins: 'M4 4h5V2h4v2h7v5h2v4h-2v7h-7v-2H9v2H4v-7H2V9h2z M9 9h6v6H9z'
    ,sun: 'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z M12 2v2 M12 20v2 M2 12h2 M20 12h2 M5 5l1.5 1.5 M17.5 17.5 19 19 M5 19l1.5-1.5 M17.5 6.5 19 5'
    ,layers: 'm12 3 10 5-10 5L2 8z M2 12l10 5 10-5 M2 16l10 5 10-5'
    ,map: 'm3 5 6-2 6 2 6-2v16l-6 2-6-2-6 2z M9 3v16 M15 5v16'
    ,gauge: 'M4 19a10 10 0 1 1 16 0z M12 15l5-6 M7 11l-1-1 M12 7V5 M17 11l1-1 M7 16H5'
    ,monitor: 'M3 4h18v13H3z M8 21h8 M12 17v4 M7 10h2l2-3 3 7 2-4h2'
    ,keyboard: 'M2 5h20v14H2z M5 8h.01 M9 8h.01 M13 8h.01 M17 8h2 M5 11h.01 M9 11h.01 M13 11h.01 M17 11h2 M5 15h2 M10 15h9'
    ,network: 'M4 18v3 M9 14v7 M14 9v12 M19 3v18'
    ,locate: 'M12 2v4 M12 18v4 M2 12h4 M18 12h4 M12 5a7 7 0 1 0 0 14 7 7 0 0 0 0-14z M12 10a2 2 0 1 0 0 4 2 2 0 0 0 0-4z'
    ,armor: 'm8 3-5 3 2 6 3-2v11h8V10l3 2 2-6-5-3c0 4-8 4-8 0z M10 12h4 M10 16h4'
    ,calendar: 'M3 5h18v16H3z M7 2v6 M17 2v6 M3 10h18 M7 14h3 M14 14h3 M7 18h3'
    ,leaf: 'M21 3c-11-1-18 3-18 10a8 8 0 0 0 8 8c7 0 11-7 10-18z M3 21 16 8 M10 14v-4 M10 14h4'
    ,wind: 'M3 8h12a3 3 0 1 0-3-3 M2 12h17a3 3 0 1 1-3 3 M4 16h5a3 3 0 1 1-3 3'
    ,hand: 'M8 13V5a2 2 0 0 1 4 0v7 M12 11V8a2 2 0 0 1 4 0v5 M16 11a2 2 0 0 1 4 0v5c0 4-2 6-6 6h-3c-2 0-3-1-4-3l-4-6a2 2 0 0 1 3-2l2 2'
    ,text: 'M3 5h14 M10 5v16 M6 21h8 M16 13h6 M19 10v11'
    ,chat: 'M3 3h18v14H8l-5 4z M7 7h10 M7 11h7'
    ,memory: 'M3 6h18v12H3z M6 9h4v5H6z M14 9h4v5h-4z M6 18v3 M10 18v3 M14 18v3 M18 18v3'
    ,rewind: 'm11 5-9 7 9 7z m11 0-9 7 9 7z'
    ,camera: 'M3 7h4l2-3h6l2 3h4v14H3z M12 10a4 4 0 1 0 0 8 4 4 0 0 0 0-8z'
    ,scan: 'M3 8V3h5 M16 3h5v5 M21 16v5h-5 M8 21H3v-5 M7 7h10v10H7z'
    ,flame: 'M13 2c2 7-4 7-2 12 3-1 4-4 4-6 7 6 6 14-3 14S1 13 8 7c-1 5 1 5 2 5-1-4 3-6 3-10z'
    ,target: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z M12 7a5 5 0 1 0 0 10 5 5 0 0 0 0-10z M12 11v2 M11 12h2'
    ,sparkles: 'm9 2 2 6 6 2-6 2-2 6-2-6-6-2 6-2z M19 14l1 3 3 1-3 1-1 3-1-3-3-1 3-1z'
    ,tag: 'M3 3h8l10 10-8 8L3 11z M7 7h.01'
    ,bomb: 'M11 7a8 8 0 1 0 0 16 8 8 0 0 0 0-16z M15 8l2-4 3 1 M20 1v1 M23 4h-1 M22 7l-1-1'
    ,heart: 'M12 21 3 12C-3 4 7 0 12 7c5-7 15-3 9 5z'
    ,food: 'M4 3v7 M7 3v7 M10 3v7 M4 7h6 M7 10v12 M18 3c-4 5-4 9 1 9V3v19'
    ,bubbles: 'M8 11a5 5 0 1 0 0 10 5 5 0 0 0 0-10z M17 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8z M19 16a2 2 0 1 0 0 4 2 2 0 0 0 0-4z'
    ,inventory: 'M3 3h18v18H3z M3 9h18 M3 15h18 M9 3v18 M15 3v18'
    ,bow: 'M4 3c20 0 20 18 0 18l6-9z M2 12h20 M18 8l4 4-4 4'
    ,rocket: 'M15 3h6v6l-9 9-6-6z M6 12H3v6l3-1 M12 18v3h6v-6 M15 7h.01 M3 21l3-3'
    ,sword: 'm14 3 7-1-1 7-9 9-5-5z M3 16l5 5 M2 22l4-4 M9 13l7-7'
    ,ruler: 'm3 16 13-13 5 5L8 21z M13 6l2 2 M10 9l2 2 M7 12l2 2 M4 15l2 2'
  };
</script>
<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d={paths[name] ?? paths.cube} /></svg>
