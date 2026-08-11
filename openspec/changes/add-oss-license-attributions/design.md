## Context

Cymbra Music ships two license-bearing dependency graphs into the same binary:

- **Dart/Flutter pub packages** (`apps/music/pubspec.yaml`). Flutter already
  auto-registers every package's `LICENSE` file into `LicenseRegistry` at
  build time via the generated `NOTICES.Z`/license binding — `showLicensePage`
  (or `LicensePage`) reads that registry for free. No app code currently opens
  it anywhere.
- **Rust crates statically linked into `apps/music/rust`** (the FFI crate
  loaded via `flutter_rust_bridge`): `midir` (patched fork), `rustysynth`,
  `cpal`, `anyhow`, `jni`/`ndk-context` on Android, plus the shared
  `cymbra-musicxml-core` crate and their transitive deps. `Cargo.toml` sets
  `license = "Apache-2.0"` at the workspace level for Cymbra's own crates, but
  third-party crates carry their own licenses (mostly MIT/Apache-2.0, a couple
  of BSD/Zlib) and are not surfaced anywhere. `backend/*` and `crates/*` that
  only serve the backend/back-office are irrelevant here — only crates that
  actually end up in the app binary matter.

`legal-links` (`openspec/specs/legal-links/spec.md`) already gives us the
pattern to extend: a locale-resolved entry reachable from the account/legal
area, opened through an injectable launcher seam for testability. This change
adds a sibling entry that opens an in-app page instead of an external URL, so
the launcher seam doesn't apply here — it's plain in-app navigation.

## Goals / Non-Goals

**Goals:**
- One in-app entry point, next to Terms/Privacy, that discloses every
  third-party OSS license actually shipped in the binary (Dart + Rust).
- Zero manual upkeep for the Dart side (already automatic via pub).
- Rust notices generated at build time from `Cargo.lock`, so they can't drift
  from what's actually vendored.
- Exclude Cymbra's own crates/packages and anything not shipped in the app
  binary (backend-only or dev-only deps).

**Non-Goals:**
- No attribution requirement coverage for backend or back-office dependencies
  (separate binaries, separate distribution — out of scope for this change).
- No legal review of individual license texts; this change is a mechanical
  disclosure surface, not a license-compatibility audit.
- No change to `legal-links`' external CGU/Privacy behavior.

## Decisions

**1. Reuse `LicenseRegistry` as the single merge point for Dart + Rust.**
Flutter's `LicensePage` already reads `LicenseRegistry.licenses`. Instead of
building a bespoke screen, register the generated Rust notices into that same
registry (`LicenseRegistry.addLicense(...)` inside `main()`, before
`runApp`), and route the new legal-menu entry to the stock
`showLicensePage(context: ..., applicationName: 'Cymbra')`. One list, minimal
new UI code, no custom screen to test independently against Flutter's own
widget.
- *Alternative considered*: hand-rolled screen listing Dart + Rust separately.
  Rejected — duplicates what `LicensePage` already does well (search, per-
  package expansion) and doubles the surface to maintain/test.

**2. Generate Rust notices with `cargo about` at build/CI time, not committed
by hand.**
`cargo about generate` walks `Cargo.lock` and each crate's declared license,
producing a structured (JSON/Markdown) report. Run it scoped to
`apps/music/rust`'s dependency tree only (`cargo about generate --manifest-
path apps/music/rust/Cargo.toml`), so backend-only crates never leak in.
Output is written as a generated asset (e.g.
`apps/music/assets/generated/rust_licenses.json`, gitignored like other
generated artifacts in this repo) and parsed into `LicenseEntry` objects at
startup.
- *Alternative considered*: `cargo license` (simpler, less structured output,
  no license-text bundling — would need a second step to fetch full texts).
  `cargo about` produces ready-to-embed license text in one pass, matching
  what `LicensePage` needs (name + paragraphs).
- *Alternative considered*: commit a hand-maintained notices file. Rejected —
  guaranteed to drift the first time a dependency is bumped; this repo's
  convention is generated-and-gitignored for anything derivable from lockfiles
  (see codegen note in `CLAUDE.md`).

**3. Wire generation into the existing `melos run generate` / CI codegen step,
not a new standalone script.**
`melos.yaml`'s `generate` target already runs `build_runner` before analyze/
test; add the `cargo about` invocation alongside it (or as a pre-step CI
already runs before the Flutter build) so the asset is never stale relative
to `Cargo.lock`, consistent with how `flutter_rust_bridge_codegen` is treated
today (regenerate-before-build, not committed).

**4. Filtering "first-party / standard" out.**
- Dart: `LicenseRegistry` only contains pub packages with a bundled license
  file; Cymbra's own packages (there are none published to pub, only local
  path packages) don't register unless they ship a `LICENSE` — no action
  needed, this falls out naturally.
- Rust: `cargo about`'s config (`about.toml`) explicitly excludes local
  workspace members (`cymbra-musicxml-core`, the FFI crate itself) via
  `workspace = false`/`ignore-build-dependencies` style filters, keeping only
  external crates.

## Risks / Trade-offs

- [`cargo about` output format changes across versions] → pin the version in
  CI/melos, same convention as other codegen tools in this repo.
- [Generated asset missing/stale on a dev machine that skipped `melos run
  generate`] → mirrors the existing `build_runner`/`gen-grpc` staleness class
  already called out in `CLAUDE.md`; the entry point degrades gracefully
  (falls back to Dart-only license list if the Rust asset is absent) rather
  than crashing.
- [`cargo about` requires network access to fetch some crates' full license
  text on first run] → acceptable for a CI/build-time step (same class of
  network dependency as `cargo fetch`); does not affect the shipped app.
- [Scope creep to backend/back-office] → explicitly out of scope (Non-Goals);
  flagged for a possible future follow-up change if ever needed.

## Open Questions

- Exact placement of the new entry: account menu (next to Terms/Privacy) vs.
  a dedicated "About" section — default to co-locating with `legal-links`'
  existing entries unless the account-menu screen is already crowded; resolve
  during implementation by looking at the current menu layout.
