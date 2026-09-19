## Why

`apps/site` became a hub plus one page per product (PR #484): `/` now positions
Cymbra and offers a card per app, while Music's landing content moved to `/music`
and `/en/music`. No route was deleted, but `https://cymbra.app` is the value sitting
in the **Website** (Play) and **Marketing URL** (App Store Connect) fields of the
Music listing — so a visitor arriving from either store now lands on a product
chooser instead of the app they were reading about.

The same restructure surfaced a contract nobody had written down. Music hard-codes
seven `cymbra.app` paths, and an installed build keeps requesting the path it was
compiled with **forever** — a user on 1.30 will ask for `/cgu/` long after the site
has been redesigned twice. Today nothing in the repo records which routes carry that
obligation, the site ships no `_redirects` file to soften a move, and the site's own
CI cannot fail on a deletion it has no reason to suspect.

## What Changes

- Repoint the Music listing's marketing/website URL at the product page:
  `https://cymbra.app/music/` for `fr`, `https://cymbra.app/en/music/` for `en`,
  `it` and `es`. Console-side values in both stores — no build, no resubmission of
  a binary.
- Leave every other declared listing URL exactly where it is, and say why in the
  spec rather than in a commit message: the **support** URL stays `/support/` and
  `/en/support/` (macOS 1.32.0 was rejected under guideline 1.5 for pointing support
  at the home page), privacy stays `/confidentialite/` and `/en/privacy/`, and the
  Play data-deletion URL stays `/en/delete-account/`.
- Record the full set of client-facing routes in the repo, next to the listing copy
  that already lives there, so the next person to restructure the site can see what
  they are standing on.
- Give the site a standing obligation: a route in that set is permanent. It may gain
  content, it may not move or disappear, and if one ever must move, a redirect ships
  in the same change that moves it.
- Back that obligation with a CI gate rather than prose. The routes live in one
  checked-in source (`apps/site/src/lib/pinned-routes.ts`); `site-check` asserts after
  `yarn build` that each produced its page, and fails the pull request when one is
  missing. The documentation points at that source instead of copying it — two copies
  of the list would drift and reproduce the same failure one level down.
- Make the site return a real `404`. Production currently answers **any** unmatched
  path with the French home page and status `200` — verified: `/route-qui-nexiste-pas/`
  returns `200` and the home page. A deleted `/en/privacy/` would not fail loudly; it
  would serve French marketing copy to someone who asked for the English privacy
  policy, with a status code no uptime check would flag.

No code changes in `apps/music`. The in-app URLs are correct as they stand — this
change exists partly to state that they are not free to edit.

## Capabilities

### New Capabilities

- `site-client-route-contract`: the routes `cymbra.app` must keep serving because
  shipped clients and store listings point at them, the rule that they are permanent,
  the redirect obligation when one has to move, the single machine-readable home for
  the list plus the build gate that enforces it, and the requirement that an unmatched
  path answers `404` instead of impersonating the home page.

### Modified Capabilities

- `store-distribution`: adds a requirement covering the listing's **URL fields**
  (marketing/website, support, privacy, data deletion) alongside the existing
  "Store-listing copy" requirement, which today covers description, subtitle,
  keywords and category but says nothing about URLs.

`legal-links` is deliberately **not** modified: the URLs it pins are unchanged, and
it is already being touched by the in-flight `open-app-without-sign-in-wall`.

## Impact

**Music** (consumes the site; nothing new): `apps/music/lib/services/legal_links.dart`
(`/cgu/`, `/confidentialite/`, `/en/terms/`, `/en/privacy/`) and
`apps/music/lib/screens/plan_screen.dart` (`/account`) keep their current values and
gain a documented reason not to drift. `apps/music/store/` gains the listing URL
fields next to the copy it already versions. Both store consoles need a field edit.

**Site** (new obligation, new pages, new gate): `apps/site` must honour the frozen
routes. It gains `src/lib/pinned-routes.ts` (the single source), a script asserting
them against `dist/`, and one step in `.github/workflows/site-check.yml` after the
existing `yarn build`. It ships no `_redirects` file today, so the redirect mechanism
Cloudflare Pages expects (`public/_redirects`) does not yet exist and is introduced
here only as the documented escape hatch, not as live redirects. It also has no `404`
page, which is why the host falls back to the home page; this change adds one in each
locale.

**Backend** (already pinned, recorded only): `CYMBRA_PADDLE_CHECKOUT_PAGE` points at
`https://cymbra.app/checkout` and Paddle returns to `/checkout/done`
(`apps/music/store/SUBSCRIPTIONS.md`), so those two routes are load-bearing for
billing as well as for the app.

**Not impacted**: Lingua, Live, the back office. No proto, no database, no new CI
workflow — one step inside the site's existing gate, on an already-watched unit.
