# Proposal

## Why

Cymbra Music for Windows ships through the Microsoft Store and for Linux as a self-updating AppImage (`add-desktop-auto-update`), yet nothing on cymbra.app leads a visitor to either: the Music page's desktop button is a dimmed "Windows / Linux — soon" placeholder, and the only route to a build is to find the GitHub Releases page by hand. Web-purchase subscribers on those platforms land on a post-checkout block that offers only store links. The release pipeline already publishes a signed manifest to the backend for every release, so the site can show the latest version without a second source of truth.

## What Changes

- **New public, anonymous, read-only endpoint** `GET /updates/desktop/latest` on the backend update feed. It returns an informational JSON view (version, release date, notes link, and per platform: install kind, download URL, size, SHA-256) of the highest **non-paused** release, **ignoring the staged-rollout percentage** (rollout throttles automatic updates, not a visitor's deliberate download). It re-verifies the stored envelope's signature before projecting it, so a tampered row yields nothing. `204` when nothing is available. Short shared-cache lifetime. CORS limited to the configured web origins. Added to the Caddy `@http` allow-list.
- **A "Download" section on the site's Music page** (fr / en / es). **Windows**: a Microsoft Store install button using the Store's web installer (a small Microsoft-signed `.exe` that installs from the Store without opening it), with the Store product id from the site's build configuration. **Linux**: the AppImage card filled **in the visitor's browser** from that endpoint — version, date, size, SHA-256, download link — with the steps an AppImage needs (allow it to run, keep it in a writable folder such as `~/Applications` so it updates itself, FUSE, menu integration through Gear Lever / AppImageLauncher) and the tarball as an alternative. The visitor's own platform is highlighted; both are always shown.
- **Graceful fallback**: if the endpoint is unreachable, answers `204`, or the script does not run, the section still renders a plain link to the GitHub Releases page (where the tarball lives).
- **The same download block** replaces the dimmed placeholder in the shared store links and is shown on the account page and on the page after a web purchase (French and English, the languages those pages exist in).
- **No change to the manifest schema or the updater.** The tarball stays GitHub-only; the manifest keeps one target per `<os>-<arch>` and carries only `linux-x64`.

## Capabilities

### New Capabilities
- `site-music-downloads`: what the Music page and the post-checkout landing show for the Windows and Linux builds, how they degrade, and what they tell a visitor installing them.

### Modified Capabilities
- `platform-app-update-feed`: gains a requirement for the public latest-release view (anonymous, rollout-agnostic, pause-respecting, signature re-verified, browser-readable). **Ordering**: this capability exists today only as a delta of the in-flight `add-desktop-auto-update`; that change MUST be archived (or its specs synced with `/opsx:sync`) before this one is archived, as `swap-store-billing-to-revenuecat` did for its predecessor. Implementation lands on top of that change's code (PR #1052).

**Ordering.** This change builds on `add-desktop-auto-update` (PR #1052), whose code and `platform-app-update-feed` capability it extends. Both live on the integration branch `final/desktop-deployment` and reach `main` together; that change MUST be archived before this one.

## Impact

Products: **site** (new section, new island, shared store links, post-checkout block — consumes the feed, owns nothing of it); **platform / backend** (`backend/server/src/updates.rs` and the `backend/updates` crate: one read route, no migration, no new secret); **Music app**: untouched; **ID, Live, back-office**: untouched.

- Ops: `/updates/desktop/latest` in the Caddy `@http` matcher (else it falls to tonic and answers `200` with an empty `grpc-status: 12`); CORS follows the existing `web_lingua.rs` pattern (`CYMBRA_WEB_ORIGINS`, no credentials); the site origin is expected to be in that list already for web sign-in — confirm on the prod box.
- Site deploys stay manual (`site-deploy`): the page reads the live endpoint, so a new release shows up without redeploying the site.
- Open decisions (resolved in design.md): release notes shown as a link; a paused release disappears from the page.
