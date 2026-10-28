## Context

Cymbra Music ships on five platforms. iOS, Android and macOS are store-managed
and keep their existing pipelines
([music-macos-store-distribution](../../specs/music-macos-store-distribution/spec.md)).
Windows and Linux are published as raw archives on GitHub Releases — a
`Compress-Archive` of the Flutter Release folder and a `tar.gz` of the Linux
bundle — with no installer, no version signal and no update path.

Constraints that shape the design:

- **The repo is public and already uses GitHub Releases**, so binary hosting is
  free and CDN-backed. There is no reason to pay for or operate binary hosting.
- **The backend exists** (Axum HTTP surface next to tonic, `backend/server`) and
  is fronted by Caddy that routes **by path** against an explicit allow-list
  ([Caddyfile:39](../../../backend/deploy/Caddyfile:39)). A path missing from
  `@http` silently falls through to tonic.
- **The app already carries everything needed client-side**: `http`, `crypto`,
  `path_provider`, `shared_preferences`, `package_info_plus`, `cymbra_flags`,
  and a platform seam at
  [app_platform.dart:52](../../../apps/music/lib/services/app_platform.dart:52).
- **Windows ships through the Microsoft Store** (owner decision, 2026-10-24, after
  weighing signing). Unsigned executables are blocked outright by Windows 11's
  Smart App Control, and every code-signing route costs money (Artifact Signing
  ~10 $/month, OV certificates 150–300 $/year); the Store re-signs MSIX packages
  for free, updates them, and company accounts are free since 2026-05. The app
  therefore never self-updates on Windows (decision 4, revised).
- **Linux: one file for every distribution.** A `.deb` upgraded through `pkexec`
  and a root helper was implemented and tested on 2026-10-24, then dropped by the
  owner for the AppImage (decision 5): every distribution, no password, no
  privileged code.
- **CLAUDE.md gates**: ≥80 % line coverage both ecosystems, Riverpod+Freezed
  with dependencies as providers, no raw technical error strings in the UI.

## Goals / Non-Goals

**Goals:**

- A Windows and a Linux build that can update themselves, with no store and no
  administrator prompt.
- An integrity chain that holds even if the backend or the GitHub account is
  compromised: nothing downloaded is executed unless it matches a signature made
  by a key neither of those systems holds.
- Staged rollout and an instant kill-switch, so a bad release can be stopped
  without shipping anything.
- A forced-update path for clients too old to talk to the backend.
- Install layouts that cannot self-update degrade gracefully instead of failing.

**Non-Goals:**

- macOS, iOS, Android — store-managed, untouched.
- Delta/differential updates (zsync, MSIX block-map). Full downloads for now;
  the manifest shape leaves room to add them.
- A `.deb`, an APT repository, Flatpak, Snap on Linux. The `.deb` (tried) is
  Debian-family only and needs a password per update; the sandboxed formats
  block raw MIDI access out of the box (Snap's `alsa` interface is
  `deny-auto-connection`).
- A Windows code-signing certificate, a self-hosted Windows installer, and the
  portable zip as a public download (unsigned ⇒ blocked by Smart App Control).
- Distributions other than Debian/Ubuntu derivatives as packages. They keep the
  tarball, which notifies but does not self-update; an `.rpm` follows only on
  demand.
- Silent updates without consent. The user always approves an install.
- ARM64 desktop builds. The pipeline is x64-only today; the manifest is keyed by
  `os`+`arch` so adding one is additive.
- A back-office screen for rollout. Rollout is set by the CI ingest payload;
  a UI is a follow-up.

## Decisions

### 1. Sign in CI, serve from the backend, host binaries on GitHub

The manifest is produced **and signed during the release workflow** with an
Ed25519 key held only as a GitHub Actions secret. The backend stores the signed
envelope opaquely and serves it; the public key is compiled into the app.

This splits the two powers deliberately: the backend decides **whether and to
whom** a release is offered (rollout, pause, minimum version), while only CI can
say **what** a release is. A compromised backend can stall or withhold updates —
a denial of service, visible and recoverable — but cannot fabricate one. The
backend also **re-verifies the signature on ingest**, so a stolen ingest
credential cannot inject an unsigned or foreign-key manifest either.

*Alternatives considered.* Backend signs on ingest — rejected: it collapses both
powers into the internet-facing service. Static manifest on the OVH bucket or a
GitHub Release asset only — rejected: no rollout, no kill-switch, and
`/releases/latest/download/…` resolves against the latest release of *any*
release-please component in this monorepo, not necessarily a `music-v*` one.
A GitHub API query filtered on the `music-v` tag prefix — kept in reserve as the
zero-infrastructure fallback, but it costs a 60 req/h/IP unauthenticated rate
limit and still offers no staging.

### 2. Sign the exact bytes, not a re-serialization

The endpoint returns an envelope:

```json
{
  "manifest": "<base64 of the exact manifest JSON bytes>",
  "signature": "<base64 Ed25519 signature over those bytes>",
  "key_id": "2026-10-a",
  "rollout_percent": 25
}
```

The signature covers the **opaque byte string**, which is then parsed only after
verification. This sidesteps JSON canonicalization entirely — no key-ordering,
whitespace or number-formatting rule that Rust and Dart must agree on, and no
class of bug where the bytes verified differ from the bytes parsed.

`key_id` sits outside the signature as a selection hint only: the app holds a map
of trusted keys and rejects an unknown id, so tampering with it can cause a
verification failure but never a forgery. That is what makes key rotation
possible later without a redesign.

`rollout_percent` is also outside the signature, because it is backend policy
rather than release identity. Tampering with it can only make a *signed* update
offered sooner — harmless.

The manifest itself:

```json
{
  "schema": 1,
  "product": "music",
  "channel": "stable",
  "version": "1.25.0+34",
  "released_at": "2026-08-21T10:00:00Z",
  "min_supported_version": "1.20.0+27",
  "notes_url": "https://github.com/NEETROF/cymbra/releases/tag/music-v1.25.0",
  "targets": {
    "linux-x64":   { "kind": "appimage",   "url": "https://….AppImage",   "size": 52428800, "sha256": "…" }
  }
}
```

### 3. The check sends no identifier, and rollout is bucketed client-side

`GET /updates/desktop?product=music&channel=stable` returns the same bytes for
every caller: no current-version parameter, no install id, no account. The
client compares versions itself and buckets itself against `rollout_percent`
using a random number drawn once at first launch and persisted locally.

Two properties fall out: the response is trivially cacheable
(`Cache-Control: public, max-age=300`, one key per product/channel), and the
update check cannot be used to count or track installs. A client that ignores
its bucket only gets a legitimately signed update early, so there is nothing to
enforce.

*Alternative considered.* Server-side bucketing on an install id — rejected: it
introduces a pseudonymous identifier and a per-client response for no benefit.

### 4. Windows: the Microsoft Store, not an installer of our own

*Revised 2026-10-24 — replaces a per-user Inno Setup installer with silent
self-update, which was implemented first and is removed.*

The `windows` job keeps its own `flutter build windows` (production endpoint and
desktop OAuth client baked in), then wraps that output into an MSIX with the
`msix` package (`--store`, no rebuild), versioned `major.minor.patch.0` — the
Store reserves the fourth field. On a `music-v*` tag it submits the package with
the Microsoft Store Developer CLI (`msstore publish -i <file>.msix`), using a
Partner Center Entra application held as repository secrets. A dry-run packages
with placeholder identity values and submits nothing, so packaging regressions
still fail the job.

The Store re-signs the package with Microsoft's certificate and owns updates, so
the app's updater is off on Windows (`AppPlatform.selfUpdates` is Linux only) and
the manifest carries no Windows target. The download page links the Store's web
installer. MSIX desktop apps run as full trust: MIDI, audio and the OAuth
loopback behave as in an unpackaged build (to confirm on a real machine).

*Alternatives considered.* A signed Inno installer with silent self-update
(implemented, then dropped: it needs a paid certificate). An unsigned installer
(blocked by Smart App Control). `msstore package`, which runs its own
`flutter build` without this repository's `--dart-define`s.

### 5. Linux: AppImage that replaces itself

The AppImage is built with the current `appimagetool`, whose static type-2
runtime embeds FUSE 3: hosts need FUSE support (`fusermount3`, present on
current desktops) but no longer `libfuse2`. It is not added to the desktop's
application menu by itself; integration tools (Gear Lever, AppImageLauncher) do
that, and the download page says so.


The AppImage runtime exports `$APPIMAGE` (the absolute path of the running file).
The updater downloads the new AppImage **into the same directory** — same
filesystem, so the swap is a single atomic `rename()` — `chmod 0755`, renames it
over `$APPIMAGE`, then re-executes that path detached and exits.

Replacing a running executable by `rename()` is safe on Linux: the running
process keeps the old inode open. (Overwriting in place is what fails, with
`ETXTBSY`.)

If `$APPIMAGE` is unset (tarball install, `flutter run`) or the directory is not
writable (`/opt`, root-owned), the updater takes the notify-only path. It never
escalates privileges.

*Alternatives considered.* A `.deb` upgraded from the app through `pkexec` and a
root-owned helper re-verifying a private copy (implemented and tested on
2026-10-24, then dropped: Debian/Ubuntu only, a password at every update, and
privileged code to maintain). `.deb` + self-hosted APT repo — apt would handle
updates with no in-app code, but needs a GPG-signed repository, the user must add
the source and have root, and the update UX leaves the product entirely.
Self-hosted Flatpak — best desktop-integrated UX, heaviest infrastructure
(ostree repo, build manifest) and the user must add a remote. Swapping the
`tar.gz` directory in place — works only for user-writable installs and is not
atomic mid-swap, so a crash leaves a broken installation.

### 6. Verify in a fixed order, and never downgrade

1. Envelope signature over the manifest bytes, against the compiled-in key for
   `key_id`. Fail ⇒ stop, nothing is written to disk.
2. Parse; reject `schema` values the client does not know.
3. `version > current` — strictly. A manifest offering an older or equal version
   is ignored, so a replayed old manifest cannot roll a user back onto a version
   with a known hole.
4. Download to a freshly created random-named directory inside the app's private
   temp dir (not a shared `/tmp` path), enforcing the declared `size` as a hard
   cap while streaming.
5. SHA-256 over the downloaded bytes must equal `sha256`.
6. Only then execute.

Because there is no downgrade path, **rollback is "pause plus ship a higher
version"**, not "re-publish the old one". That is a deliberate trade: replay
protection is worth more than a rollback button.

Version comparison uses a small `AppVersion` value type over the app's real
format (`major.minor.patch+build`, e.g. `1.24.0+32`), ordered on the triple then
the build number. Hand-rolled and unit-tested rather than pulling a semver
package, because the `+build` component is significant here and store-style
semantics differ from strict semver's "build metadata is ignored".

### 7. One manifest crate, two verifiers, one golden fixture

`crates/cymbra-update-manifest` holds the envelope/manifest types, signing and
verification (`ed25519-dalek`). The CI signing binary and the backend ingest
handler both use it. The Dart side re-implements verification with a pure-Dart
Ed25519 package — a second implementation, which is exactly where cross-language
drift hides. Both sides therefore run a test against the **same checked-in
golden fixture** (a signed manifest plus its public key, with valid and tampered
variants), so an incompatibility fails a unit test rather than an install.

### 8. State and UI

`DesktopUpdateService` (fetch/verify/download) and `UpdateInstaller` (per-platform
execution) are trait/abstract seams exposed as Riverpod providers and overridden
with generated mocks in tests, per the repo's Riverpod and testing conventions.
No test ever spawns a process.

The notifier state is one Freezed union — `idle | checking | upToDate |
available | downloading(received,total) | ready | installing | failed |
updateRequired` — and every side effect (banner, dialog, blocking screen) lives
in a dedicated listener widget near the top of the feature subtree. The UI never
awaits a notifier action's return.

Behavioural rules: the launch check is throttled to once per 24 h (persisted
timestamp) and skipped entirely on mobile/macOS and on non-desktop builds; a
settings entry forces a check ignoring the throttle; a version the user
dismissed is remembered and not re-offered; the prompt is deferred while a play
or practice session is active. A failed check is a silent no-op with a logged
cause — never a raw error string on screen.

`min_supported_version > current` produces a blocking, localized screen with a
single action, because the alternative is a client failing against the backend
with errors the user cannot act on.

### 9. (superseded) Windows signing

Artifact Signing in CI was implemented, then removed with decision 4's revision:
the Store signs Windows packages. Kept as a note so the history reads.

### 10. The Linux binary is built on the oldest supported Ubuntu

A Linux binary runs only on systems whose glibc is at least as new as the one it
was linked against. The release job used `ubuntu-latest` (24.04), so its output
refused to start on Ubuntu 22.04 and Debian 12. The `linux` job — and only that
job — now runs on `ubuntu-22.04`, the oldest image GitHub provides; every other
job keeps its image. When GitHub retires that image the job stops starting, and
moving to the next one is a deliberate decision to drop those distributions.

## Risks / Trade-offs

- **The updater is remote code execution by design.** → The verify-before-execute
  order in decision 6, a signing key held only by CI, backend re-verification on
  ingest, and no downgrade. The golden-fixture test keeps the two verifier
  implementations honest.
- **A missing Caddy allow-list entry breaks the check invisibly** — `/updates/*`
  absent from `@http` answers `200` with an empty `grpc-status: 12` rather than
  a connection error, so the client sees a malformed response and stays quiet.
  → The Caddyfile change is a blocking deploy task with an explicit
  `curl` verification against production before the first release is ingested.
- **Store certification** takes up to three business days per submission, so a
  Windows hotfix is not immediate. → Accepted; the Linux path keeps its staged
  rollout and kill-switch.
- **The Store can be disabled on managed (enterprise) PCs.** → Not the target
  audience; no public fallback, since an unsigned binary is blocked anyway.
- **The first Store submission is manual** (app reservation, listing, age rating,
  privacy policy URL) before CI can submit updates.

- **`appimagetool` needs FUSE**, which GitHub runners do not reliably provide. →
  Invoke it with `--appimage-extract-and-run`.
- **Signing key loss or compromise.** → Loss means clients stop accepting new
  releases until a build ships with a new key, so the key is backed up out of
  band at creation. Compromise is contained by shipping a build that drops the
  key from the trusted map; the map is a set, not a single value, precisely to
  allow overlap during rotation.
- **Two extra artifacts per release** to build and keep working. → Both are
  produced in the existing `windows` and `linux` jobs, and the archives stay
  published so a packaging regression never leaves users with no download at all.
- **A user on a metered connection pays for a full download** (no deltas). →
  Downloads only start after explicit consent, and the manifest carries `size`
  so the prompt can state it.

## Migration Plan

1. Ship the backend route, the table and the Caddy allow-list entry first, with
   no release ingested — the endpoint answers `204` and existing clients are
   unaffected.
2. Ship the app-side updater and the new artifacts in one release. That release
   is the first one that *can* update; users still install it by hand.
3. Ingest the following release at `rollout_percent = 0`, verify the endpoint and
   the signature against production, then raise the percentage in steps.
4. **Rollback**: set `rollout_percent = 0` for the offending version. Clients
   already updated are handled by shipping a higher fixed version — never by
   re-publishing an older one, which decision 6 forbids by design.

## Open Questions

- Resolved at implementation: the Dart Ed25519 verifier is the already-present
  `cryptography` package; ingest uses a dedicated shared secret.
- Whether anything in the app behaves differently inside an MSIX package (MIDI
  device access, audio output, the OAuth loopback port, file locations under
  package virtualization). Expected not — packaged desktop apps run as full
  trust — and confirmed on a real Windows machine before the first submission.
