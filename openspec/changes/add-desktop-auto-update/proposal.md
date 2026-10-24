## Why

Windows and Linux ship outside any store, so nothing updates them, and nothing distributes them past a bare archive on GitHub Releases. The release
pipeline publishes a bare `Compress-Archive` zip of the Flutter Release folder
and a `tar.gz` of the Linux bundle
([music-release.yml](../../../.github/workflows/music-release.yml)); a user who installed
1.18 has no signal that 1.24 exists and no path to it except finding the GitHub
Releases page again by hand. iOS, Android and macOS all get store-managed
updates, so the desktop-without-a-store users are the only ones stuck on
whatever build they first downloaded — including when a backend contract moves
and their client silently breaks.

An updater is also, by construction, remote code execution on the user's
machine. Doing it means committing to an integrity chain that survives a
compromise of the hosting side, not just to a download button.

## What Changes

- **Windows ships through the Microsoft Store** (revised 2026-10-24: the owner will not pay for a code-signing certificate). The release builds an MSIX from the Flutter build and submits it to the Store, which re-signs it — no SmartScreen warning, no Smart App Control block, no certificate — and updates it. The site's download button uses the Store web installer, so a visitor never has to browse the Store. Company developer accounts are free since 2026-05. The app does not self-update on Windows. The portable zip is no longer offered to the public (unsigned, Smart App Control blocks it).
- **Linux artifact becomes a `.deb` package.** Installed under `/opt/cymbra-music`
  with a desktop entry, as a Linux user expects to install software. It ships a
  small privileged update helper so the app can update itself through the
  system's own password dialog. The `tar.gz` stays published (other distributions;
  no self-update). The Linux build moves to the oldest supported Ubuntu (22.04),
  so the binary also runs on Ubuntu 22.04 and Debian 12 — today's build on the
  latest Ubuntu does not, a defect the tarball already carries.
- **On Linux, the app checks for updates, downloads, verifies and installs.** A throttled
  check on launch (once per 24 h) plus an on-demand check from settings; a
  non-blocking prompt; download with progress; then a password-confirmed package
  upgrade + relaunch. The tarball, which cannot self-update, degrades to "a new version exists, here is the download
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
## Capabilities

### New Capabilities
- `platform-app-update-feed`: the backend service that stores CI-signed release
  manifests and serves them anonymously per product/channel/OS/arch, with
  staged rollout, pause, and minimum-supported-version policy. Platform, not
  Music: Cymbra Live (Tauri) will consume the same feed, and a product consumes
  the socle rather than redeclaring it.
- `music-desktop-auto-update`: the in-app updater for Cymbra Music on Linux — when it checks, what it verifies before executing anything, how it
  installs per platform, how it degrades, and how it behaves in the UI.
- `music-desktop-distribution`: the Windows and Linux release artifacts and how
  they are produced and published — the Windows MSIX submitted to the Microsoft
  Store, the Linux `.deb` and the signed manifest published alongside it.

### Modified Capabilities
<!-- None. `runtime-feature-flags`, `backend-service` and
     `music-macos-store-distribution` are consumed unchanged; macOS keeps its
     App Store path and is explicitly out of scope here. -->

## Impact

**Products.** Music: new (Store MSIX, `.deb`, Linux in-app updater, settings entry,
localized strings). Platform/backend: new (public `/updates/*` router, admin
ingest route, one table, one shared manifest crate). Live: nothing now — the
feed is shaped so it can consume it later. Back-office and site: untouched;
rollout is driven by the CI ingest payload, a back-office screen is a follow-up.

**Code.**
- `apps/music/lib/services/update/*` (service, installer seams, manifest
  verification), `apps/music/lib/state/*` (Riverpod notifier + a listener
  widget), settings entry, `l10n` strings (fr/en).
- `apps/music/pubspec.yaml` (`msix` packaging config), `apps/music/linux/deb/`
  (package recipe: control file, `.desktop`, icon, polkit policy).
- `backend/updates` (the module owns its HTTP surface; the server mounts it) + a
  new `crates/update-manifest` shared by the backend ingest, the CI signing
  binary and the Linux privileged update helper.
- [music-release.yml](../../../.github/workflows/music-release.yml): MSIX
  packaging and Store submission in the `windows` job, `.deb` packaging in the
  `linux` job, plus a manifest job that signs and publishes the Linux release.

**Ops.** `/updates/*` **must** be added to the Caddy allow-list
([Caddyfile:39](backend/deploy/Caddyfile:39)) — a path missing from the `@http`
matcher falls through to tonic and answers `200` with an empty `grpc-status: 12`
instead of the manifest, which would make the update check fail invisibly in
production. GitHub Actions secrets hold the Ed25519 signing key, the ingest
credential and the Partner Center API credentials. One migration adds the
releases table. The Store needs a (free) company developer account for the
publisher, an app reservation and a first manual submission in Partner Center
before CI can submit updates; each submission is certified in up to three
business days.

**Dependencies.** One Flutter package for Ed25519 verification; `ed25519-dalek`
on the Rust side. No new runtime service.

**Risk.** The updater executes downloaded code, so the whole design hangs on
verify-before-execute (signature, then size, then SHA-256, then no-downgrade).
On Linux the privileged helper is the piece to keep small and tested; on Windows the Store owns signing and updates.
