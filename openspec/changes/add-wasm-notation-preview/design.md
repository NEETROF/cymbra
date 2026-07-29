## Context

The app renders notation from a **pure Rust geometry engine**, `cymbra-musicxml-core`
(`crates/musicxml-core/`): `parse(&[u8]) -> ScoreDocument` (`src/lib.rs:130`) and
`layout_systems(&ScoreDocument, available_width: f64) -> Vec<System>` (`src/lib.rs:88`)
produce measure widths, note positions, stems/beams/clefs. The Flutter side draws that
geometry with a `CustomPainter` (`apps/music/lib/painters/partition_painter.dart`) using
SMuFL/Bravura glyphs (`smufl.dart`). Rust emits **geometry, not pixels/SVG**.

The back-office console (`apps/back-office/`, Vue 3 + Vite 6 + Connect gRPC-web) already
fetches score bytes for the preview via `stores/catalog.ts → GetCatalogScoreBytes`, but
`ScorePreview.vue` only shows metadata and a `"loaded N bytes"` placeholder. Its header
comment already names the intended seam (design D5): compile `layout_systems` to wasm and
paint with a JS/SVG SMuFL painter, kept isolated so it drops in — or is swapped for a JS
fallback — without touching the rest of the console.

Constraints:
- `cymbra-musicxml-core` is FFI-free, no IO/threads/audio; deps (`quick-xml`,
  `zip` deflate-only, `unicode-normalization`, `anyhow`) are wasm-compatible. The
  **app FFI crate** `rust_lib_music` (cpal/midir/`flutter_rust_bridge`) is native-only
  and must never enter the wasm build.
- No wasm toolchain exists in the repo today (no `wasm-bindgen`/`wasm-pack`/`wasm32`).
- The SPA runs under a strict CSP (`script-src 'self'`, `object-src 'none'`, no
  `wasm-unsafe-eval`) set via a Vite build-time meta plugin + `public/_headers`.
- Front-end rules (vue-frontend-architecture): components never call APIs/heavy modules
  directly — only stores/composables do, behind the `api()` seam; every async resource is
  one `Async<T>` ts-pattern union.

## Goals / Non-Goals

**Goals:**
- Render read-only notation in the console **faithfully to the app**, by reusing the
  app's own `layout_systems` geometry via WebAssembly (no re-implemented layout).
- A minimal, stable wasm entry point: `bytes → laid-out geometry` (parse + layout at a
  width), serialized to a JS-consumable shape; no IO, no FFI, no editing.
- Lazy-load and isolate the renderer so the console's initial bundle and the rest of the
  UI are unaffected, and a pure-JS fallback stays possible.
- Fulfill `moderation-console`'s existing *"Read-only score preview"* requirement and
  close deferred task 4.8.

**Non-Goals:**
- Playback, cursor/highlighting, or auto-scroll (the app's partition *playback* state) —
  preview is static notation only.
- Editing the score in any way.
- Changing `cymbra-musicxml-core` behavior or the Flutter app / native gRPC surfaces.
- Pixel-identical parity with Flutter; the goal is faithful, judgeable notation, not a
  golden-image match across two painter implementations.
- Server-side rendering of the preview (client-rendered SPA only).

## Decisions

### D1 — Reuse `cymbra-musicxml-core` via a thin `wasm-bindgen` wrapper crate
Add a new workspace crate (e.g. `crates/musicxml-wasm/`) with `crate-type = ["cdylib"]`
depending **only** on `cymbra-musicxml-core`. It exposes one primary export that runs
`parse` then `layout_systems(doc, width)` and returns the geometry.
- *Why:* the core is already pure and is the single source of truth for layout; a wrapper
  keeps the wasm concern out of the core crate and out of `rust_lib_music`.
- *Alternatives:* (a) add `cdylib` + wasm exports directly to `musicxml-core` — rejected,
  pollutes the shared crate with a bindgen dep and a web-only `crate-type`; (b) target
  `wasm-bindgen` at `rust_lib_music` — rejected, it drags cpal/midir/frb (native-only).

### D2 — Serialize geometry as JSON across the wasm boundary (v1)
`wasm-bindgen` can't directly hand back the rich `ScoreDocument`/`Vec<System>`. v1
serializes them (serde → JSON string, or `serde-wasm-bindgen` → JS object) and the JS
painter consumes a typed mirror.
- *Why:* simplest correct boundary; the model is modest per score and produced once per
  preview (not per frame). Keeps the Rust surface tiny and stable.
- *Trade-off:* a serialization pass per preview. Acceptable — previews are user-initiated,
  one at a time. If it ever matters, a zero-copy/`extern "C"` struct boundary is a later
  optimization behind the same JS API.

### D3 — Draw with a JS/SVG SMuFL painter mirroring `partition_painter.dart`
The renderer paints **SVG** (declarative, CSP-friendly, no canvas 2D-context eval, crisp
at any zoom) using Bravura glyphs, mirroring the app painter's element set (staff lines,
clefs, note heads, stems, beams, accidentals, dots, rests, ledger lines).
- *Why SVG over Canvas:* SVG is inspectable/testable in Vitest (assert nodes exist),
  scales for the read-only preview, and needs no per-frame redraw. Canvas would be faster
  for large scores but the preview is static.
- *Font:* bundle Bravura (SMuFL) **same-origin** as a static asset under the strict CSP;
  no external font fetch.

### D4 — Lazy load + isolate behind a store/composable seam
The wasm module is `import()`-ed on demand (first preview), instantiated once, and driven
by a composable (e.g. `useScoreRenderer`) or the catalog store — **not** from
`ScorePreview.vue` directly. The component stays presentational (props in, SVG slot out);
the render lifecycle is an `Async<T>` union (`idle|loading|success|error`) so a wasm
load/instantiate/parse failure degrades to the existing informational placeholder, never a
page error.
- *Why:* honors the vue-frontend-architecture rules and the D5 isolation already scaffolded
  in `ScorePreview.vue`; keeps the pure-JS fallback swappable at one seam.

### D5 — Build the wasm artifact in CI, not at Cloudflare Pages
Produce the artifact with `wasm-pack build --target web` (or `wasm-bindgen` directly) as a
back-office build step wired into `package.json` (alongside `gen`) and run in **GitHub
Actions** — mirroring how proto codegen is already built in Actions because the Pages image
can't run the Rust/protoc toolchain. Commit-free: the artifact is a build output, loaded by
Vite as a static/asset import.
- *Why:* the CF Pages build image lacks the Rust/wasm toolchain, exactly like `yarn gen`.

### D6 — Relax CSP minimally to allow wasm instantiation
Add `wasm-unsafe-eval` to `script-src` in the Vite CSP meta plugin and `public/_headers`.
Keep everything else (`object-src 'none'`, same-origin scripts) unchanged.
- *Why:* browsers require `wasm-unsafe-eval` (or `unsafe-eval`) to instantiate wasm under a
  restrictive CSP; `wasm-unsafe-eval` is the narrow, wasm-only allowance.

## Risks / Trade-offs

- **Wasm bundle size / instantiation cost** → lazy-load on first preview; measure the
  gzipped artifact; the D4 isolation keeps a pure-JS fallback viable if the cost is too
  high (proposal/4.8 explicitly wanted this escape hatch).
- **Fidelity drift between the SVG painter and `partition_painter.dart`** (two painters,
  one geometry) → the *geometry* is shared (the app's own `layout_systems`), so drift is
  limited to glyph drawing; scope v1 to the common element set and treat exotic notation as
  best-effort. Non-goal: pixel parity.
- **A future native dep silently breaks the wasm build** → the `shared-musicxml-crate`
  spec gains a wasm-buildability requirement and CI compiles the wrapper for `wasm32`, so a
  regression fails the build.
- **CSP `wasm-unsafe-eval` broadens the policy** → it is the wasm-specific token, not
  `unsafe-eval`; scripts stay same-origin and `object-src 'none'`.
- **Serialization overhead (D2)** → one-shot per preview, acceptable; optimization path
  exists behind the same JS API.

## Migration Plan

1. Land the wrapper crate + wasm CI build (dormant; nothing imports it yet).
2. Add the SVG painter + composable/store seam with Vitest coverage, still behind the
   placeholder (feature-guarded).
3. Wire `ScorePreview.vue` to the composable; ship the CSP allowance and the wasm asset
   loading.
4. Rollback: the renderer is one isolated seam guarded by an `Async` union — reverting to
   the metadata/`"loaded N bytes"` placeholder is a single-seam change and needs no
   backend/data changes.

## Open Questions

- **Serialization format (D2):** raw serde-JSON string vs. `serde-wasm-bindgen` JS objects
  — decide by ergonomics/perf during implementation.
- **Painter scope for v1:** which notation elements are in-scope vs. best-effort (lyrics,
  tuplet brackets, multi-voice, directions)? Aim for the app painter's core set; confirm
  the minimum a moderator needs to judge a score.
- **wasm toolchain:** `wasm-pack` vs. bare `wasm-bindgen` + `cargo build --target wasm32` —
  pick the lighter CI setup that Vite loads cleanly.
- **Bravura licensing/bundling:** confirm the SMuFL font we bundle same-origin and its
  license fit (SIL OFL for Bravura) — coordinate with the existing font/licensing notes.
