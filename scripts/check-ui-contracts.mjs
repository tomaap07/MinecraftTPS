import assert from 'node:assert/strict';
import { readFile, readdir, mkdir, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { pathToFileURL } from 'node:url';
import ts from 'typescript';
import { parse } from 'svelte/compiler';

const root = resolve(import.meta.dirname, '..');
const source = await readFile(join(root, 'src/routes/+page.svelte'), 'utf8');
parse(source, { modern: true });
assert.match(source, /class:home-content=\{activeTab === "jugar"\}/);
assert.match(source, /\.content\.home-content\s*\{\s*display:\s*block\s*;/);
assert.match(source, /\.play-layout\s*\{[^}]*height:\s*auto\s*;[^}]*flex:\s*none\s*;/);
assert.match(source, /\.launch-panel \.bottom-bar\s*\{[^}]*display:\s*grid\s*;[^}]*height:\s*auto\s*;[^}]*max-height:\s*none\s*;/);
assert.match(source, /\.launch-panel \.play-btn\s*\{[^}]*justify-content:\s*center\s*;[^}]*min-height:\s*60px\s*;/);
assert.ok(source.indexOf('class="home-secondary"') > source.indexOf('class="play-btn"'));
assert.ok(source.indexOf('<NewsHub ') > source.indexOf('class="home-secondary"'));
console.log('Inicio: flujo normal, altura automática y bloque de noticias después de Jugar.');



const output = join(root, '.codex-tmp', 'ui-contracts-locales');
await mkdir(output, { recursive: true });
for (const name of await readdir(join(root, 'src/lib/locales'))) {
  if (!name.endsWith('.ts')) continue;
  const text = await readFile(join(root, 'src/lib/locales', name), 'utf8');
  const compiled = ts.transpileModule(text, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext } }).outputText
    .replace(/from (['"])(\.\/[^'"]+)\1/g, (_, quote, path) => `from ${quote}${path}.mjs${quote}`);
  await writeFile(join(output, name.replace(/\.ts$/, '.mjs')), compiled);
}
const { english, partialCatalogs } = await import(pathToFileURL(join(output, 'catalog.mjs')).href);
const keys = Object.keys(english);
for (const [code, catalog] of Object.entries(partialCatalogs)) {
  const missing = keys.filter(key => typeof catalog[key] !== 'string' || !catalog[key].trim());
  assert.deepEqual(missing, [], `Faltan traducciones en ${code}`);
}
console.log(`Idiomas: ${keys.length} textos en ${Object.keys(partialCatalogs).length + 4} idiomas, sin entradas faltantes.`);
