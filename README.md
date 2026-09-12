# AdamantiumFiles

A small file-management package foundation for Adamantium, version `0.1.0`.
It has no external Rust dependencies and provides a tested Rust library and
a native/WASI command for reading, writing, appending, copying, renaming,
deleting, checking existence, and creating/listing/removing directories.

## Adamantium integration status

**This package cannot currently be called from Adamantium code.** The supplied
compiler's `docs/CREATING_PACKAGES.md`, README, and installer implementation
support downloading WASM assets, but do not implement package execution,
bindings, or a package ABI. There is no working `.ad` import example to provide.
The Rust library API below is not a proposed stable Adamantium ABI.

Two compiler changes are needed for integration:

1. Package execution and bindings, including a filesystem host interface.
2. Installer support for `https://github.com/AdamantiumORG/AdamantiumFiles`.
   The current installer only accepts repositories under `AdmerPRO` and rejects
   this repository's actual organization before downloading anything.

The compiler was treated as read-only. No compiler changes are included here.

The package guide demonstrates `wasm32-unknown-unknown`, which does not provide
host filesystem access. This package deliberately uses **`wasm32-wasip1`**,
a core WASM module importing `wasi_snapshot_preview1` and exporting `_start`.
It has the eight-byte header the current installer checks, but requires a WASI
host to run. Passing that header check does not imply Adamantium compatibility.
See the [Rust target documentation](https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1.html)
and [Node WASI documentation](https://nodejs.org/api/wasi.html).

## Run locally

Install stable Rust, then run from this repository:

```sh
cargo run -- mkdir target/demo
cargo run -- write target/demo/hello.txt "Hello Adamantium"
cargo run -- append target/demo/hello.txt "!"
cargo run -- read target/demo/hello.txt
cargo run -- list target/demo
cargo run -- exists target/demo/hello.txt
cargo run -- copy target/demo/hello.txt target/demo/copy.txt
cargo run -- rename target/demo/copy.txt target/demo/moved.txt
cargo run -- remove target/demo/moved.txt
```

Use `cargo run -- --help` for all commands. Quote paths and text containing
spaces. Successful commands exit with `0`, filesystem failures with `1`, and
unknown commands or incorrect argument counts with `2`. Errors go to stderr.
`read` outputs exact bytes, `exists` outputs `true` or `false` plus a newline,
and `list` prints entry names one per line. Mutations produce no stdout.

## Rust library

| Function | Behavior |
| --- | --- |
| `read(path)` | Read the complete file as bytes |
| `read_text(path)` | Read UTF-8 text; reject invalid encoding |
| `write(path, contents)` | Create or truncate a file |
| `append(path, contents)` | Append bytes; create a missing file |
| `exists(path)` | Check a file or directory; preserve access errors |
| `copy(source, destination)` | Copy bytes and permissions, replacing destination; return byte count |
| `rename(source, destination)` | Rename using host OS behavior |
| `remove_file(path)` | Delete a file |
| `create_dir(path)` | Create a directory and missing parents |
| `remove_dir(path)` | Delete an empty directory |
| `list_dir(path)` | Return sorted immediate entry names as `PathBuf` values |

Functions return `std::io::Result`, preserving filesystem errors rather than
panicking. Files are closed when each operation returns. Reading loads the whole
file into memory. Writes are not atomic or guaranteed durable and may partially
complete on an I/O failure. Write and append do not create parent directories or
add newlines. Copy requires source and destination to be different files.
Rename replacement rules vary by OS and cross-filesystem renames can fail;
use an absent destination for portable behavior.

Paths and symlinks follow host semantics. This library imposes no workspace
restriction on native callers. The CLI's `list` output is for humans: non-UTF-8
names are displayed lossily and embedded newlines are not escaped. Use the Rust
API when exact native filenames are needed. Recursive deletion, open-handle
APIs, metadata queries, and Adamantium bindings are outside this initial scope.

## Build and run WASM

```sh
rustup target add wasm32-wasip1
cargo build --locked --release --target wasm32-wasip1 --bin adamantium-files
node scripts/run-wasi.mjs target/wasm32-wasip1/release/adamantium-files.wasm . write /workspace/hello.txt "Hello WASI"
node scripts/run-wasi.mjs target/wasm32-wasip1/release/adamantium-files.wasm . read /workspace/hello.txt
```

The runner needs Node.js 24 or later. Its second argument is the host directory
exposed as `/workspace`; use absolute guest paths under `/workspace`.
Node's WASI implementation is not a security sandbox. The runner is for this
trusted package's local execution and testing, not untrusted modules.

Node's Windows WASI host currently returns `Function not implemented` for
directory listing (`fd_readdir`). Use the native command for listing on Windows.
The WASI test checks this explicit error on Windows and requires successful
listing on Linux and macOS. Other operations are tested on all three platforms.

## Tests and GitHub Actions

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release --target wasm32-wasip1 --bin adamantium-files
node tests/wasi.mjs
```

Native tests cover UTF-8 and binary data, truncation, append creation, copying,
renaming, sorted directory listings, nonempty-directory protection, missing
files, missing parents, and CLI error behavior. WASI tests validate the actual
module and execute filesystem commands, checking results on the host filesystem.
Fixtures are isolated below `target` and removed after tests.

`.github/workflows/ci.yml` runs on pushes, pull requests, and manual dispatches
on Linux, Windows, and macOS. It runs the checks above and uploads the tested
Linux-built WASM as an artifact named `adamantium-files-wasi`, containing
`adamantium_packet.wasm`. Download and extract the artifact to obtain the WASM;
the ZIP itself is not an installable package. CI does not publish a release.

## Future distribution

The compiler guide requires a release tag `adamantium_packet_0_1_0` and an asset
named exactly `adamantium_packet.wasm`. Rename the tested WASM when preparing
a release. Do not upload a native executable or the GitHub artifact ZIP.

Once the installer accepts this organization, the consumer declaration will be:

```toml
[packages]
"https://github.com/AdamantiumORG/AdamantiumFiles" = "0.1.0"
```

This declaration is currently rejected. No release has been published by this
implementation, and installing a future release will still require runtime
support before Adamantium code can use its file operations.
