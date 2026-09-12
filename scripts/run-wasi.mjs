// Standalone preview1 runner. The host exposes the chosen directory as /workspace.
import { readFileSync, realpathSync } from 'node:fs';
import { WASI } from 'node:wasi';

const [artifact, directory, ...args] = process.argv.slice(2);
if (!artifact || !directory || args.length === 0) {
  console.error('Usage: node scripts/run-wasi.mjs <wasm> <host-directory> <command> [args...]');
  process.exit(2);
}
const wasi = new WASI({
  version: 'preview1',
  args: ['adamantium-files', ...args],
  preopens: { '/workspace': realpathSync(directory) },
  returnOnExit: true,
});
const module = await WebAssembly.compile(readFileSync(artifact));
const instance = await WebAssembly.instantiate(module, {
  wasi_snapshot_preview1: wasi.wasiImport,
});
process.exitCode = wasi.start(instance);
