# Design

## Context

`add-desktop-auto-update` (PR #1052, into `final/desktop-deployment`) has the release CI sign a manifest and POST it to the backend, which re-verifies and stores it (`backend/updates`, table `releases`) and serves it at `GET /updates/desktop` for the in-app updater. That endpoint answers only for releases that are not paused **and** above rollout 0, and returns a base64 envelope meant to be verified by the app. The manifest is keyed by `<os>-<arch>` with one target each (`inno-setup`, `deb`). The Windows installer and its binaries are Authenticode-signed with Artifact Signing (publisher NEETROF); the Linux `.deb` is built on Ubuntu 22.04 and installs on 22.04, 24.04 and Debian 12.

The site (`apps/site`, Astro, fr/en/es) has no desktop download: `lib/stores.ts` renders a dimmed "Windows / Linux — soon" button. `site-deploy` is **manual by design** (releasing and deploying are two decisions). Interactive pieces are Vue islands using the shared `@cymbra/web-auth` client.

## Goals / Non-Goals

**Goals:** a visitor reaches the latest Windows and Linux build from cymbra.app in two clicks; the page never goes stale between site deploys; the page never lies (no dead card, no foreign link); the existing updater is untouched.

**Non-Goals:** code signing itself (owned by `add-desktop-auto-update`) or SmartScreen reputation; a Linux store; an `.rpm`; listing the zip/tarball in the manifest; install counting or any per-visitor data; changing rollout semantics.

## Decisions

**D1. Read the backend in the browser, not at site build time.** Deploys are manual, so a build-time read would show last deploy's version until someone re-runs `site-deploy`. The site already calls the backend from the browser (`PUBLIC_API_URL`) and its origin is already in `CYMBRA_WEB_ORIGINS`. *Alternative*: rebuild on each `music-v*` tag — rejected, it reverses the repo's "release ≠ deploy" decision and adds a cross-workflow trigger.

**D2. A separate informational endpoint, not the updater feed.** `GET /updates/desktop/latest` returns decoded JSON for display; `GET /updates/desktop` stays byte-opaque for the updater. Mixing them would either force the page to decode and verify base64 envelopes in the browser or make the updater's feed ignore rollout. *Alternative*: the page reads `/updates/desktop` — rejected: hidden at rollout 0, which is the CI's ingest state.

**D3. Rollout-agnostic, pause-respecting.** Rollout throttles *automatic* updates to protect against a bad build; a person choosing to download has no such need. Pause is the operator's stop sign for a bad release, so a paused release disappears from the page and the fallback shows (resolves the "paused release" decision: it disappears). Because a paused release makes the page fall back to an older non-paused one, an operator pulling a build must pause **every** release they want hidden; documented in the runbook.

**D4. Re-verify before projecting.** The endpoint verifies the stored envelope against the trusted keys on each (cached) read and returns `204` on failure. This preserves the "a compromised backend cannot forge a release" property for the page: it can withhold, not invent. Cost is one Ed25519 verification per cache miss.

**D5. The browser still allow-lists the link.** The page renders a link only for `https://github.com/NEETROF/cymbra/releases/download/…`. Defence in depth if the signing key were ever misused.

**D6. Installer and `.deb` only.** The manifest's `<os>-<arch>` key admits one artifact per platform; the portable zip and tarball are reachable through the GitHub Releases link. No manifest schema change, no signature-compatibility fixture to regenerate.

**D7. Island follows the site's front-end rules** (`vue-frontend-architecture`): the component never calls the API; a composable owns the request behind an injectable client seam, and its state is one `ts-pattern` union (`idle | loading | success | none | error`), matched exhaustively. `none` (204) and `error` render the same fallback.

**D8. Server-render the fallback.** The Astro page emits the GitHub Releases link and the platform guidance statically; the island enhances it. No-JS and outage cases need no extra code path.

**D9. Release notes as a link only** (resolves the notes decision): the manifest carries `notes_url`; the page shows it when present and nothing when absent. No rendering of notes content.

**D11. Install guidance per platform, for signed and packaged builds.** Windows: the installer is signed, so Smart App Control lets it run, but SmartScreen may still show a notice while the publisher's reputation is young; the card shows what it looks like, that the publisher named is NEETROF, and that "More info → Run anyway" is safe *only* when that name is shown. It never suggests turning a protection off. Linux: the `.deb` opens in the distribution's software installer, or `sudo apt install ./cymbra-music…deb` (`apt`, not `dpkg -i`, so dependencies resolve); other distributions take the tarball, which does not update itself.

**D10. Platform highlight is cosmetic.** Detected from the user agent; wrong detection only changes which card is emphasised. Both cards always render.

## Risks / Trade-offs

- **Stale CDN cache** → short `max-age` (same order as the updater feed); a just-ingested release can take minutes to show.
- **Pause surprises** (older release resurfaces) → D3 note in the runbook; the page shows the version number so it is visible.
- **No reliable "Music only" releases list**: GitHub's releases search (`?q=`) is full-text over titles and notes — checked 2026-10-24, `q=music` also returns site, backend and back-office releases, `q=music-v` returns nothing. → each card links the exact `releases/tag/music-vX.Y.Z` page (derived from the manifest version); the outage fallback links the unfiltered releases page.
- **SmartScreen notice on early releases**, signed or not, until reputation builds. → Explained on the card with the publisher name (D11); signing lets reputation carry across releases.
- **A release built before signing is configured** would be blocked by Smart App Control. → `add-desktop-auto-update` makes a `music-v*` tag refuse to publish unsigned Windows binaries, so the page can never link one.
- **Archive ordering** with `add-desktop-auto-update` (see proposal) → archive that change first, or `/opsx:sync`.
- **Feed route missing from the Caddy matcher** → silent `200` + empty `grpc-status: 12`; task and prod check below.
