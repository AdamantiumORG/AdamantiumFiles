import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';

const artifact = resolve(process.argv[2] ?? 'target/wasm32-wasip1/release/adamantium-files.wasm');
const bytes = readFileSync(artifact);
assert.deepEqual([...bytes.subarray(0, 8)], [0, 97, 115, 109, 1, 0, 0, 0]);
assert.ok(WebAssembly.validate(bytes), 'artifact must be a valid core WASM module');
const module = await WebAssembly.compile(bytes);
assert.ok(WebAssembly.Module.exports(module).some(x => x.name === '_start'));
assert.ok(WebAssembly.Module.imports(module).some(x => x.module === 'wasi_snapshot_preview1'));
assert.ok(WebAssembly.Module.imports(module).every(x => x.module === 'wasi_snapshot_preview1'));
mkdirSync('target', { recursive: true });
const root = mkdtempSync(resolve('target/wasi-test-'));
function run(args, code = 0) {
  const result = spawnSync(process.execPath, ['scripts/run-wasi.mjs', artifact, root, ...args], { encoding: 'utf8' });
  assert.ifError(result.error);
  assert.equal(result.status, code, `${args}: ${result.stderr}`);
  return result.stdout;
}
try {
  assert.match(run(['--help']), /Usage:/);
  assert.equal(run(['exists', '/workspace/missing']).trim(), 'false');
  run(['mkdir', '/workspace/a/nested']);
  run(['mkdir', '/workspace/a/nested']);
  run(['append', '/workspace/a/new', 'created']);
  assert.equal(run(['read', '/workspace/a/new']), 'created');
  run(['remove', '/workspace/a/new']);
  run(['write', '/workspace/a/hello ą.txt', 'Hello 世界']);
  run(['append', '/workspace/a/hello ą.txt', '\nnext']);
  assert.equal(run(['read', '/workspace/a/hello ą.txt']), 'Hello 世界\nnext');
  run(['copy', '/workspace/a/hello ą.txt', '/workspace/copy']);
  run(['rename', '/workspace/copy', '/workspace/moved']);
  assert.equal(run(['exists', '/workspace/copy']).trim(), 'false');
  assert.equal(readFileSync(join(root, 'moved'), 'utf8'), 'Hello 世界\nnext');
  if (process.platform === 'win32') {
    // Node's Windows WASI host does not implement fd_readdir.
    const result = spawnSync(process.execPath,
      ['scripts/run-wasi.mjs', artifact, root, 'list', '/workspace/a'], { encoding: 'utf8' });
    assert.ifError(result.error);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Function not implemented/);
  } else {
    assert.equal(run(['list', '/workspace/a']), 'hello ą.txt\nnested\n');
  }
  run(['rmdir', '/workspace/a'], 1);
  run(['remove', '/workspace/a'], 1);
  run(['write', '/workspace/moved', 'x', 'extra'], 2);
  assert.equal(readFileSync(join(root, 'moved'), 'utf8'), 'Hello 世界\nnext');
  run(['write', '/workspace/moved', '']);
  assert.equal(run(['read', '/workspace/moved']), '');
  run(['remove', '/workspace/moved']);
  run(['read', '/workspace/missing'], 1);
  run(['write', '/workspace/missing/file', 'data'], 1);
  run(['unknown'], 2);
  run(['read'], 2);
  run(['rmdir', '/workspace/a/nested']);
  run(['remove', '/workspace/a/hello ą.txt']);
  run(['rmdir', '/workspace/a']);
  writeFileSync(join(root, 'binary'), Buffer.from([0, 255, 128, 10]));
  run(['copy', '/workspace/binary', '/workspace/binary-copy']);
  assert.deepEqual(readFileSync(join(root, 'binary-copy')), Buffer.from([0, 255, 128, 10]));
  console.log('WASM validation and WASI filesystem tests passed.');
} finally {
  rmSync(root, { recursive: true });
}
