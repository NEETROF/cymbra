## 1. The route list, in one place

- [ ] 1.1 Create `apps/site/src/lib/pinned-routes.ts`: each entry is the route, the consumer pinning it, and where that consumer lives (`services/legal_links.dart`, `screens/plan_screen.dart`, `CYMBRA_PADDLE_CHECKOUT_PAGE`, App Store Connect, Play Console). This file is the single source — the gate and the docs both read it.
- [ ] 1.2 Build the list from the sources, not from this change's prose: `grep -rn "https://cymbra.app" apps/music/lib` must yield exactly the five in-app URLs, and `apps/music/store/SUBSCRIPTIONS.md` must still pin `/checkout` and `/checkout/done`.
- [ ] 1.3 Add a "Routes shipped clients depend on" section to `apps/site/README.md` that explains the rule — a pinned route may change content but never move — and **points at `pinned-routes.ts`** instead of restating the list.
- [ ] 1.4 State the redirect obligation in the same section: a move ships a `301` from the old path in `public/_redirects` in the same change, host-side (a native app opening an external browser never runs client-side script).
- [ ] 1.5 Add a pointer from `apps/music/store/README.md` to that section, so the operator arriving from the store side finds it without reading the site's docs end to end.

## 2. The gate

- [ ] 2.1 Add a script (`apps/site/scripts/check-pinned-routes.mjs`) that imports `pinned-routes.ts`, and for each route asserts `dist/<path>/index.html` exists; on failure it exits non-zero and names every missing route, not just the first.
- [ ] 2.2 Wire it as a `yarn check:routes` script and add the step to `.github/workflows/site-check.yml` immediately after `yarn build` — `dist/` only exists from that point.
- [ ] 2.3 Prove the gate actually fails: delete a pinned page locally, run `yarn build && yarn check:routes`, confirm a non-zero exit naming that route, then restore it. A gate never seen failing is not known to work.
- [ ] 2.4 Confirm the gate passes on the untouched tree.

## 3. Record the listing URL fields

- [ ] 3.1 Add a "Listing URLs" section to `apps/music/store/copy/fr.md` with marketing `https://cymbra.app/music/`, support `https://cymbra.app/support/`, privacy `https://cymbra.app/confidentialite/`.
- [ ] 3.2 Add the same section to `apps/music/store/copy/{en,it,es}.md` with marketing `https://cymbra.app/en/music/`, support `https://cymbra.app/en/support/`, privacy `https://cymbra.app/en/privacy/`.
- [ ] 3.3 Record the Play data-deletion URL `https://cymbra.app/en/delete-account/` once, in `apps/music/store/README.md` — it is a single console-level field, not per-locale.
- [ ] 3.4 Note in `apps/music/store/README.md` that the support URL is `/support/` deliberately and must never be pointed at the home or marketing page, citing the macOS 1.32.0 guideline 1.5 rejection.

## 4. Give the site a real 404

- [ ] 4.1 Add `src/pages/404.astro` (fr) and `src/pages/en/404.astro`, using `Base.astro` so the not-found page carries the header, the footer and the language switch.
- [ ] 4.2 Confirm Cloudflare Pages stops falling back to the home page once a `404.html` exists in `dist/`; if the project is configured in single-page-application mode, turn that off — it is what produces the `200` today.
- [ ] 4.3 Re-verify against the deployed site: `/route-qui-nexiste-pas/` returns `404`, and `/en/route-qui-nexiste-pas/` returns the English not-found page.

## 5. Confirm the target pages before touching a console

- [ ] 5.1 Confirm the deployed site actually serves the new pages. As of 2026-09-18, production still served the **old** build: `/music/` returned `200` but with the old home page's title, which is the fallback, not the page. Compare the served title, never the status code.
- [ ] 5.2 If the site has not redeployed since PR #484, deploy it first — the `site-deploy` workflow runs on a `site-vX.Y.Z` tag (release-please) or a manual dispatch. Pointing a store field at a path that silently serves the hub is no better than pointing it at the hub.
- [ ] 5.3 Check `https://cymbra.app/support/` and `/en/support/` still render support content and were untouched by the restructure.

## 6. Edit the App Store Connect listing

- [ ] 6.1 Set the Marketing URL to `https://cymbra.app/en/music/` for `en`, `it` and `es`, and `https://cymbra.app/music/` for `fr`, on **both** platforms — the field is platform × locale, so editing macOS does not cover iOS (see `app-store-metadata-scope`).
- [ ] 6.2 Leave the Support URL untouched on all four locales and both platforms; confirm it still reads `/support/` or `/en/support/` after saving.
- [ ] 6.3 Record whether the edit saved against the live version or opened a new version needing review, and say so in the change's follow-up notes — this has not been verified for this account.

## 7. Edit the Play Console listing

- [ ] 7.1 Set the store listing Website field to `https://cymbra.app/music/` for `fr` and `https://cymbra.app/en/music/` for `en`, `it` and `es`, in *Grow → Store presence → Main store listing*, per language.
- [ ] 7.2 Leave the privacy policy and data-deletion URLs untouched; confirm they still read `/confidentialite/` (fr), `/en/privacy/` and `/en/delete-account/`.
- [ ] 7.3 Submit the listing edit if the console requires a review pass, and note that the previous value still resolves meanwhile.

## 8. Verify the live listings

- [ ] 8.1 Fetch every pinned route against production and confirm each serves **its own page**, by comparing the returned `<title>` against the expected one. A status-code check proves nothing here: until task 3 lands, every path returns `200`.
- [ ] 8.2 Confirm the App Store listing's Website link resolves to the Music product page in each published locale.
- [ ] 8.3 Confirm the Play listing's Website link resolves to the Music product page in each published locale.
- [ ] 8.4 Confirm the support link on both listings still lands on the support page and not on a marketing surface.

## 9. Close out

- [ ] 9.1 Run `openspec validate pin-music-site-url-contract --strict`.
- [ ] 9.2 Open the follow-up for renaming `store-distribution` to `music-store-distribution` — deferred here on purpose (the rename is a folder move outside the delta mechanism, and it should settle the merge-or-split question with `music-macos-store-distribution` at the same time).
- [ ] 9.3 Confirm `ci-units` still passes — this change adds a script and a workflow step under `apps/site`, an already-watched unit, so no paths filter should need editing (`python3 scripts/check_ci_units.py`).
