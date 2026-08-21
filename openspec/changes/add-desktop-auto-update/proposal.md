## Why

Windows and Linux ship outside any store, so nothing updates them. The release
pipeline publishes a bare `Compress-Archive` zip of the Flutter Release folder
and a `tar.gz` of the Linux bundle
([release-build.yml](.github/workflows/release-build.yml)); a user who installed
1.18 has no signal that 1.24 exists and no path to it except finding the GitHub
Releases page again by hand. iOS, Android and macOS all get store-managed
updates, so the desktop-without-a-store users are the only ones stuck on
whatever build they first downloaded — including when a backend contract moves
and their client silently breaks.

An updater is also, by construction, remote code execution on the user's
machine. Doing it means committing to an integrity chain that survives a
compromise of the hosting side, not just to a download button.

## What Changes

- **Windows artifact becomes a per-user installer.** An Inno Setup `setup.exe`
  installing into `%LocalAppData%\Programs\Cymbra` replaces the zip as the
  primary download. Per-user is the whole point: an update can then run silently
  with **no UAC prompt**. The portable zip stays published for users who want it.
- **Linux artifact gains an AppImage.** A single self-contained executable that
  can replace itself in place. The `tar.gz` stays published.
- **The app checks for updates, downloads, verifies and installs.** A throttled
  check on launch (once per 24 h) plus an on-demand check from settings; a
  non-blocking prompt; download with progress; then the platform install path —
  silent installer + relaunch on Windows, atomic AppImage swap + relaunch on
  Linux. Install layouts that cannot self-update (portable zip, tarball, a
  read-only directory) degrade to "a new version exists, here is the download
  page" rather than failing.
- **A signed update manifest, served by the backend.** A new public HTTP
  endpoint returns the manifest for a product/channel/OS/arch. The manifest is
  signed **in CI** with an Ed25519 key the backend never holds, and the public
  key is compiled into the app. The backend can *withhold* or *stage* a release;
  it cannot forge one. Binaries stay on GitHub Releases (free, CDN-backed).
- **Staged rollout and a kill-switch**, as the percentage carried next to the
  signed payload and evaluated client-side against a locally generated random
  bucket — so the check sends **no identifier at all** and stays fully
  cacheable. `rollout_percent = 0` is the kill-switch.
- **`min_supported_version` forces an update** when a client is too old to talk
  to the backend, via a blocking localized screen — not a raw error.
- **No Windows code-signing certificate in this change.** SmartScreen warnings
  are accepted for now; MSIX is out because it cannot be sideloaded without a
  trusted certificate. The manifest carries a `key_id` so signing material can
  be added or rotated later without redesigning anything.

## Capabilities

### New Capabilities
- `platform-app-update-feed`: the backend service that stores CI-signed release
  manifests and serves them anonymously per product/channel/OS/arch, with
  staged rollout, pause, and minimum-supported-version policy. Platform, not
  Music: Cymbra Live (Tauri) will consume the same feed, and a product consumes
  the socle rather than redeclaring it.
- `music-desktop-auto-update`: the in-app updater for Cymbra Music on Windows
  and Linux — when it checks, what it verifies before executing anything, how it
  installs per platform, how it degrades, and how it behaves in the UI.
- `music-desktop-distribution`: the Windows and Linux release artifacts and how
  they are produced and signed — per-user installer, AppImage, and the signed
  manifest published alongside them.

### Modified Capabilities
<!-- None. `runtime-feature-flags`, `backend-service` and
     `music-macos-store-distribution` are consumed unchanged; macOS keeps its
     App Store path and is explicitly out of scope here. -->

## Impact

**Products.** Music: new (installer, AppImage, in-app updater, settings entry,
localized strings). Platform/backend: new (public `/updates/*` router, admin
ingest route, one table, one shared manifest crate). Live: nothing now — the
feed is shaped so it can consume it later. Back-office and site: untouched;
rollout is driven by the CI ingest payload, a back-office screen is a follow-up.

**Code.**
- `apps/music/lib/services/update/*` (service, installer seams, manifest
  verification), `apps/music/lib/state/*` (Riverpod notifier + a listener
  widget), settings entry, `l10n` strings (fr/en).
- `apps/music/windows/installer/` (Inno Setup script), `apps/music/linux/appimage/`
  (AppDir recipe: `AppRun`, `.desktop`, icon).
- `backend/server/src/updates.rs` + a new `crates/` manifest crate shared by the
  backend ingest and the CI signing binary.
- [release-build.yml](.github/workflows/release-build.yml): packaging steps in
  the `windows` and `linux` jobs, plus a manifest job that signs and publishes.

**Ops.** `/updates/*` **must** be added to the Caddy allow-list
([Caddyfile:39](backend/deploy/Caddyfile:39)) — a path missing from the `@http`
matcher falls through to tonic and answers `200` with an empty `grpc-status: 12`
instead of the manifest, which would make the update check fail invisibly in
production. One new GitHub Actions secret holds the Ed25519 signing key. One
migration adds the releases table.

**Dependencies.** One Flutter package for Ed25519 verification; `ed25519-dalek`
on the Rust side. No new runtime service.

**Risk.** The updater executes downloaded code, so the whole design hangs on
verify-before-execute (signature, then size, then SHA-256, then no-downgrade).
The Inno `AppId` GUID becomes permanent the day the first installer ships:
changing it later turns every update into a second parallel installation.
