# Tasks

## 1. Prerequisite

- [x] 1.1 Bring `claude/desktop-auto-update-466cf2` (1 commit, 790 behind `main` on 2026-10-24) onto the integration branch — done as PR #1052 into `final/desktop-deployment`, which this change also targets: its CI edits target `release-build.yml`, now `music-release.yml`, and 22 files it touches also moved on `main` (Cargo, `main.rs`, `config.rs`, roles, Caddyfile, DEPLOY.md, env examples, ARB files). Verify with `cargo test -p cymbra-updates -p cymbra-update-manifest` and `flutter test test/services/update test/state/update_notifier_test.dart`; this change is not mergeable before that one.
- [x] 1.2 Regenerate the release signing keypair — done in #1052 (`2026-10-a`).

## 2. Backend: latest-release view

- [ ] 2.1 Add a pure `select_latest_downloadable` (not paused, highest `version_order`, rollout ignored) next to `select_servable` in `backend/updates/src/core.rs`; unit tests for: rollout 0 included, newest paused skipped, all paused, empty, `1.9.0` vs `1.10.0` ordering.
- [ ] 2.2 Add the view type and projection (version, `released_at`, `notes_url`, `targets` with kind/url/size/sha256) built from a verified manifest; unit test that no signature, key id or rollout field leaks.
- [ ] 2.3 Add `GET /updates/desktop/latest` in `backend/server/src/updates.rs`: re-verify the envelope against the trusted keys, `204` on verify failure / unknown product or channel / nothing available, `200` JSON otherwise with `Cache-Control: public, max-age=…`; router tests over the repository trait with `mockall` mocks (200, 204 ×4, byte-identical responses, no request identifier read).
- [ ] 2.4 Restrict cross-origin reads of this route to `CYMBRA_WEB_ORIGINS`, without credentials, following `backend/server/src/web_lingua.rs` (the `/updates/desktop` route itself keeps no CORS layer); test an allowed origin, a foreign origin, and no `Origin`.
- [ ] 2.5 Confirm the Caddy `@http` matcher entry added for `/updates/desktop` also covers `/updates/desktop/latest` (widen it to `/updates/*` if not), and add to `backend/deploy/DEPLOY.md`: `curl -i https://<prod>/updates/desktop/latest` answers `204` or `200` JSON, never `200` with `grpc-status: 12`.
- [ ] 2.6 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo llvm-cov` stays ≥ 80 %.

## 3. Site: client and island

- [ ] 3.1 Add `apps/site/src/lib/desktop-release.ts`: the response type, the GitHub-asset address check (D5), the releases-page URL constant and the per-version `releases/tag/music-v<major.minor.patch>` link (the manifest's `1.37.0+51` is tagged `music-v1.37.0`: the build number is dropped); unit tests for accepted and rejected addresses and for the tag link.
- [ ] 3.2 Add the composable `useLatestDesktopRelease.ts` behind an injectable client seam with the `Async` union (`idle | loading | success | none | error`); tests for success, 204 → `none`, network failure and malformed body → `error`, foreign address filtered.
- [ ] 3.3 Add `MusicDownloadsIsland.vue` rendering a card per platform (version, date, size, SHA-256, link, notes link, install steps), platform highlight, and the fallback for `none`/`error`; component tests with the fake client for each state and for a platform with no target.
- [ ] 3.4 Add the fr / en / es strings (card titles, steps, fallback, "not available yet") with no key missing in any locale; run the site's i18n/lint check.

## 4. Site: pages

- [ ] 4.1 Add the "Download" section to `music.astro` and its `en/` and `es/` twins: statically rendered releases-page link and platform guidance, island mounted on top; verify the built HTML contains the link with scripts disabled.
- [ ] 4.2 Replace the dimmed `DESKTOP_SOON` entry in `lib/stores.ts` with an anchor to the section, render the island in `Downloads.astro` (used by `/account`), and add `Downloads` to `checkout/done.astro` and `en/checkout/done.astro`, which today only link to the account; check no "coming soon" text remains in any locale.
- [ ] 4.3 `site-check` passes (build, typecheck, tests); add the Music page to the sitemap/pinned-routes tests only if they enumerate sections.

## 5. Integration checks

- [ ] 5.1 Against a local backend with one ingested rollout-0 release: the page shows its version, both links resolve to GitHub assets, and the SHA-256 shown equals the downloaded file's.
- [ ] 5.2 Pause that release: the page shows the fallback within the cache lifetime; unpause: it returns.
- [ ] 5.3 Install the `.deb` from the page's link on clean Ubuntu 22.04, 24.04 and Debian 12 (software installer and `apt`), and run the Windows installer from it on a machine with Smart App Control on; adjust the cards' wording to what actually shows (D11).
- [ ] 5.4 Stop the backend: the fallback shows with no error text; re-check in all three locales and on a phone-width viewport.
- [ ] 5.5 `openspec validate add-music-download-page --strict` passes (with `add-desktop-auto-update` available for the delta).
