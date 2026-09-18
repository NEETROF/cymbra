## 1. Record the route contract in the repository

- [ ] 1.1 Add a "Routes shipped clients depend on" section to `apps/site/README.md`, under the page structure added by PR #484: the table of pinned routes with the consumer pinning each (`legal_links.dart`, `plan_screen.dart`, `CYMBRA_PADDLE_CHECKOUT_PAGE`, the two store consoles), and the rule that they may change content but never move.
- [ ] 1.2 State the redirect obligation in the same section: a move ships a `301` from the old path in `public/_redirects` in the same change, host-side (a native app opening an external browser never runs client-side script).
- [ ] 1.3 Add a pointer from `apps/music/store/README.md` to that section, so the operator arriving from the store side finds it without reading the site's docs end to end.
- [ ] 1.4 Verify the recorded list against the sources rather than against this change: `grep -rn "https://cymbra.app" apps/music/lib` returns exactly the five in-app URLs, and `apps/music/store/SUBSCRIPTIONS.md` still pins `/checkout` and `/checkout/done`.

## 2. Record the listing URL fields

- [ ] 2.1 Add a "Listing URLs" section to `apps/music/store/copy/fr.md` with marketing `https://cymbra.app/music/`, support `https://cymbra.app/support/`, privacy `https://cymbra.app/confidentialite/`.
- [ ] 2.2 Add the same section to `apps/music/store/copy/{en,it,es}.md` with marketing `https://cymbra.app/en/music/`, support `https://cymbra.app/en/support/`, privacy `https://cymbra.app/en/privacy/`.
- [ ] 2.3 Record the Play data-deletion URL `https://cymbra.app/en/delete-account/` once, in `apps/music/store/README.md` — it is a single console-level field, not per-locale.
- [ ] 2.4 Note in `apps/music/store/README.md` that the support URL is `/support/` deliberately and must never be pointed at the home or marketing page, citing the macOS 1.32.0 guideline 1.5 rejection.

## 3. Give the site a real 404

- [ ] 3.1 Add `src/pages/404.astro` (fr) and `src/pages/en/404.astro`, using `Base.astro` so the not-found page carries the header, the footer and the language switch.
- [ ] 3.2 Confirm Cloudflare Pages stops falling back to the home page once a `404.html` exists in `dist/`; if the project is configured in single-page-application mode, turn that off — it is what produces the `200` today.
- [ ] 3.3 Re-verify against the deployed site: `/route-qui-nexiste-pas/` returns `404`, and `/en/route-qui-nexiste-pas/` returns the English not-found page.

## 4. Confirm the target pages before touching a console

- [ ] 4.1 Confirm the deployed site actually serves the new pages. As of 2026-09-18, production still served the **old** build: `/music/` returned `200` but with the old home page's title, which is the fallback, not the page. Compare the served title, never the status code.
- [ ] 4.2 If the site has not redeployed since PR #484, deploy it first — the `site-deploy` workflow runs on a `site-vX.Y.Z` tag (release-please) or a manual dispatch. Pointing a store field at a path that silently serves the hub is no better than pointing it at the hub.
- [ ] 4.3 Check `https://cymbra.app/support/` and `/en/support/` still render support content and were untouched by the restructure.

## 5. Edit the App Store Connect listing

- [ ] 5.1 Set the Marketing URL to `https://cymbra.app/en/music/` for `en`, `it` and `es`, and `https://cymbra.app/music/` for `fr`, on **both** platforms — the field is platform × locale, so editing macOS does not cover iOS (see `app-store-metadata-scope`).
- [ ] 5.2 Leave the Support URL untouched on all four locales and both platforms; confirm it still reads `/support/` or `/en/support/` after saving.
- [ ] 5.3 Record whether the edit saved against the live version or opened a new version needing review, and say so in the change's follow-up notes — this has not been verified for this account.

## 6. Edit the Play Console listing

- [ ] 6.1 Set the store listing Website field to `https://cymbra.app/music/` for `fr` and `https://cymbra.app/en/music/` for `en`, `it` and `es`, in *Grow → Store presence → Main store listing*, per language.
- [ ] 6.2 Leave the privacy policy and data-deletion URLs untouched; confirm they still read `/confidentialite/` (fr), `/en/privacy/` and `/en/delete-account/`.
- [ ] 6.3 Submit the listing edit if the console requires a review pass, and note that the previous value still resolves meanwhile.

## 7. Verify the live listings

- [ ] 7.1 Fetch every pinned route against production and confirm each serves **its own page**, by comparing the returned `<title>` against the expected one. A status-code check proves nothing here: until task 3 lands, every path returns `200`.
- [ ] 7.2 Confirm the App Store listing's Website link resolves to the Music product page in each published locale.
- [ ] 7.3 Confirm the Play listing's Website link resolves to the Music product page in each published locale.
- [ ] 7.4 Confirm the support link on both listings still lands on the support page and not on a marketing surface.

## 8. Close out

- [ ] 8.1 Run `openspec validate pin-music-site-url-contract --strict`.
- [ ] 8.2 Open the follow-up for renaming `store-distribution` to `music-store-distribution` — deferred here on purpose (the rename is a folder move outside the delta mechanism, and it should settle the merge-or-split question with `music-macos-store-distribution` at the same time).
- [ ] 8.3 Open the follow-up for build-output enforcement: assert in the site's gate that every pinned route exists in `dist/` after `yarn build`, so a deletion fails the pull request instead of being found in production.
