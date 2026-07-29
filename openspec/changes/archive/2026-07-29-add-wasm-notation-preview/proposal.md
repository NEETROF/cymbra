## Why

The back-office moderation console's score preview (`ScorePreview.vue`) is a shell:
it shows metadata and a `"loaded N bytes"` placeholder, not the actual notation. The
`moderation-console` spec already requires a read-only preview whose notation is
*"rendered faithfully to how the app renders it"* — so a moderator can judge a score
as users will see it — but that requirement is unmet. The app renders notation from a
pure Rust geometry engine (`cymbra-musicxml-core`) that is already FFI-free and
wasm-friendly; compiling it to WebAssembly lets the browser reuse the *same* layout,
guaranteeing the moderator sees what the app draws instead of a re-implemented
approximation.

## What Changes

- Add a thin **`wasm-bindgen` wrapper crate** around `cymbra-musicxml-core` that
  compiles to `wasm32-unknown-unknown`, exposing a minimal read-only entry point:
  score `bytes → laid-out geometry` (i.e. `parse` then `layout_systems` at a given
  width), serialized to a JS-consumable shape. The app FFI crate (`rust_lib_music`,
  with cpal/midir/frb native deps) is **not** touched and stays gated out of the wasm
  build.
- Add a **JS/SVG SMuFL painter** in the back office that consumes that geometry and
  draws the notation read-only, mirroring the app's `PartitionPainter` + `smufl.dart`
  (staves, clefs, note heads, stems, beams, accidentals, rests). No editing affordances.
- **Lazy-load** the wasm module (dynamic import, instantiate on demand) so the console's
  initial bundle is unaffected; keep it behind an isolated seam so a pure-JS fallback
  remains possible if the wasm cost proves too high.
- Wire the renderer into `ScorePreview.vue` **through the existing store/composable
  seam** (bytes already arrive via `stores/catalog.ts` → `GetCatalogScoreBytes`); the
  component stays presentational and the render lifecycle is an `Async<T>` union.
- Add the **build/tooling** to produce and load the wasm artifact: a build step (e.g.
  `wasm-pack`/`wasm-bindgen`) wired into the back-office `gen`/`build` scripts and CI,
  plus the Vite side (wasm loading + the CSP `wasm-unsafe-eval` allowance needed to
  instantiate a module).

## Capabilities

### New Capabilities
- `web-notation-render`: render MusicXML score notation in a browser, **read-only** and
  faithful to the app, by compiling the shared notation/layout core
  (`cymbra-musicxml-core`) to WebAssembly and painting the resulting geometry with a
  JS/SVG SMuFL renderer. Covers the wasm entry-point contract (bytes → geometry, no
  IO/FFI), lazy loading + isolation (JS-fallback-friendly), and read-only fidelity.

### Modified Capabilities
- `shared-musicxml-crate`: add a requirement that the crate **remains buildable for a
  WebAssembly target** (`wasm32-unknown-unknown`) — no dependency or code may be
  introduced that is incompatible with that target — so the web wrapper keeps
  compiling. Locks in the "pure, no-IO/FFI" guarantee as a checked wasm-build contract.

## Impact

- **New crate:** a `wasm-bindgen` wrapper (workspace member under `crates/`) with
  `crate-type = ["cdylib"]`, depending only on `cymbra-musicxml-core`. Net-new wasm
  toolchain in the repo (none exists today).
- **Back office (`apps/back-office/`):** new renderer module + SVG painter, `ScorePreview.vue`
  wired to it via a store/composable, Vite config (wasm loading) and CSP meta
  (`wasm-unsafe-eval`) updated, `package.json` scripts + CI build step for the wasm
  artifact, Vitest coverage for the renderer seam.
- **Fulfills** the existing `moderation-console` *"Read-only score preview"* requirement
  end-to-end (no requirement change there — the consumer is completed) and closes the
  deferred task **4.8** from `add-moderation-back-office`.
- **Unchanged:** the Flutter app, `rust_lib_music`, the native gRPC/gRPC-web surface,
  and `cymbra-musicxml-core`'s existing behavior (the crate is reused as-is, only its
  wasm-buildability is asserted).
- **Risk to weigh in design:** wasm bundle size / instantiation cost vs. a pure-JS
  renderer; the SMuFL font (Bravura) must be bundled same-origin under the strict CSP.
