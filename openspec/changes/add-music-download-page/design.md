# Design

## Context

`add-desktop-auto-update` (PR #1052, into `final/desktop-deployment`) has the release CI sign a manifest and POST it to the backend, which re-verifies and stores it (`backend/updates`, table `releases`) and serves it at `GET /updates/desktop` for the in-app updater. That endpoint answers only for releases that are not paused **and** above rollout 0, and returns a base64 envelope meant to be verified by the app. The manifest is keyed by `<os>-<arch>` and, since Windows ships through the Microsoft Store, carries only `linux-x64`, the self-updating AppImage, built on Ubuntu 22.04 so it starts on 22.04, 24.04 and Debian 12.

The site (`apps/site`, Astro, fr/en/es) has no desktop download: `lib/stores.ts` renders a dimmed "Windows / Linux — soon" button. `site-deploy` is **manual by design** (releasing and deploying are two decisions). Interactive pieces are Vue islands using the shared `@cymbra/web-auth` client.

## Goals / Non-Goals

**Goals:** a visitor reaches the latest Windows and Linux build from cymbra.app in two clicks; the page never goes stale between site deploys; the page never lies (no dead card, no foreign link); the existing updater is untouched.

**Non-Goals:** Windows distribution itself (the Store, owned by `add-desktop-auto-update`); a Linux store; `.deb`/`.rpm` packages; listing the zip/tarball in the manifest; install counting or any per-visitor data; changing rollout semantics.

## Decisions

**D1. Read the backend in the browser, not at site build time.** Deploys are manual, so a build-time read would show last deploy's version until someone re-runs `site-deploy`. The site already calls the backend from the browser (`PUBLIC_API_URL`) and its origin is already in `CYMBRA_WEB_ORIGINS`. *Alternative*: rebuild on each `music-v*` tag — rejected, it reverses the repo's "release ≠ deploy" decision and adds a cross-workflow trigger.

**D2. A separate informational endpoint, not the updater feed.** `GET /updates/desktop/latest` returns decoded JSON for display; `GET /updates/desktop` stays byte-opaque for the updater. Mixing them would either force the page to decode and verify base64 envelopes in the browser or make the updater's feed ignore rollout. *Alternative*: the page reads `/updates/desktop` — rejected: hidden at rollout 0, which is the CI's ingest state.

**D3. Rollout-agnostic, pause-respecting.** Rollout throttles *automatic* updates to protect against a bad build; a person choosing to download has no such need. Pause is the operator's stop sign for a bad release, so a paused release disappears from the page and the fallback shows (resolves the "paused release" decision: it disappears). Because a paused release makes the page fall back to an older non-paused one, an operator pulling a build must pause **every** release they want hidden; documented in the runbook.

**D4. Re-verify before projecting.** The endpoint verifies the stored envelope against the trusted keys on each (cached) read and returns `204` on failure. This preserves the "a compromised backend cannot forge a release" property for the page: it can withhold, not invent. Cost is one Ed25519 verification per cache miss.

**D5. The browser still allow-lists the link.** The page renders a link only for `https://github.com/NEETROF/cymbra/releases/download/…`. Defence in depth if the signing key were ever misused.

**D6. Windows is a Store link; Linux is the AppImage.** Windows ships through the Microsoft Store, so its card is a static Store web-installer button (product id from the site's build configuration; "coming soon" while unset) and the endpoint carries no Windows target. On Linux the manifest's `linux-x64` target is the AppImage; the tarball is reachable through the release page. No manifest schema change, no signature-compatibility fixture to regenerate.

**D7. Island follows the site's front-end rules** (`vue-frontend-architecture`): the component never calls the API; a composable owns the request behind an injectable client seam, and its state is one `ts-pattern` union (`idle | loading | success | none | error`), matched exhaustively. `none` (204) and `error` render the same fallback.

**D8. Server-render the fallback.** The Astro page emits the GitHub Releases link and the platform guidance statically; the island enhances it. No-JS and outage cases need no extra code path.

**D9. Release notes as a link only** (resolves the notes decision): the manifest carries `notes_url`; the page shows it when present and nothing when absent. No rendering of notes content.

**D11. Install guidance per platform.** Windows: installs and updates through the Microsoft Store; nothing to explain about signing or SmartScreen. Linux: the AppImage is one file for every distribution; allow it to run (file properties, or `chmod +x`); keep it in a folder the user can write to (`~/Applications`) so the app can replace it when updating — elsewhere it only notifies; it needs FUSE support, which current desktops include (the static runtime no longer needs `libfuse2`); Gear Lever or AppImageLauncher add it to the application menu. No card ever suggests turning a protection off.

**D10. Platform highlight is cosmetic.** Detected from the user agent; wrong detection only changes which card is emphasised. Both cards always render.

## Risks / Trade-offs

- **Stale CDN cache** → short `max-age` (same order as the updater feed); a just-ingested release can take minutes to show.
- **Pause surprises** (older release resurfaces) → D3 note in the runbook; the page shows the version number so it is visible.
- **No reliable "Music only" releases list**: GitHub's releases search (`?q=`) is full-text over titles and notes — checked 2026-10-24, `q=music` also returns site, backend and back-office releases, `q=music-v` returns nothing. → each card links the exact `releases/tag/music-vX.Y.Z` page (derived from the manifest version); the outage fallback links the unfiltered releases page.
- **Store product id not yet known** when the page ships. → The Windows card shows "coming soon" until the id is configured in the site's build variables; no code change needed.
- **Archive ordering** with `add-desktop-auto-update` (see proposal) → archive that change first, or `/opsx:sync`.
- **Feed route missing from the Caddy matcher** → silent `200` + empty `grpc-status: 12`; task and prod check below.
