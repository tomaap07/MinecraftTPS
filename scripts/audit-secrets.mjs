
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

const textLimit = 12 * 1024 * 1024;
const binaryLimit = 256 * 1024 * 1024;
const privateNames = '(?:CURSEFORGE_API_KEY|SUPABASE_SERVICE_ROLE_KEY|SUPABASE_SECRET_KEY|TP_MASTER_SECRET|CRON_SECRET|TP_DB_BRIDGE_TOKEN|DATABASE_URL|DB_PASSWORD|CLIENT_SECRET|GITHUB_TOKEN|GH_TOKEN|VERCEL_TOKEN|API_KEY|ACCESS_TOKEN|REFRESH_TOKEN)';
const localSecretName = /^(?:CURSEFORGE_API_KEY|(?:SUPABASE_)?SERVICE_ROLE(?:_KEY)?|SUPABASE_SECRET_KEY|TP_MASTER_SECRET|CRON_SECRET|TP_DB_BRIDGE_TOKEN|DATABASE_URL|DB_PASSWORD|CLIENT_SECRET|GITHUB_TOKEN|GH_TOKEN|VERCEL_TOKEN|API_KEY|ACCESS_TOKEN|REFRESH_TOKEN)$/i;
const placeholder = value => !value || /^(?:undefined|null|true|false)$/i.test(value)
  || /(?:your[_ -]?(?:api[_ -]?)?(?:key|token|secret|password)|replace[_ -]?me|example|placeholder|changeme|dummy|fake|test[_ -]?(?:key|token|secret)|<[^>]+>|\.\.\.|\$\{)/i.test(value)
  || /^(?:\$\{|process\.env\.|std::env::|env::|env\(|os\.environ|import\.meta\.env\.)/.test(value);

export function scanText(text, file = '<memory>', scope = 'worktree') {
  const findings = [];
  const seen = new Set();
  const add = (type, index, severity = 'high', note = '') => {
    const line = text.slice(0, index).split('\n').length;
    const key = `${type}:${line}:${severity}:${note}`;
    if (!seen.has(key)) findings.push({ scope, file, line, type, severity, ...(note ? { note } : {}) });
    seen.add(key);
  };
  const matches = (regex, type, classify) => {
    for (const match of text.matchAll(regex)) {
      const value = match[0];
      if (placeholder(value)) continue;
      const result = classify?.(value);
      add(type, match.index, result?.severity ?? 'high', result?.note ?? 'Credential-shaped value; verify owner and revoke if real.');
    }
  };
  matches(/\b(?:gh[pousr]_[A-Za-z0-9]{30,255}|github_pat_[A-Za-z0-9_]{40,255})\b/g, 'github-token');
  matches(/\b(?:AKIA|ASIA)[A-Z0-9]{16}\b/g, 'aws-access-key-id');
  matches(/\b(?:sk_live_|rk_live_)[A-Za-z0-9]{20,200}\b/g, 'payment-secret');
  matches(/\bsb_secret_[A-Za-z0-9_-]{20,255}\b/g, 'supabase-secret');
  matches(/\b(?:sk-proj-|sk-ant-api03-)[A-Za-z0-9_-]{30,255}\b/g, 'model-provider-secret');
  matches(/\$2[aby]\$\d{2}\$[./A-Za-z0-9]{45,80}/g, 'bcrypt-or-curseforge-key', () => ({ note: 'May be a CurseForge API key or a password hash; the shape alone does not prove validity.' }));
  matches(/-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----/g, 'private-key');
  matches(/\b(?:postgres(?:ql)?|mysql|mongodb(?:\+srv)?):\/\/[^\s/:"'<>]+:[^\s@"'<>]+@[^\s"'<>]+/gi, 'database-url-with-password');
  matches(/["'](?:Authorization|x-api-key|api-key|x-cron-secret)["']\s*[:,]\s*["'](?:Bearer\s+|token\s+)?[^"'\r\n]{20,}["']/gi, 'literal-authentication-header');
  matches(/\b(?:vault\.)?create_secret\s*\(\s*['"][^'"\r\n]{20,}['"]/gi, 'literal-vault-secret');
  for (const match of text.matchAll(/\beyJ[A-Za-z0-9_-]{12,}\.[A-Za-z0-9_-]{12,}\.[A-Za-z0-9_-]{16,}\b/g)) {
    if (placeholder(match[0])) continue;
    let role = '';
    try { role = JSON.parse(Buffer.from(match[0].split('.')[1], 'base64url').toString('utf8')).role ?? ''; } catch {   }
    add(role === 'anon' ? 'supabase-anon-public-key' : 'jwt-token', match.index, role === 'anon' ? 'info' : 'high', role === 'anon' ? 'Public anon key; verify RLS separately.' : 'Signed-token-shaped value; validity was not tested.');
  }
  const assignments = new RegExp(`\\b(${privateNames})\\b["']?\\s*[:=]\\s*(?:r#?"([^"\\n]*)"#?|"([^"\\n]*)"|'([^'\\n]*)'|([^\\s#,;]+))`, 'gi');
  for (const match of text.matchAll(assignments)) {
    const value = match[2] ?? match[3] ?? match[4] ?? match[5] ?? '';
    if (value.length < 20 || placeholder(value) || /^[A-Z][A-Z0-9_]*$/.test(value)
      || (match[5] !== undefined && /[\[\]{}()"']/.test(value))
      || /^(?:&?str|String|std::|[A-Za-z_$][\w.$]*\(|[A-Za-z_$][\w$]*\.[A-Za-z_$][\w$]*)/.test(value)) continue;
    add('private-value-assignment', match.index, 'high', `Sensitive field ${match[1].toUpperCase()}; value omitted.`);
  }

  if (/\.(?:json)$/i.test(file) && /(?:secret|credential|local-env)/i.test(file)) {
    const sensitive = new RegExp(`^(?:${privateNames}|master|bridge|cron)$`, 'i');
    const walk = value => {
      if (!value || typeof value !== 'object') return;
      for (const [key, child] of Object.entries(value)) {
        if (sensitive.test(key) && typeof child === 'string' && child.length >= 20 && !placeholder(child)) {
          add('private-json-store', 0, 'high', `Sensitive field ${key.toUpperCase()}; value omitted.`);
        }
        if (child && typeof child === 'object') walk(child);
      }
    };
    try { walk(JSON.parse(text)); } catch {   }
  }
  for (const match of text.matchAll(/\b(?:VITE_MS_CLIENT_ID|TP_MS_CLIENT_ID|DEFAULT_CLIENT_ID)\b\s*[:=][^\n]{0,40}?([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/gi)) {
    add('oauth-public-client-id', match.index, 'info', 'Public application identifier, not a client secret.');
  }
  matches(/\bsb_publishable_[A-Za-z0-9_-]{20,255}\b/g, 'supabase-publishable-key', () => ({ severity: 'info', note: 'Public publishable key; verify RLS separately.' }));
  return findings;
}

function git(args, options = {}) {
  return execFileSync('git', args, { maxBuffer: 64 * 1024 * 1024, windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'], ...options });
}
function within(root, candidate) {
  const absolute = path.resolve(root, candidate);
  const relative = path.relative(root, absolute);
  if (relative === '..' || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) throw new Error('Requested file is outside the repository.');
  return absolute;
}
const binaryExtensions = /\.(?:png|jpe?g|gif|webp|ico|icns|bmp|woff2?|ttf|otf|mp[34]|ogg|wav|zip|jar|exe|dll|pdb|class|pdf|7z|gz)$/i;

export function parseLocalEnvSecrets(text) {
  const secrets = [];
  for (const line of text.replace(/^\uFEFF/, '').split(/\r?\n/)) {
    const match = /^\s*(?:export\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$/.exec(line);
    if (!match || !localSecretName.test(match[1])) continue;
    let value = match[2].trim();
    if (value.startsWith('"') || value.startsWith("'")) {
      const quote = value[0];
      let end = -1;
      for (let i = 1; i < value.length; i++) {
        if (value[i] === '\\' && i + 1 < value.length) { i++; continue; }
        if (value[i] === quote) { end = i; break; }
      }
      if (end < 0 || !/^\s*(?:#.*)?$/.test(value.slice(end + 1))) throw new Error('Invalid quoting in a local secret input.');
      value = value.slice(1, end);

      value = value.replace(/\\([\\"'])/g, '$1');
    } else value = value.replace(/\s+#.*$/, '').trim();
    if (value) secrets.push({ name: match[1].toUpperCase(), value });
  }
  return secrets;
}

export function auditRepository(options = {}) {
  const root = git(['rev-parse', '--show-toplevel'], options.cwd ? { cwd: options.cwd } : {}).toString('utf8').trim();
  const runGit = (args, extra = {}) => git(args, { cwd: root, ...extra });
  const report = { version: 1, scannedAt: new Date().toISOString(), mode: options.staged ? 'staged' : 'worktree', filesScanned: 0, historyBlobsScanned: 0, binariesScanned: 0, skippedBinary: 0, skippedLarge: 0, missingFiles: 0, findings: [], limitations: ['Heuristic scanner: no finding is not proof that all secrets are absent.', 'No credential was sent to a provider or tested for validity.', 'Compressed archives and compressed executable payloads are not unpacked.'] };
  const exactSecrets = [];
  report.exactSecretScan = { inputs: [], filesCompared: 0, comparisons: 0, matches: 0 };
  for (const file of options.localSecretFiles ?? []) {
    const absolute = within(root, file);
    const stat = fs.lstatSync(absolute);
    if (!stat.isFile() || stat.isSymbolicLink() || stat.size > 512 * 1024) throw new Error('Invalid local secret input.');
    const secrets = parseLocalEnvSecrets(fs.readFileSync(absolute, 'utf8'));
    if (!secrets.length) throw new Error('No supported local secrets in the requested input.');
    report.exactSecretScan.inputs.push({ file, variableNames: [...new Set(secrets.map(secret => secret.name))] });
    exactSecrets.push(...secrets);
  }
  const readLimit = exactSecrets.length ? binaryLimit : textLimit;
  const exactCompare = (bytes, file, scope) => {
    if (!exactSecrets.length) return;
    report.exactSecretScan.filesCompared++;
    const matched = new Set();
    for (const secret of exactSecrets) {
      report.exactSecretScan.comparisons++;
      const encodings = ['utf8', 'utf16le'].filter(encoding => bytes.includes(Buffer.from(secret.value, encoding)));
      if (encodings.length && !matched.has(secret.name)) {
        report.findings.push({ scope, file, type: 'exact-local-secret-match', severity: 'high', secretName: secret.name, found: true, encodings });
        matched.add(secret.name);
        report.exactSecretScan.matches++;
      }
    }
  };
  const inspect = (bytes, file, scope) => {
    exactCompare(bytes, file, scope);
    if (bytes.length > textLimit) { report.skippedLarge++; return; }
    if (binaryExtensions.test(file) || bytes.subarray(0, 8192).includes(0)) { report.skippedBinary++; return; }
    report.findings.push(...scanText(bytes.toString('utf8'), file, scope));
    report.filesScanned++;
  };
  let files = options.staged
    ? runGit(['diff', '--cached', '--name-only', '--diff-filter=ACMRT', '-z']).toString('utf8').split('\0').filter(Boolean)
    : runGit(['ls-files', '--cached', '--others', '--exclude-standard', '-z']).toString('utf8').split('\0').filter(Boolean);
  files = [...new Set([...files, ...(options.paths ?? [])])];
  for (const file of files) {
    try {
      let bytes;
      if (options.staged && !(options.paths ?? []).includes(file)) bytes = runGit(['show', `:${file}`], { maxBuffer: readLimit + 1 });
      else {
        const absolute = within(root, file);
        const stat = fs.lstatSync(absolute);
        if (!stat.isFile() || stat.isSymbolicLink()) continue;
        if (stat.size > readLimit) { report.skippedLarge++; continue; }
        bytes = fs.readFileSync(absolute);
      }
      inspect(bytes, file, options.staged ? 'staged' : 'worktree');
    } catch { report.missingFiles++; }
  }
  if (options.history) {
    const objects = runGit(['rev-list', '--objects', '--all']).toString('utf8').split('\n').map(line => {
      const separator = line.indexOf(' ');
      return separator > 0 ? { oid: line.slice(0, separator), file: line.slice(separator + 1) } : null;
    }).filter(Boolean);
    if (objects.length) {
      const metadata = runGit(['cat-file', '--batch-check=%(objectname) %(objecttype) %(objectsize)'], { input: objects.map(item => item.oid).join('\n') + '\n' }).toString('utf8').trim().split('\n');
      for (let i = 0; i < metadata.length; i++) {
        const [oid, type, size] = metadata[i].split(' ');
        if (type !== 'blob') continue;
        if (Number(size) > readLimit) { report.skippedLarge++; continue; }
        const before = report.filesScanned;
        inspect(runGit(['cat-file', 'blob', oid], { maxBuffer: readLimit + 1 }), objects[i].file, `history:${oid.slice(0, 12)}`);
        if (report.filesScanned > before) report.historyBlobsScanned++;
      }
    }
  }
  for (const file of options.binaries ?? []) {
    const absolute = within(root, file);
    if (fs.statSync(absolute).size > binaryLimit) { report.skippedLarge++; continue; }
    const bytes = fs.readFileSync(absolute);
    exactCompare(bytes, file, 'binary');
    const ascii = bytes.toString('latin1').match(/[\x20-\x7e]{6,}/g)?.join('\n') ?? '';
    const wide = bytes.toString('utf16le').match(/[\x20-\x7e]{6,}/g)?.join('\n') ?? '';
    report.findings.push(...scanText(ascii, file, 'binary-ascii'), ...scanText(wide, file, 'binary-utf16le'));
    report.binariesScanned++;
  }
  report.summary = { high: report.findings.filter(item => item.severity === 'high').length, info: report.findings.filter(item => item.severity === 'info').length };
  return report;
}

function main() {
  const args = process.argv.slice(2);
  const options = { staged: args.includes('--staged'), history: args.includes('--history'), paths: [], binaries: [], localSecretFiles: [] };
  let reportFile = '';
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--path') options.paths.push(args[++i]);
    if (args[i] === '--binary') options.binaries.push(args[++i]);
    if (args[i] === '--local-secret-file') options.localSecretFiles.push(args[++i]);
    if (args[i] === '--report') reportFile = args[++i];
  }
  const report = auditRepository(options);
  const rendered = JSON.stringify(report, null, 2);
  if (reportFile) fs.writeFileSync(reportFile, rendered + '\n');
  process.stdout.write(rendered + '\n');
  if (report.summary.high) process.exitCode = 1;
}
if (process.argv[1] && pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url) {
  try { main(); } catch { process.stderr.write('Secret audit failed. No credential values were printed.\n'); process.exitCode = 2; }
}
