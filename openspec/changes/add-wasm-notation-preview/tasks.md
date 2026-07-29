## 1. Wasm wrapper crate (Rust)

- [ ] 1.1 Add a new workspace crate `crates/musicxml-wasm/` (`crate-type = ["cdylib"]`) depending only on `cymbra-musicxml-core` + `wasm-bindgen` (and `serde`/`serde-wasm-bindgen` or `serde_json` for the boundary); register it in the workspace `Cargo.toml` members. Do NOT depend on `rust_lib_music`.
- [ ] 1.2 Expose one read-only entry point `render(bytes: &[u8], available_width: f64) -> Result<JsValue, JsError>` that runs `parse` then `layout_systems` and serializes `{ document, systems }` (design D2). No IO, no FFI, no mutation.
- [ ] 1.3 Ensure the serialized geometry carries what the painter needs (measures + `min_width`, notes with positions/stem/beams/dots/accidental, clefs, systems' measure ranges). Add `serde` derives to the core model behind a feature if not already serializable, keeping app behavior unchanged.
- [ ] 1.4 Host-side Rust unit test: a fixture score parses+lays out and the entry point returns non-empty systems; malformed bytes return a typed error (no panic).

## 2. Wasm build & toolchain

- [ ] 2.1 Add a `wasm-pack build --target web` (or `wasm-bindgen` + `cargo build --target wasm32-unknown-unknown`) build for `musicxml-wasm`; output an ES module + `.wasm` asset consumable by Vite.
- [ ] 2.2 Wire the wasm build into the back-office `package.json` (e.g. a `gen:wasm`/`build` step) so `yarn build` produces the artifact locally.
- [ ] 2.3 Add the wasm build to CI (GitHub Actions) before the Cloudflare Pages deploy — build in Actions since the Pages image lacks the Rust/wasm toolchain (design D5); publish the built artifact with the SPA.
- [ ] 2.4 Add a CI check that `musicxml-core` (via `musicxml-wasm`) compiles for `wasm32-unknown-unknown`, satisfying the `shared-musicxml-crate` wasm-buildability requirement.

## 3. Browser renderer (back office)

- [ ] 3.1 Add a typed TS mirror of the serialized geometry (`ScoreDocument`/`System`/`NoteEvent` shape) under `apps/back-office/src/` for the painter to consume.
- [ ] 3.2 Implement an SVG SMuFL painter (`renderNotation(geometry) -> SVG`) mirroring `partition_painter.dart` + `smufl.dart`: staves, clefs, note heads, stems, beams, accidentals, dots, rests, ledger lines (design D3). Read-only, no interaction handlers.
- [ ] 3.3 Bundle the Bravura SMuFL font same-origin as a static asset; reference it from the painter without any external fetch (respect the strict CSP).
- [ ] 3.4 Add a lazy loader/composable `useScoreRenderer` that dynamic-`import()`s and instantiates the wasm module once, exposing `render(bytes, width) -> Async<SVG-or-model>` as a ts-pattern `Async<T>` union (design D4). Keep it the single isolated seam (JS-fallback-swappable).

## 4. Wire into the console

- [ ] 4.1 Drive the renderer from the store/composable seam (not the leaf component): feed the existing `bytes: Uint8Array` from `stores/catalog.ts` (`GetCatalogScoreBytes`) into `useScoreRenderer`; keep `ScorePreview.vue` presentational.
- [ ] 4.2 Update `ScorePreview.vue` `.notation` block to render the SVG on `success`, and fall back to the existing informational placeholder on `idle`/`loading`/`error` (no page-level error; design D4).
- [ ] 4.3 Add the `wasm-unsafe-eval` allowance to `script-src` in the Vite CSP meta plugin and `public/_headers`, leaving `object-src 'none'` and same-origin scripts unchanged (design D6).

## 5. Tests & verification

- [ ] 5.1 Vitest: the SVG painter draws expected nodes (staff lines, a clef, note heads, a beam) for a fixture geometry; the read-only view exposes no edit affordance.
- [ ] 5.2 Vitest: `useScoreRenderer` is lazy (module not imported until first render), instantiates once, and a load/instantiate/render failure yields the `error` union → placeholder (graceful degradation).
- [ ] 5.3 Vitest/component: `ScoreDetailView`/`ScorePreview` shows notation on success and the placeholder otherwise; `yarn build` (with the wasm artifact) is green.
- [ ] 5.4 Rust: `cargo fmt`/`clippy` clean for `musicxml-wasm`; `cargo llvm-cov` unaffected (wrapper is thin; core logic tested in `musicxml-core`).
- [ ] 5.5 `openspec validate add-wasm-notation-preview --strict` passes.

## 6. Docs & rollout

- [ ] 6.1 Document the wasm build + local dev in the back-office README (how to build the artifact, the CI step, the CSP note) and the JS-fallback escape hatch.
- [ ] 6.2 Note in the back-office README that this fulfills `moderation-console`'s read-only preview requirement and closes `add-moderation-back-office` task 4.8.
