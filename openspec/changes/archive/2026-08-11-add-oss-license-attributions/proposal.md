## Why

Cymbra Music ships MIT/Apache-2.0/BSD-licensed Dart and Rust dependencies. Those
licenses contractually require reproducing the license text and copyright
notice in the distributed product — this is a compliance obligation, not a
cosmetic nice-to-have, and the app currently has no such surface anywhere near
its legal section (see `legal-links`, which only covers external CGU/Privacy
URLs).

## What Changes

- Add a "Licences open source" / "Open Source Licenses" entry in the app's
  legal/account area, next to the existing Terms of Service / Privacy Policy
  entries.
- Tapping it opens Flutter's built-in `LicensePage` (via `showLicensePage`),
  which auto-lists every Dart/Flutter pub dependency's bundled `LICENSE` —
  populated automatically by `flutter pub get` via `LicenseRegistry`, no
  manual data entry.
- Generate a Rust third-party notices file at build/CI time (via `cargo about`
  or `cargo license` over the workspace, scoped to the crates statically
  linked into the shipped `rust_lib_music` binary) and register its entries
  into the same `LicenseRegistry` via `LicenseRegistry.addLicense`, so Rust
  and Dart notices surface in one unified list.
- Exclude first-party Cymbra crates/packages and the Flutter/Dart SDK itself
  (already covered by Flutter's own about-page conventions) — only true
  third-party dependencies are listed.

## Capabilities

### New Capabilities
- `music-oss-license-attributions`: in-app third-party open-source license
  disclosure (Dart pub packages + statically-linked Rust crates), reachable
  from the same legal/account area as the CGU and Privacy Policy links.

### Modified Capabilities
(none — `legal-links` is unchanged; this is an adjacent, separate entry point)

## Impact

- **Cymbra Music (Flutter app, `apps/music`)**: new settings/account-menu
  entry + route to `LicensePage`; a generated Rust notices asset consumed at
  startup to extend `LicenseRegistry`.
- **Rust workspace**: new build-time step (likely a `cargo xtask` / shell
  script invoked from CI and local `melos run generate`) producing a
  machine-readable notices file (e.g. JSON) checked into or built alongside
  `apps/music/assets/`.
- **CI**: one additional generation step before the Flutter build so the
  notices asset is never stale relative to `Cargo.lock`.
- No backend, back-office, or Cymbra ID/Live impact.
