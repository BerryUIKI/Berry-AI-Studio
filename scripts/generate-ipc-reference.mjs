import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';

// Deliberately limited to the current plain Tauri function declarations.
// Fail on any registered command that cannot be parsed instead of omitting it.
const root = fileURLToPath(new URL('../', import.meta.url));
const source = readFileSync(resolve(root, 'src-tauri/src/commands.rs'), 'utf8');
const shell = readFileSync(resolve(root, 'src-tauri/src/lib.rs'), 'utf8');
const handler = shell.match(/generate_handler!\[([\s\S]*?)\]/)?.[1];
if (!handler) throw new Error('Cannot find the Tauri command registry');
const registered = [...handler.matchAll(/commands::(\w+)/g)].map(match => match[1]);
const signatures = new Map();
const pattern = /#\[tauri::command\]\s*(?:#\[[^\n]*\]\s*)*pub\s+(?:async\s+)?fn\s+(\w+)\s*\(([\s\S]*?)\)\s*(?:->\s*([^\{]+))?\{/g;

function parameters(text) {
  const result = [];
  let start = 0, depth = 0;
  for (let i = 0; i < text.length; i++) {
    if ('<(['.includes(text[i])) depth++;
    if ('>)]'.includes(text[i])) depth--;
    if (text[i] === ',' && depth === 0) { result.push(text.slice(start, i)); start = i + 1; }
  }
  result.push(text.slice(start));
  return result.map(value => value.trim()).filter(Boolean).filter(value => !/:\s*(?:State\s*<|AppHandle\b)/.test(value)).map(value => {
    const colon = value.indexOf(':');
    if (colon < 0) throw new Error(`Unrecognized argument: ${value}`);
    const name = value.slice(0, colon).trim().replace(/^mut\s+/, '').replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
    const type = value.slice(colon + 1).trim().replace(/\s+/g, ' ');
    return `${name}: ${type}`;
  });
}

for (const match of source.matchAll(pattern)) signatures.set(match[1], {
  args: parameters(match[2]), result: (match[3] || '()').trim().replace(/\s+/g, ' '),
});
const rows = registered.map(name => {
  const signature = signatures.get(name);
  if (!signature) throw new Error(`Registered command not parsed: ${name}`);
  return `| \`${name}\` | ${signature.args.map(arg => `\`${arg}\``).join('<br>') || '(none)'} | \`${signature.result}\` |`;
});
const document = `# Registered Tauri IPC reference

Generated from the current source tree by \`node scripts/generate-ipc-reference.mjs\`. Run with \`--check\` to detect drift. This inventory describes the implementation snapshot, including unmerged work; it is not a claim about a published release. See [API_CONTRACTS.md](API_CONTRACTS.md) for wire conventions, side effects, error behavior and proposals that are not implemented.

There are ${registered.length} registered commands. Arguments below use the default Tauri camelCase invoke keys, while nested Serde DTO fields retain their declared snake_case names. AppHandle and State are injected by Tauri and must not be sent by the caller. Rust types are retained to avoid inventing missing DTO definitions: \`Option<T>\` accepts null/omission, \`Vec<T>\` is an array, \`HashMap\` is a JSON object, \`Result<T, String>\` resolves T or rejects with a string, and \`()\` resolves null. See Rust/TypeScript DTO locations in API_CONTRACTS.md.

The generator validates command coverage, not Serde field compatibility, safety, or supported feature availability. In particular, registered remote-database commands do not imply a working remote backend. When command attributes or declaration syntax change, update the generator or use a proper Rust parser; never silently skip commands.

| Command | Caller arguments (Rust value types) | Rust return type |
| --- | --- | --- |
${rows.join('\n')}
`;
const target = resolve(root, 'docs/IPC_REFERENCE.md');
if (process.argv.includes('--check')) {
  if (readFileSync(target, 'utf8').replace(/\r\n/g, '\n') !== document) {
    throw new Error('IPC reference is stale. Run node scripts/generate-ipc-reference.mjs');
  }
} else writeFileSync(target, document);
console.log(`Verified ${registered.length} registered command signatures.`);
