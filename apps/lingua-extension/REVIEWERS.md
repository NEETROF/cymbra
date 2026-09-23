# Cymbra Lingua — building the submitted package from this source

This archive is the human-readable source of the reviewed add-on. It is submitted because
the package is produced by a build step twice over: the JavaScript is bundled with esbuild,
and `wasm/lingua_wasm_bg.wasm` is compiled from the Rust in `crates/lingua-core` and
`crates/lingua-wasm`.

## What is here

| Path                     | What it is                                                    |
| ------------------------ | ------------------------------------------------------------- |
| `apps/lingua-extension/` | the add-on: `src/` (TypeScript), `manifest.json`, `build.mjs` |
| `crates/lingua-core/`    | the analysis engine, in Rust                                  |
| `crates/lingua-wasm/`    | the `wasm-bindgen` wrapper the add-on loads                   |
| `crates/lingua-pack/`    | the builder that writes the language pack, in Rust            |
| `scripts/lingua-data/`   | the pipeline that produces the bundled language pack          |
| `backend/*/proto/`       | the `.proto` files the add-on's sync client is generated from |
| `Cargo.toml`             | the Rust workspace root, reduced to the three crates above    |
| `Cargo.lock`             | the dependency versions the submitted `.wasm` was built with  |

Inside the add-on, `apps/lingua-extension/vendor/foliate-js/` is third-party source, unmodified:
the EPUB renderer of the book reader ([foliate-js](https://github.com/johnfactotum/foliate-js),
MIT), vendored at the commit `vendor/VENDOR.md` records because it publishes no release. It is
bundled by the same esbuild step; its zip reader comes from the npm package `@zip.js/zip.js`
(`yarn.lock`), not from a prebuilt copy.

The add-on lives in a larger repository. This archive is cut from it by
`apps/lingua-extension/tool/make_source_archive.sh`, and every pull request rebuilds the
add-on from the archive alone, with the commands below, so that they keep working.

## Build it

Requires Node 22, Yarn (via Corepack), a stable Rust toolchain with the
`wasm32-unknown-unknown` target, `wasm-pack`, `protoc`, and Python 3.12.

```sh
cd apps/lingua-extension
corepack enable
yarn install --immutable
yarn gen:wasm     # compiles crates/lingua-wasm → src/wasm/pkg
yarn gen:proto    # generates the gRPC-web client stubs from the .proto files
pip install wordfreq
yarn gen:pack:real  # builds assets/pack.lingua from public corpora — see the next section
LINGUA_GRPC_WEB_URL=https://api.cymbra.app yarn build:firefox
```

The result is `dist-firefox/`, which is what was submitted. The archive's copy of this file
ends with the sign-in client ids the package was built with; without them the build is the
same add-on with its sign-in buttons hidden.

## The one file that is not in this archive

`assets/pack.lingua` — the English→French language pack, about 1.2 MB of frequency and
translation data. It is **generated, not authored**, and it is not kept in version control.
`yarn gen:pack:real` runs `scripts/lingua-data/build.sh en-fr assets/pack.lingua`, which
downloads its inputs from public corpora — Kaikki's French Wiktionary extract of
English entries, the `wordfreq` distribution, AGID's inflection list, and the CEFR-J and
Octanove vocabulary profiles — then reduces them with `scripts/lingua-data/reduce-en-fr.py`,
which is in this archive. The download is around 200 MB, so it takes a while on a cold cache.

`yarn gen:pack` builds a small test pack instead, which is enough to load the add-on and
exercise it without the download.

## Where the add-on reaches the network

The analysis is local: the pack and the WebAssembly engine run in the browser, and no page
content ever leaves the machine. A reader who never signs in makes no request at all. The book
reader (`reader.html`) opens the reader's own files from the device and requests nothing.

There are three origins, all of them in the source:

- `https://api.cymbra.app` — the reader's own account, and only once they have signed in: the
  word statuses and review history they asked to synchronise across their devices. Set at
  build time by `LINGUA_GRPC_WEB_URL` (`src/net/transport.ts`), and it lands in the manifest's
  `host_permissions`.
- `https://accounts.google.com` and `https://appleid.apple.com` — the sign-in pages, opened in
  the browser by the reader's own click (`src/state/oidc.ts`). The add-on receives the
  provider's token through the redirect and exchanges it with the backend; it never sees a
  password.
- `https://cymbra.app` — a link in the account screen to the account-deletion page
  (`src/account/flow.ts`). A link, not a request.
