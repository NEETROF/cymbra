## 1. Manifest contract (shared crate + cross-language fixture)

- [x] 1.1 Create `crates/update-manifest` (`cymbra-update-manifest`), added to the workspace members, with `serde` + `ed25519-dalek` and no backend dependency.
- [x] 1.2 Define the manifest types: `schema`, `product`, `channel`, `version`, `released_at`, `min_supported_version`, `notes_url`, and `targets` keyed by `<os>-<arch>` with `kind`, `url`, `size`, `sha256`.
- [x] 1.3 Define the envelope type: base64 `manifest` bytes, base64 `signature`, `key_id`, `rollout_percent` — with the manifest bytes kept opaque (never re-serialized between verify and parse).
- [x] 1.4 Implement `sign(manifest_bytes, secret_key) -> Envelope` and `verify(envelope, trusted_keys) -> Result<Manifest>`, rejecting unknown `key_id`, bad signature, and unknown `schema`.
- [x] 1.5 Unit-test sign/verify: valid round trip, tampered manifest bytes, tampered signature, unknown `key_id`, unknown `schema`, malformed base64.
- [x] 1.6 Add a `keygen` helper (behind a bin or a test-only fn) and generate the release keypair; record the public key and keep the private key out of the repo.
- [x] 1.8 Regenerate the release keypair: the private key of `2026-08-a` was lost before reaching a secret (no build ever shipped its public key). New `key_id`, public key in `update_signing_keys.dart`, in the helper's compiled-in key set and in `CYMBRA_UPDATE_TRUSTED_KEYS`; private key straight into `DESKTOP_UPDATE_SIGNING_KEY` + an offline backup. Verify: `gh secret list` shows it; golden fixtures unchanged (they use the fixed test key). Done 2026-10-24: `key_id` `2026-10-a`, secret set, offline backup by the owner.
- [x] 1.7 Check in the golden fixture under `crates/update-manifest/fixtures/`: a signed manifest, its public key, and tampered variants — consumed by both the Rust and the Dart tests.

## 2. Backend update feed

- [x] 2.1 Add the migration creating the releases table (product, channel, version, envelope bytes, signature, key id, rollout percent, paused, timestamps) with a unique key on (product, channel, version).
- [x] 2.2 Add a repository trait + Postgres implementation: upsert a release, fetch the highest servable release for a product/channel (not paused, rollout > 0).
- [x] 2.3 Create `backend/server/src/updates.rs` with the Axum router: `GET /updates/desktop` (public) and `POST /updates/desktop` (credential-gated ingest).
- [x] 2.4 Implement the public handler: return `200` with the stored envelope + `Cache-Control: public, max-age=300`, or `204` when nothing is servable — identical bytes for every caller, no request-scoped input beyond product/channel.
- [x] 2.5 Implement the ingest handler: check the credential in constant time, **verify the signature via `cymbra-update-manifest` before storing**, reject unsigned/foreign-key/malformed payloads, upsert on (product, channel, version).
- [x] 2.6 Wire config: trusted public key(s) and the ingest credential from env; document them in `backend/.env.example` and `backend/deploy/.env.prod.example`.
- [x] 2.7 Mount the router next to the existing HTTP surface in `backend/server/src/lib.rs` / `main.rs`.
- [x] 2.8 Decision-logic unit tests (host-testable, no Postgres): servable selection, paused, rollout zero, unknown product/channel → 204, ingest credential failure, ingest signature failure, re-ingest replaces.
- [x] 2.9 Handler/router tests over the trait seam with `mockall` mocks, per the `rust-testing` skill.
- [x] 2.10 Move the router into `backend/updates` (the module owning `app_updates`); the server only mounts it (`platform-module-boundaries`). Verified: `cargo test -p cymbra-updates` 33 passed.

## 3. Production edge and deploy

- [x] 3.1 Add `/updates/*` to the `@http` path matcher in [Caddyfile:39](../../../backend/deploy/Caddyfile:39) — without it the path falls through to tonic and answers `200` with an empty `grpc-status: 12`.
- [x] 3.2 Document the deploy step in `backend/deploy/DEPLOY.md`: git pull + `caddy reload` on the box, then `curl -i https://<prod>/updates/desktop?product=music&channel=stable` expecting `204` (not a gRPC-shaped response).
- [x] 3.3 Add the new env vars to the prod compose/env files and to the deploy checklist.

## 4. Windows packaging — Microsoft Store

*Revised 2026-10-24: the per-user Inno installer, its silent self-update and the Artifact Signing step (implemented first, groups 4.1–4.9 of the earlier plan) are replaced by the Microsoft Store; their code is removed.*

- [x] 4.1 Remove the Inno installer (`apps/music/windows/installer/`), the Windows signing steps and secrets documentation, the `WindowsUpdateInstaller` and its tests; the Windows build never self-updates (`AppPlatform.selfUpdates` is Linux only) and the manifest carries no Windows target. Verify: no `inno`/`signtool`/`ARTIFACT_SIGNING` left outside the archive notes; installer and notifier tests pass.
- [x] 4.2 Add the `msix` dev dependency and its `msix_config` (display name, logo, languages, capabilities, `store: true`), the Store identity values coming from repository variables at package time. Verify: `flutter analyze` clean.
- [x] 4.3 In the `windows` job: package the release build with `msix:create --store --build-windows false`, version `major.minor.patch.0`; placeholder identity on a dry run. Verify: the dry-run job produces an `.msix`. Verified 2026-10-24, run 40121542599: `cymbra-dryrun-windows-x64.msix` (76 MB) built from the release build with the placeholder identity.
- [ ] 4.4 On a `music-v*` tag: install the Microsoft Store Developer CLI, `msstore reconfigure` from the Partner Center secrets, `msstore publish -i <msix> -id <product>`; fail the job if the credentials are missing. Stop attaching the portable zip to the GitHub Release.
- [x] 4.5 Document the Partner Center setup (company account, app reservation, first manual submission, Entra app for the API, the secrets and variables) in the workflow header and `apps/music/README.md`.
- [x] 4.6 The runner version resource names the company (`CompanyName`, `LegalCopyright`), consistent with the Store publisher.

## 5. Linux packaging — AppImage

*Revised 2026-10-24: a `.deb` upgraded through `pkexec` and a root helper was implemented, tested (CI dry-run, Docker end to end) and then dropped by the owner for the AppImage implemented in August; the `.deb` recipe, the helper and their CI steps are removed.*

- [x] 5.1 Move the `linux` job of `music-release.yml` — and only it — to `ubuntu-22.04`, with a comment on why and on what retiring that image will mean. Verified 2026-10-24, run 40114401284: highest required symbol `GLIBC_2.34`.
- [ ] 5.2 Restore `apps/music/linux/appimage/` (`AppRun`, `.desktop`, icon, `build_appdir.sh`) and the `appimagetool --appimage-extract-and-run` packaging in the `linux` job, keeping the tarball. Verify: the dry-run builds the AppImage.
- [ ] 5.3 Smoke-check in CI: the AppImage is executable, unpacks to the expected layout, and the bundled binary's glibc floor is reported. Verify: in the dry-run log.
- [ ] 5.4 Attach the AppImage to the release (tag-gated) and hand it to the manifest job as the `linux-x64` target of kind `appimage`.
- [x] 5.5 Remove the `.deb` recipe, the privileged helper (`cymbra-update-helper`, `release-keys.json`) and their tests and CI steps.

## 6. Release pipeline — sign, publish, ingest

- [x] 6.1 Add the CI signing binary (bin in `crates/update-manifest`) that takes the artifact URLs/sizes/hashes and emits the signed envelope.
- [x] 6.2 Add a `manifest` job depending on `windows` and `linux`: compute sizes + SHA-256 of the two artifacts, build the manifest, sign it with the `DESKTOP_UPDATE_SIGNING_KEY` secret.
- [x] 6.3 Attach the signed envelope to the GitHub Release as an asset (audit trail independent of the backend).
- [x] 6.4 POST the envelope to the backend ingest with `rollout_percent = 0`, using the ingest credential secret.
- [x] 6.5 Gate the whole job on a real `music-v*` tag: a branch dry-run builds and packages but signs, uploads and ingests nothing.
- [x] 6.6 Store the signing key and ingest credential as repository secrets; document them in the workflow header comment next to the existing secret documentation.

## 7. App — verification and service layer

- [x] 7.1 Add the Ed25519 verification dependency to `apps/music/pubspec.yaml` and pin it.
- [x] 7.2 Add `lib/services/update/update_manifest.dart`: envelope/manifest models, base64 decode, signature verification over the exact bytes, `schema` guard — pure Dart, no I/O.
- [x] 7.3 Add `lib/services/update/update_signing_keys.dart`: the compiled-in trusted key **set** keyed by `key_id` (a set, not a single value, so rotation can overlap).
- [x] 7.4 Add `lib/services/update/app_version.dart`: `AppVersion` value type over `major.minor.patch+build` with a total ordering, parsing and comparison.
- [x] 7.5 Add `DesktopUpdateService` (abstract + HTTP implementation): fetch the feed, verify, select the target for the running os/arch, reject non-newer versions.
- [x] 7.6 Implement the download path: unique directory inside the app's private temp storage, streamed download with progress, hard size cap, incremental SHA-256, delete on mismatch.
- [x] 7.7 Add the `UpdateInstaller` seam plus a `ProcessRunner` seam so no test ever spawns a process.
- [x] 7.8 ~~`WindowsUpdateInstaller`~~ — implemented, then removed with the move to the Microsoft Store (4.1).
- [x] 7.9 Restore `LinuxUpdateInstaller` (AppImage): read `$APPIMAGE`, verify the directory is writable, stage beside it, `chmod 0755`, atomic rename over the running file, relaunch detached, exit; unset or read-only ⇒ notify-only. (The `.deb` installer that replaced it on 2026-10-24 is removed.)
- [x] 7.10 Add `lib/services/update/rollout_bucket.dart`: draw once, persist in `shared_preferences`, compare against `rollout_percent`.
- [x] 7.11 Expose all seams as Riverpod providers per the `flutter-riverpod-architecture` skill.

## 8. App — state, UI and localization

- [x] 8.1 Add the Freezed `UpdateState` union: `idle | checking | upToDate | available | downloading(received,total) | ready | installing | failed | updateRequired`.
- [x] 8.2 Add `UpdateNotifier` (`@riverpod`): launch check with the 24 h throttle, forced manual check, skipped-version memory, rollout gate, feature-flag gate, platform gate (desktop only), single in-flight operation.
- [x] 8.3 Persist the throttle timestamp and the skipped version in `shared_preferences`.
- [x] 8.4 Gate the whole feature behind a `cymbra_flags` runtime flag.
- [x] 8.5 Add the dedicated listener widget near the top of the app subtree that turns state into effects — banner, dialog, blocking screen — with no side effects in build methods.
- [x] 8.6 Defer the prompt while a play or practice session is active, resuming when it ends.
- [x] 8.7 Add the update prompt UI: version, release-notes link, download size, Update / Later / Skip this version.
- [x] 8.8 Add the download-progress UI and the not-self-installable variant that opens the release page instead.
- [x] 8.9 Add the blocking forced-update screen for `min_supported_version`, dismissible only by updating.
- [x] 8.10 Add the manual "check for updates" entry showing the current version, in the profile/account area; hidden on store-managed platforms.
- [x] 8.11 Add the strings to all four ARB files (`app_en`, `app_fr`, `app_es`, `app_it`) — no raw exception, status code or transport string ever reaches the UI.
- [x] 8.12 Log every failure cause through the existing diagnostic logging.

## 9. Tests and gates

- [x] 9.1 Dart unit tests for `AppVersion` ordering, including the `+build` tiebreaker and malformed input.
- [x] 9.2 Dart tests for manifest verification against the **checked-in golden fixture** — valid, tampered bytes, tampered signature, unknown key id, unknown schema — proving Rust/Dart compatibility.
- [x] 9.3 Dart tests for the service: no-downgrade, size cap exceeded, checksum mismatch deletes and refuses to execute, target selection per os/arch, missing target.
- [x] 9.4 Notifier tests with mockito-generated mocks via `ProviderScope` overrides: throttle honoured, manual check bypasses throttle and rollout, skipped version not re-offered, flag off ⇒ no check, non-desktop ⇒ no check, prompt deferred during a session.
- [x] 9.5 Installer tests over the `ProcessRunner` seam (the Windows and AppImage cases are superseded by 4.1 and 9.9).
- [x] 9.9 AppImage installer tests over the `ProcessRunner` seam: chmod before rename, atomic swap, relaunch, `$APPIMAGE` unset or read-only ⇒ notify-only, failed chmod leaves the old file, stale staging file replaced.
- [x] 9.6 Widget tests: prompt, progress, notify-only variant, blocking forced-update screen.
- [x] 9.7 Rust tests for the feed handlers and the manifest crate, including the same golden fixture.
- [x] 9.8 `cargo llvm-cov` ≥ 80 % and `flutter test --coverage` ≥ 80 % for the new code; `melos run analyze`, `dart run custom_lint`, `dart format`, `cargo fmt`, `cargo clippy -D warnings` all clean.

## 10. Manual validation

- [ ] 10.1 Deploy the backend + Caddy change and confirm `GET /updates/desktop` returns `204` in production before any release is ingested.
- [x] 10.2 Build both artifacts from a dispatch dry-run and confirm the workflow signs, uploads and ingests nothing. Verified 2026-10-24, run 40114401284: all six build jobs green, `manifest` skipped, release uploads, Play, TestFlight and the Discord post not run; Windows built unsigned with the notice.
- [ ] 10.3 Install the Store build on a real Windows 11 machine with Smart App Control on: no warning, MIDI keyboard, sound, Google sign-in (loopback) and web checkout work inside the package.
- [ ] 10.4 Run the AppImage on clean Ubuntu 22.04, Ubuntu 24.04, Debian 12 and Fedora desktops: launches, MIDI keyboard and sound work; note what an integration tool (Gear Lever) adds to the menu.
- [ ] 10.5 Tag a release, verify the envelope asset, ingest at rollout 0, then raise it and confirm the client picks it up.
- [ ] 10.6 Submit a second version through the tag: certification passes and the installed app updates through the Store.
- [ ] 10.7 End-to-end Linux AppImage update: no password asked, file replaced in place, relaunch on the new version; an AppImage in a read-only directory shows notify-only.
- [ ] 10.8 Negative paths on real builds: the tarball shows notify-only; a manifest with a bad signature is refused and nothing is downloaded.
- [ ] 10.9 Set `rollout_percent = 0` and confirm the release stops being offered within the cache lifetime.
- [ ] 10.10 Confirm a forced update below `min_supported_version` blocks, in all four locales.
