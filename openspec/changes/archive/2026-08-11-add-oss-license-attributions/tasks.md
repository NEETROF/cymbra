## 1. Rust notices generation

- [x] 1.1 Add `cargo about` config (`about.toml`) scoped to `apps/music/rust`,
      excluding local workspace crates (`cymbra-musicxml-core`, the app FFI
      crate itself)
- [x] 1.2 Add a generation step (script or `cargo xtask`) that runs
      `cargo about generate` against `apps/music/rust/Cargo.toml` and writes a
      structured notices file (name, license, license text per crate) to
      `apps/music/assets/generated/rust_licenses.json`
- [x] 1.3 Wire the generation step into `melos run generate` (or the
      equivalent pre-build CI step) so it always runs before the Flutter build,
      matching how `build_runner`/`flutter_rust_bridge_codegen` are already
      regenerated
- [x] 1.4 Verify the generated file excludes backend-only crates and includes
      the expected third-party crates (`midir`, `rustysynth`, `cpal`, etc.)

## 2. Flutter license registry wiring

- [x] 2.1 Add a pure module (host-testable, no Flutter framework types beyond
      what's needed) that parses the generated Rust notices asset into
      `LicenseEntry` objects
- [x] 2.2 Register those entries via `LicenseRegistry.addLicense` during app
      startup (`main()`, before `runApp`), guarded so a missing/absent asset
      degrades gracefully (Dart-only list still shows)
- [x] 2.3 Unit test the parser (malformed/missing asset, well-formed asset →
      expected `LicenseEntry` list)

## 3. UI entry point

- [x] 3.1 Add an "Open Source Licenses" entry to the account/legal menu next
      to the existing Terms of Service / Privacy Policy entries (see
      `legal-links`)
- [x] 3.2 Wire the entry to `showLicensePage(context: ..., applicationName:
      'Cymbra')`
- [x] 3.3 Add translated strings for the new menu entry (French default,
      English/other fallback per `app-localization`)
- [x] 3.4 Widget test: tapping the entry opens the license page (no external
      browser launch)

## 4. Verification

- [x] 4.1 Widget test drives the real flow end-to-end: taps the account menu
      → licenses entry, opens the real `LicensePage`, and (in
      `license_notices_test.dart`) reads the actual bundled
      `assets/generated/rust_licenses.json` through `LicenseRegistry`,
      confirming a `midir` entry surfaces with real license text. Manual
      on-device visual check (does the page look right, do Dart pub packages
      also show) is still open — no simulator/desktop run was done this
      session.
- [x] 4.2 `melos run analyze` + `dart format` clean
- [x] 4.3 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets
      -- -D warnings`
- [x] 4.4 Flutter + Rust coverage stays ≥ 80% (new parser code covered by 2.3;
      `license_notices.dart` 100% covered, suite-wide 86.9% unit/widget,
      Rust workspace 86.6%)
- [x] 4.5 `openspec validate add-oss-license-attributions --strict` passes
