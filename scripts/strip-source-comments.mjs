import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from 'node:fs';
import { dirname, extname, relative, resolve, sep } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import ts from 'typescript';
import { compile, parse } from 'svelte/compiler';
import postcss from 'postcss';

const root = realpathSync(resolve(dirname(fileURLToPath(import.meta.url)), '..'));
const languageByExtension = new Map([
  ['.ts', 'typescript'], ['.tsx', 'typescript'], ['.js', 'javascript'], ['.jsx', 'javascript'],
  ['.mjs', 'javascript'], ['.cjs', 'javascript'], ['.svelte', 'svelte'], ['.html', 'html'],
  ['.css', 'css'], ['.rs', 'rust'], ['.java', 'java'], ['.gradle', 'groovy'],
  ['.py', 'python'], ['.ps1', 'powershell'], ['.nsi', 'nsis'], ['.nsh', 'nsis'],
  ['.sh', 'shell'], ['.sql', 'sql'], ['.toml', 'toml'], ['.properties', 'properties']
]);
const excludedSegments = new Set(['.git', 'node_modules', 'target', 'build', 'dist', '.svelte-kit', 'vendor', 'generated', 'coverage', 'attachments']);
const thirdPartyFiles = new Set(['tp-companion/gradlew', 'tp-companion/gradlew.bat', 'tp-companion/gradle/wrapper/gradle-wrapper.properties']);

function run(command, args, input) {
  const result = spawnSync(command, args, { cwd: root, input, encoding: 'utf8', windowsHide: true, maxBuffer: 32 * 1024 * 1024 });
  if (result.error || result.status !== 0) throw new Error((result.error?.message || result.stderr || `${command} exited ${result.status}`).slice(0, 1800));
  return result.stdout;
}

function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function lineAt(text, offset) { return text.slice(0, offset).split('\n').length; }
function preserveReason(comment, source, start, language) {
  if (/SPDX-License-Identifier|@license\b|@preserve\b|copyright\s+(?:©|\(c\)|\d{4})|licensed\s+under|all rights reserved|\bbased on\b.{0,160}\b(?:MIT|Apache|BSD|GPL)|\bsee\b.{0,100}\b(?:NOTICE|LICENSE)[-\w.]*\b/i.test(comment)) return 'legal-notice';
  if (/svelte-ignore\b|@ts-(?:check|nocheck|ignore|expect-error)\b|eslint(?:-disable|-enable|\s)|^\/\*+\s*(?:global|globals)\s+[\w$]|istanbul\s+ignore|c8\s+ignore|prettier-ignore|@vite-ignore|webpack(?:ChunkName|Ignore|Mode|Prefetch|Preload)|[@#]__PURE__|[@#]__NO_SIDE_EFFECTS__|sourceMappingURL\s*=|sourceURL\s*=|^\/\/\/\s*<(?:reference|amd-)/i.test(comment)) return 'tool-directive';
  if (start <= (source.charCodeAt(0) === 0xfeff ? 1 : 0) && /^#!/.test(comment)) return 'executable-shebang';
  if (language === 'python' && lineAt(source, start) <= 2 && /coding[:=]\s*[-\w.]+/.test(comment)) return 'encoding-directive';
  if (language === 'powershell' && /^#requires\b/i.test(comment.trim())) return 'powershell-requires';
  if (language === 'shell' && /^#\s*shellcheck\b/.test(comment)) return 'shellcheck-directive';
  return null;
}

function deduplicate(ranges) {
  const ordered = [...new Map(ranges.map(range => [`${range.start}:${range.end}`, range])).values()].sort((a, b) => a.start - b.start || b.end - a.end);
  const result = [];
  for (const range of ordered) {
    if (range.end <= range.start) continue;
    const previous = result.at(-1);
    if (previous && range.start < previous.end) {
      if (range.end <= previous.end) continue;
      throw new Error('Overlapping lexer comment ranges');
    }
    result.push(range);
  }
  return result;
}

function applyRanges(source, ranges, language) {
  let output = '', cursor = 0;
  const retained = [];
  for (const range of deduplicate(ranges)) {
    const comment = source.slice(range.start, range.end);
    const reason = preserveReason(comment, source, range.start, language);
    let prefix = source.slice(cursor, range.start), replacement = comment, end = range.end;
    if (!reason) {
      if (range.markup) replacement = '';
      else {
        const startOfLine = source.lastIndexOf('\n', range.start - 1) + 1;
        let endOfLine = source.indexOf('\n', range.end); if (endOfLine < 0) endOfLine = source.length;
        const aloneAtStart = /^\s*$/.test(source.slice(startOfLine, range.start));
        const aloneAtEnd = /^\s*$/.test(source.slice(range.end, endOfLine));
        replacement = comment.replace(/[^\r\n]/g, '') + (aloneAtEnd ? '' : ' ');
        if (aloneAtStart && aloneAtEnd && startOfLine >= cursor) prefix = source.slice(cursor, startOfLine);
        else if (aloneAtEnd) prefix = prefix.replace(/[ \t]+$/, '');
        if (aloneAtEnd) end = endOfLine;
      }
    }
    output += prefix + replacement;
    if (reason) retained.push({ reason, line: lineAt(source, range.start) });
    cursor = end;
  }
  return { output: output + source.slice(cursor), retained };
}

function typeScriptAst(source, filename) {
  const kind = /\.[cm]?jsx?$/.test(filename) ? (filename.endsWith('x') ? ts.ScriptKind.JSX : ts.ScriptKind.JS) : (filename.endsWith('x') ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
  const ast = ts.createSourceFile(filename, source, ts.ScriptTarget.Latest, true, kind);
  if (ast.parseDiagnostics.length) throw new Error(`TypeScript parse: ${ts.flattenDiagnosticMessageText(ast.parseDiagnostics[0].messageText, ' ')}`);
  return ast;
}

function typeScriptComments(source, filename) {
  const ast = typeScriptAst(source, filename);
  const literals = [];
  const visit = node => {
    if (ts.isStringLiteral(node) || ts.isRegularExpressionLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node) || node.kind === ts.SyntaxKind.TemplateHead || node.kind === ts.SyntaxKind.TemplateMiddle || node.kind === ts.SyntaxKind.TemplateTail || node.kind === ts.SyntaxKind.JsxText) {
      literals.push({ start: node.getStart(ast), end: node.end });
    }
    ts.forEachChild(node, visit);
  };
  visit(ast);
  literals.sort((a, b) => a.start - b.start || b.end - a.end);
  const scanner = ts.createScanner(ts.ScriptTarget.Latest, false, ts.LanguageVariant.Standard, source);
  const ranges = [];
  let literalIndex = 0;
  for (let token = scanner.scan(); token !== ts.SyntaxKind.EndOfFileToken; token = scanner.scan()) {
    const start = scanner.getTokenPos();
    while (literalIndex < literals.length && literals[literalIndex].end <= start) literalIndex++;
    const literal = literals[literalIndex];
    if (literal && literal.start <= start && literal.end > start) { scanner.setTextPos(literal.end); continue; }
    if (token === ts.SyntaxKind.SingleLineCommentTrivia || token === ts.SyntaxKind.MultiLineCommentTrivia || token === ts.SyntaxKind.ShebangTrivia) ranges.push({ start, end: scanner.getTextPos() });
  }
  return ranges;
}

function typeScriptFingerprint(source, filename) {
  const ast = typeScriptAst(source, filename);
  const visit = node => {
    if (node.kind >= ts.SyntaxKind.FirstJSDocNode && node.kind <= ts.SyntaxKind.LastJSDocNode) return null;
    const children = node.getChildren(ast).map(visit).filter(value => value !== null);
    return children.length ? [node.kind, children] : [node.kind, node.getText(ast)];
  };
  return JSON.stringify(visit(ast));
}

function svelteComments(source) {
  const ast = parse(source, { modern: true });
  const ranges = (ast.comments || []).map(comment => ({ start: comment.start, end: comment.end }));
  for (const comment of ast.css?.comments || []) ranges.push({ start: comment.start, end: comment.end });
  const walk = node => {
    if (!node || typeof node !== 'object') return;
    if (node.type === 'Comment') ranges.push({ start: node.start, end: node.end, markup: true });
    for (const [key, value] of Object.entries(node)) {
      if (['parent', 'metadata', 'comments', 'leadingComments', 'trailingComments', 'loc'].includes(key)) continue;
      if (Array.isArray(value)) value.forEach(walk); else if (value && typeof value === 'object') walk(value);
    }
  };
  walk(ast.fragment);
  return ranges;
}

function svelteFingerprint(source, filename) {
  const compiled = compile(source, { filename, generate: 'client', dev: false, css: 'external', cssHash: () => 'tp-comment-equivalence' });
  return typeScriptFingerprint(compiled.js.code, 'compiled.js') + '\nCSS\n' + cssFingerprint(compiled.css?.code || '');
}

function htmlFingerprint(source) {
  const ast = parse(source, { modern: true });
  const ignored = new Set(['start', 'end', 'loc', 'name_loc', 'metadata', 'comments', 'leadingComments', 'trailingComments', 'innerComments', 'parent', 'raw']);
  const normalize = node => {
    if (Array.isArray(node)) {
      const values = node.map(normalize).filter(value => value !== null), merged = [];
      for (const value of values) {
        if (value?.type === 'Text' && merged.at(-1)?.type === 'Text') merged.at(-1).data += value.data;
        else merged.push(value);
      }
      return merged;
    }
    if (!node || typeof node !== 'object') return node;
    if (node.type === 'Comment') return null;
    const result = {};
    for (const key of Object.keys(node).sort()) if (!ignored.has(key) && key !== 'css') result[key] = normalize(node[key]);
    return result;
  };
  return JSON.stringify(normalize(ast)) + '\nCSS\n' + cssFingerprint(ast.css?.content.styles || '');
}

function quotedEnd(source, start, quote, escape = '\\', doubled = false) {
  let i = start + quote.length;
  while (i < source.length) {
    if (escape && source[i] === escape) { i += 2; continue; }
    if (source.startsWith(quote, i)) {
      if (doubled && source.startsWith(quote + quote, i)) { i += quote.length * 2; continue; }
      return i + quote.length;
    }
    i++;
  }
  throw new Error('Unterminated quoted literal');
}

function rustCharEnd(source, start) {
  let i = start + 1;
  if (source[i] === '\\') {
    if (source.startsWith('\\u{', i)) { const end = source.indexOf('}', i + 3); if (end < 0) return null; i = end + 1; }
    else if (source.startsWith('\\x', i)) i += 4;
    else i += 2;
  } else if (source[i] && !/[\r\n']/.test(source[i])) i += source.codePointAt(i) > 0xffff ? 2 : 1;
  return source[i] === "'" ? i + 1 : null;
}

function cStyleComments(source, language) {
  if (language === 'java' && /\\u+(?:002[fFaA]|000[aAdD]|002[27]|005[cC])/.test(source)) throw new Error('Java Unicode lexical delimiter escape requires explicit review');
  const ranges = [], literals = [];
  let i = 0;
  while (i < source.length) {
    if (language === 'rust' && (i === 0 || !/[\w]/.test(source[i - 1]))) {
      const raw = /^(?:br|cr|r)(#{0,255})"/.exec(source.slice(i));
      if (raw) {
        const delimiter = '"' + raw[1], end = source.indexOf(delimiter, i + raw[0].length);
        if (end < 0) throw new Error('Unterminated Rust raw literal');
        literals.push({ start: i, end: end + delimiter.length }); i = end + delimiter.length; continue;
      }
    }
    if (source[i] === '"' || source[i] === "'") {
      if (language === 'rust' && source[i] === "'") {
        const end = rustCharEnd(source, i);
        if (end) { literals.push({ start: i, end }); i = end; continue; }
        i++; continue;
      }
      const quote = ['java', 'groovy'].includes(language) && source.startsWith('"""', i) ? '"""' : source[i];
      const end = quotedEnd(source, i, quote);
      literals.push({ start: i, end }); i = end; continue;
    }
    if (source.startsWith('//', i)) {
      let end = source.indexOf('\n', i + 2); if (end < 0) end = source.length;
      ranges.push({ start: i, end }); i = end; continue;
    }
    if (source.startsWith('/*', i)) {
      const start = i; i += 2; let depth = 1;
      while (i < source.length && depth) {
        if (language === 'rust' && source.startsWith('/*', i)) { depth++; i += 2; }
        else if (source.startsWith('*/', i)) { depth--; i += 2; }
        else i++;
      }
      if (depth) throw new Error('Unterminated block comment');
      ranges.push({ start, end: i }); continue;
    }
    if (language === 'groovy' && (source.startsWith('$/', i) || source[i] === '/' && /[=(:,[]\s*$/.test(source.slice(0, i)))) throw new Error('Groovy slashy literal requires explicit review');
    i++;
  }
  return { ranges, literals };
}

function cssComments(source) {
  postcss.parse(source);
  const ranges = [], literals = [];
  let i = 0;
  while (i < source.length) {
    if (source[i] === '"' || source[i] === "'") { const end = quotedEnd(source, i, source[i]); literals.push({ start: i, end }); i = end; continue; }
    const url = /^url\s*\(/i.exec(source.slice(i));
    if (url && (i === 0 || !/[\w-]/.test(source[i - 1]))) {
      const start = i; i += url[0].length; let depth = 1;
      while (i < source.length && depth) {
        if (source[i] === '\\') i += 2;
        else if (source[i] === '"' || source[i] === "'") i = quotedEnd(source, i, source[i]);
        else if (source[i] === '(') { depth++; i++; }
        else if (source[i] === ')') { depth--; i++; }
        else i++;
      }
      if (depth) throw new Error('Unterminated CSS URL');
      literals.push({ start, end: i }); continue;
    }
    if (source.startsWith('/*', i)) { const end = source.indexOf('*/', i + 2); if (end < 0) throw new Error('Unterminated CSS comment'); ranges.push({ start: i, end: end + 2 }); i = end + 2; continue; }
    i++;
  }
  return { ranges, literals };
}

function lexicalFingerprint(source, lexed) {
  const comments = deduplicate(lexed.ranges), literals = [...lexed.literals].sort((a, b) => a.start - b.start);
  let i = 0, ci = 0, li = 0, output = '';
  while (i < source.length) {
    while (comments[ci]?.end <= i) ci++;
    while (literals[li]?.end <= i) li++;
    if (comments[ci]?.start === i) { i = comments[ci].end; continue; }
    if (literals[li]?.start === i) { output += `L${JSON.stringify(source.slice(i, literals[li].end))}`; i = literals[li].end; continue; }
    if (!/\s/.test(source[i])) output += source[i];
    i++;
  }
  return output;
}

function cssFingerprint(source) { return lexicalFingerprint(source, cssComments(source)); }

const pythonProgram = String.raw`import ast, io, json, sys, tokenize
s=json.load(sys.stdin)['source']
clean=s[1:] if s.startswith('\ufeff') else s
tree=ast.dump(ast.parse(clean), include_attributes=False)
comments=[{'start': list(t.start), 'end': list(t.end)} for t in tokenize.generate_tokens(io.StringIO(clean).readline) if t.type == tokenize.COMMENT]
print(json.dumps({'comments':comments,'fingerprint':tree}))`;

function pythonAnalysis(source) {
  const parsed = JSON.parse(run('python', ['-X', 'utf8', '-c', pythonProgram], JSON.stringify({ source })));
  const lines = source.split('\n'), offsets = [0];
  for (let i = 1; i < lines.length; i++) offsets.push(offsets[i - 1] + lines[i - 1].length + 1);
  const at = ([line, column]) => offsets[line - 1] + Array.from(lines[line - 1]).slice(0, column + (line === 1 && source.charCodeAt(0) === 0xfeff ? 1 : 0)).join('').length;
  return { ranges: parsed.comments.map(comment => ({ start: at(comment.start), end: at(comment.end) })), fingerprint: parsed.fingerprint };
}

const powershellProgram = String.raw`$ErrorActionPreference='Stop'; [Console]::InputEncoding=New-Object System.Text.UTF8Encoding($false); [Console]::OutputEncoding=New-Object System.Text.UTF8Encoding($false); $value=[Console]::In.ReadToEnd() | ConvertFrom-Json; $tokens=$null; $errors=$null; $ast=[System.Management.Automation.Language.Parser]::ParseInput($value.source,[ref]$tokens,[ref]$errors); if($errors.Count){throw $errors[0].Message}; $comments=@($tokens | Where-Object {$_.Kind -eq 'Comment'} | ForEach-Object {@{start=$_.Extent.StartOffset;end=$_.Extent.EndOffset}}); $signature=@($tokens | Where-Object {$_.Kind -notin @('Comment','NewLine','EndOfInput')} | ForEach-Object {@{kind=[string]$_.Kind;text=$_.Text}}); @{comments=$comments;fingerprint=($signature|ConvertTo-Json -Compress -Depth 10)} | ConvertTo-Json -Compress -Depth 10`;
function powershellAnalysis(source) {
  const value = JSON.parse(run('powershell', ['-NoProfile', '-NonInteractive', '-Command', powershellProgram], JSON.stringify({ source })));
  return { ranges: value.comments, fingerprint: value.fingerprint };
}

function simpleComments(source, language) {
  const ranges = [], literals = [];
  let i = 0;
  while (i < source.length) {
    if (language === 'nsis' && source.startsWith('{{', i)) {
      const delimiter = source.startsWith('{{{', i) ? '}}}' : '}}';
      const end = source.indexOf(delimiter, i + (delimiter.length === 3 ? 3 : 2));
      if (end < 0) throw new Error('Unterminated installer template directive');
      const range = { start: i, end: end + delimiter.length };
      if (source.startsWith('{{!', i)) ranges.push(range); else literals.push(range);
      i = range.end; continue;
    }
    if (language === 'sql') {
      const dollar = /^\$(?:[A-Za-z_][A-Za-z0-9_]*)?\$/.exec(source.slice(i));
      if (dollar) { const end = source.indexOf(dollar[0], i + dollar[0].length); if (end < 0) throw new Error('Unterminated SQL dollar literal'); literals.push({ start: i, end: end + dollar[0].length }); i = end + dollar[0].length; continue; }
    }
    if (source[i] === '"' || source[i] === "'" || language === 'nsis' && source[i] === '`') {
      const quote = language === 'toml' && source.startsWith(source[i].repeat(3), i) ? source[i].repeat(3) : source[i];
      const escape = language === 'nsis' ? '$' : language === 'sql' || language === 'toml' && source[i] === "'" ? '' : '\\';
      const end = quotedEnd(source, i, quote, escape, language === 'sql');
      literals.push({ start: i, end }); i = end; continue;
    }
    if (language === 'shell' && (source.startsWith('<<', i) || source.startsWith('$(', i) || source.startsWith('${', i))) throw new Error('Shell heredoc/substitution requires explicit parser review');
    const lineComment = language === 'sql' ? source.startsWith('--', i) : language === 'nsis' ? source[i] === ';' || source[i] === '#' : language === 'properties' ? (source[i] === '#' || source[i] === '!') && /^\s*$/.test(source.slice(source.lastIndexOf('\n', i - 1) + 1, i)) : source[i] === '#' && (language !== 'shell' || i === 0 || /[\s;|&()]/.test(source[i - 1]));
    if (lineComment) { let end = source.indexOf('\n', i); if (end < 0) end = source.length; ranges.push({ start: i, end }); i = end; continue; }
    if (['sql', 'nsis'].includes(language) && source.startsWith('/*', i)) {
      const start = i; let depth = 1; i += 2;
      while (i < source.length && depth) {
        if (language === 'sql' && source.startsWith('/*', i)) { depth++; i += 2; }
        else if (source.startsWith('*/', i)) { depth--; i += 2; }
        else i++;
      }
      if (depth) throw new Error('Unterminated block comment');
      ranges.push({ start, end: i }); continue;
    }
    i++;
  }
  return { ranges, literals };
}

function analyze(source, filename, language) {
  let ranges, fingerprint, beforeFingerprint;
  if (['typescript', 'javascript'].includes(language)) {
    ranges = typeScriptComments(source, filename); beforeFingerprint = typeScriptFingerprint(source, filename); fingerprint = candidate => typeScriptFingerprint(candidate, filename);
  } else if (language === 'svelte') {
    ranges = svelteComments(source); beforeFingerprint = svelteFingerprint(source, filename); fingerprint = candidate => svelteFingerprint(candidate, filename);
  } else if (language === 'html') {
    ranges = svelteComments(source); beforeFingerprint = htmlFingerprint(source); fingerprint = htmlFingerprint;
  } else if (language === 'css') {
    ranges = cssComments(source).ranges; beforeFingerprint = cssFingerprint(source); fingerprint = cssFingerprint;
  } else if (language === 'python' || language === 'powershell') {
    const parser = language === 'python' ? pythonAnalysis : powershellAnalysis;
    const parsed = parser(source); ranges = parsed.ranges; beforeFingerprint = parsed.fingerprint; fingerprint = candidate => parser(candidate).fingerprint;
  } else {
    const lexer = ['rust', 'java', 'groovy'].includes(language) ? text => cStyleComments(text, language) : text => simpleComments(text, language);
    const lexed = lexer(source); ranges = lexed.ranges; beforeFingerprint = lexicalFingerprint(source, lexed); fingerprint = candidate => lexicalFingerprint(candidate, lexer(candidate));
  }
  ranges = deduplicate(ranges);
  const { output, retained } = applyRanges(source, ranges, language);
  if (fingerprint(output) !== beforeFingerprint) throw new Error('Comment removal changed the parser/lexer semantic fingerprint');
  if (language === 'rust' && output !== source) run('rustfmt', ['--edition', '2024', '--emit', 'stdout', '--config', 'skip_children=true'], output);
  return { output, before: ranges.length, after: retained.length, removed: ranges.length - retained.length, retained, verification: ['typescript', 'javascript'].includes(language) ? 'TypeScript syntax-tree token equivalence' : language === 'svelte' ? 'Svelte parser + production JS/CSS compilation equivalence' : language === 'html' ? 'HTML/Svelte parser tree + CSS equivalence' : language === 'python' ? 'Python tokenize + ast.dump equivalence' : language === 'powershell' ? 'PowerShell parser + noncomment token equivalence' : language === 'rust' ? 'Rust literal-aware token equivalence + rustfmt parsing' : 'Literal-aware noncomment token equivalence' };
}

function ownInventory() {
  const inventory = run('git', ['ls-files', '-z', '--cached', '--others', '--exclude-standard']).split('\0').filter(Boolean);
  return [...new Set(inventory)].filter(name => !name.split(/[\\/]/).some(part => excludedSegments.has(part) || part.startsWith('.codex-'))).sort();
}

function selfTest() {
  let count = 0;
  const check = (language, filename, source, expectedRemoved, required) => {
    const result = analyze(source, filename, language);
    assert.equal(result.removed, expectedRemoved, filename);
    for (const literal of required) assert.ok(result.output.includes(literal), `${filename} altered literal ${literal}`);
    count++;
    return result;
  };
  check('typescript', 'fixture.ts', 'const url="https://host/path"; const regex=/https?:\\/\\/[^/]+/; // remove\nconst t=`raw /* keep */ ${1 /* remove */}`;', 2, ['https://host/path', '/https?:\\/\\/[^/]+/', 'raw /* keep */']);
  const directive = check('typescript', 'directive.ts', '// @ts-expect-error test directive\nconst n: number="x"; /* remove */', 1, ['// @ts-expect-error']);
  assert.equal(directive.retained[0].reason, 'tool-directive');
  check('svelte', 'Fixture.svelte', '<script lang="ts">let a=1; // remove\n</script><style>/* remove */div{color:red}</style><div><!-- remove -->{a /* remove */}<span>{"https://host/path"}</span></div>', 4, ['https://host/path']);
  check('svelte', 'Directive.svelte', '<script>let a=1;</script><!-- svelte-ignore a11y_no_static_element_interactions --><div onclick={()=>a++}>{a}</div>', 0, ['svelte-ignore']);
  check('rust', 'fixture.rs', 'fn main(){let x=r###"https://host/*keep*/"###;let c=\'/\';/* nested /* inner */ outer */ let a=1; // remove\nlet _: &\'static str="//keep";}', 2, ['r###"https://host/*keep*/"###', "&'static", '"//keep"']);
  check('java', 'Fixture.java', 'class Fixture { String x="https://host/*keep*/"; char slash=\'/\'; /* remove */ String block="""\n// keep inside text block\n"""; // remove\n}', 2, ['https://host/*keep*/', '// keep inside text block']);
  check('css', 'fixture.css', '/* remove */a{background:url("https://host/a/*keep*/");content:"/*keep*/";color:red/* remove */}', 2, ['https://host/a/*keep*/', '"/*keep*/"']);
  check('python', 'fixture.py', '#!/usr/bin/env python\n# coding: utf-8\n"""Module metadata stays a literal."""\nx="https://host/#keep" # remove\n', 1, ['#!/usr/bin/env python', '# coding: utf-8', 'Module metadata stays a literal.', 'https://host/#keep']);
  check('python', 'bom-fixture.py', '\ufeff# coding: utf-8\nx="# keep" # remove\n', 1, ['\ufeff# coding: utf-8', '"# keep"']);
  check('sql', 'fixture.sql', "-- remove\nselect 'https://host/--keep', $$ /* literal stays */ $$; /* outer /* inner */ outer */", 2, ['https://host/--keep', '/* literal stays */']);
  check('nsis', 'fixture.nsh', '; remove\n!define WEB "https://host/#keep"\n# remove\n', 2, ['https://host/#keep']);
  check('nsis', 'template.nsi', '{{#if signed_plugins_path}}\n!addplugindir "{{signed_plugins_path}}" ; remove\n{{/if}}\n', 1, ['{{#if signed_plugins_path}}', '{{/if}}', '!addplugindir']);
  check('powershell', 'fixture.ps1', '#Requires -Version 5.1\n$x="https://host/#keep" # remove\n$y=@\'\n# literal remains\n\'@\n', 1, ['#Requires -Version 5.1', 'https://host/#keep', '# literal remains']);
  const legal = check('java', 'Legal.java', '/* SPDX-License-Identifier: Apache-2.0 */\nclass Legal {} // remove', 1, ['SPDX-License-Identifier']);
  assert.equal(legal.retained[0].reason, 'legal-notice');
  console.log(JSON.stringify({ self_test: 'passed', fixtures: count }));
}

async function main() {
  const args = process.argv.slice(2);
  if (args.includes('--self-test')) { selfTest(); return; }
  const write = args.includes('--write');
  if (write && args.includes('--dry-run')) throw new Error('Choose either --write or --dry-run');
  const value = flag => { const index = args.indexOf(flag); if (index < 0) return null; if (!args[index + 1] || args[index + 1].startsWith('--')) throw new Error(`Missing value for ${flag}`); return args[index + 1]; };
  const reportPath = value('--report'), only = value('--only');
  const report = { mode: write ? 'write' : 'dry-run', root, files_scanned: 0, files_changed: 0, comments_before: 0, comments_after: 0, comments_removed: 0, preserved_by_reason: {}, deferred: [], third_party_preserved: [], rejected: [], files: [] };
  const changes = [];
  for (const filename of ownInventory()) {
    if (thirdPartyFiles.has(filename)) { report.third_party_preserved.push(filename); continue; }
    const language = languageByExtension.get(extname(filename));
    if (!language || only && filename !== only && !filename.startsWith(only.replace(/\/$/, '') + '/')) continue;
    if (filename.startsWith('src-tauri/installer/') && !args.includes('--include-installer')) { report.deferred.push(filename); continue; }
    const absolute = resolve(root, filename), canonical = realpathSync(absolute);
    if (canonical !== root && !canonical.startsWith(root + sep)) { report.rejected.push({ path: filename, reason: 'File resolves outside repository' }); continue; }
    const original = readFileSync(absolute), source = original.toString('utf8');
    if (!Buffer.from(source).equals(original)) { report.rejected.push({ path: filename, reason: 'Non-UTF-8 source requires explicit encoding review' }); continue; }
    report.files_scanned++;
    try {
      const result = analyze(source, filename, language);
      const entry = { path: filename, language, before: result.before, after: result.after, removed: result.removed, changed: result.output !== source, verification: result.verification, retained: result.retained, original_sha256: sha256(original), cleaned_sha256: sha256(result.output) };
      report.files.push(entry);
      report.comments_before += result.before; report.comments_after += result.after; report.comments_removed += result.removed;
      for (const { reason } of result.retained) report.preserved_by_reason[reason] = (report.preserved_by_reason[reason] || 0) + 1;
      if (entry.changed) { report.files_changed++; changes.push({ filename, absolute, original, output: result.output }); }
    } catch (error) { report.rejected.push({ path: filename, reason: error.message }); }
  }
  if (write) {
    if (report.rejected.length) throw new Error(`Refusing mass edit: ${report.rejected.length} files require review. Run --dry-run --report first.`);
    for (const change of changes) if (!readFileSync(change.absolute).equals(change.original)) throw new Error(`Concurrent edit detected in ${change.filename}; no changes applied.`);
    for (const change of changes) writeFileSync(change.absolute, change.output, 'utf8');
  }
  if (reportPath) {
    const absolute = resolve(root, reportPath);
    if (!absolute.startsWith(root + sep)) throw new Error('Report must stay inside repository');
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, JSON.stringify(report, null, 2) + '\n');
  }
  const { files, ...summary } = report;
  console.log(JSON.stringify({ ...summary, report: reportPath ? relative(root, resolve(root, reportPath)).replaceAll('\\', '/') : null }, null, 2));
  if (report.rejected.length) process.exitCode = 1;
}

if (resolve(process.argv[1] || '') === fileURLToPath(import.meta.url)) main().catch(error => { console.error(error.message); process.exitCode = 1; });

export { analyze, ownInventory, selfTest };
