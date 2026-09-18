## Context

`apps/site` is a hub plus one page per product since PR #484. The restructure added
`/music`, `/lingua` and their `/en/` counterparts and rewrote what `/` serves; it
deleted nothing. That is the whole reason it was safe, and the reason is worth
writing down, because it was established by inspection rather than by a rule: the
route diff against `main` was 4 added, 2 modified, 0 deleted.

Two asymmetries drive this change.

**Store listing fields are read by humans who already chose the app.** Someone who
taps *Website* on the Play listing for Cymbra Music is not shopping for a product;
they are reading about one. The hub serves them a chooser. It is one extra tap, not
a bug — which is exactly why it will never fix itself if it is not written down.

**In-app URLs are read by binaries nobody can update.** A user who installed 1.30 in
September has a copy of `/cgu/` compiled into their app. If the site moves that path,
that user's Terms link dies and no amount of shipping fixes it for them. The set of
paths carrying this property is currently discoverable only by grepping two Dart
files and remembering what was typed into two consoles.

**The fallback is the real find.** The restructure was verified safe by diffing
routes, and that verification was sound — but it would have passed just as happily on
a change that deleted `/en/privacy/`, because production answers any unmatched path
with the French home page and `200`. Checked directly:
`GET /route-qui-nexiste-pas/` → `200`, title `Cymbra — pratique et cours de musique
connectés`. A dead privacy link would present French marketing copy to an English
reader, look healthy to any monitor, and only surface as an App Review rejection.

Constraints worth stating: the site ships **no** `_redirects` file today, so there is
no existing redirect mechanism to lean on; `store-distribution` already owns
"Store-listing copy" and versions the listing text under `apps/music/store/copy/`;
and `legal-links` is currently being modified by the in-flight
`open-app-without-sign-in-wall`.

## Goals / Non-Goals

**Goals:**

- Put the Music listing's marketing URL on the Music product page, per locale.
- Name the routes that shipped clients and store listings pin, and the consumer that
  pins each, somewhere a site author will actually look.
- Make "moving a pinned route requires a redirect in the same change" a rule rather
  than a thing the reviewer happens to notice.
- State in the spec why the support URL is not free to retarget, so the guideline 1.5
  rejection is not re-earned by someone who never saw it.
- Make a missing page fail visibly. The site has no `404` page, so the host answers
  every unmatched path with the French home page and `200` — which means the exact
  failure this capability exists to prevent is the one failure nobody would see.

**Non-Goals:**

- Changing any URL inside `apps/music`. The five hard-coded values are correct. This
  change constrains them, it does not touch them.
- Shipping live redirects. Nothing is moving. `public/_redirects` is specified as the
  mechanism for a future move, not created with entries now.
- Automated enforcement of the route list. See the decision below. (The `404` page is
  in scope; a CI gate asserting each pinned route survives a build is not.)
- Renaming the `store-distribution` capability. See Open Questions.
- Lingua's own listing URLs. The extension is unpublished; it has no listing fields
  to pin.

## Decisions

**The route contract lives in `apps/site/README.md`, not in the Flutter app.**
The obligation binds the site — it is the side that can break it. `apps/music` is the
consumer, and a consumer listing its own dependencies is a list nobody consults at
the moment of danger. The site README already documents the page structure added by
PR #484, so the contract sits directly under the thing it constrains. A pointer from
`apps/music/store/README.md` covers the operator coming from the store side.

*Alternative considered:* a top-level `docs/` file. Rejected — one more place to go
stale, and neither the site author nor the store operator has a reason to open it.

**Listing URL fields go in `apps/music/store/copy/<locale>.md`, next to the copy.**
Those files are already the checked-in mirror of what gets pasted into the consoles,
already per-locale, and already carry the privacy URL in the subscription block. The
marketing and support URLs are the same kind of artefact and belong in the same file.

*Alternative considered:* a single table in `apps/music/store/README.md`. Rejected —
it splits per-locale listing values across two files, and the copy files are what an
operator has open while filling a console form.

**A `404` page ships here; a CI gate does not.** The not-found page is three files
and removes the silent-failure mode outright, so it belongs in the change that
discovered it. A CI check that fetched every pinned route after deploy is buildable,
but it would test the deployed site rather than the pull request, and a broken route
would be found after it shipped. A build-time check that
the expected `dist/**/index.html` files exist would catch a deletion in CI — cheap
and worth doing, but it is a separate change with its own testing story, and adding
it here would widen a documentation change into a tooling one.

*Alternative considered:* asserting the route list in `apps/site/test/`. Tempting,
since `stores.ts` already has a vitest file. Rejected for this change and recorded as
the natural follow-up: the test would need the build output, not the source, so it is
a different gate from the ones vitest currently runs.

**`ADDED` rather than `MODIFIED` on `store-distribution`.** The existing
"Store-listing copy" requirement covers description, subtitle, keywords and category.
URLs are a new concern in the same capability, not a change to what that requirement
already says. Per the delta guidance, adding a concern without changing existing
behaviour is `ADDED`; a `MODIFIED` here would risk losing detail at archive time for
no gain.

**`site-client-route-contract` is a new capability, not an extension of
`legal-links`.** `legal-links` is a Music capability about resolving and opening two
URLs from the app. The obligation being specified is the site's, covers routes that
have nothing to do with legal pages (`/checkout/done`, `/redeem`, `/account`), and
outlives any particular consumer. Putting it in `legal-links` would also collide with
`open-app-without-sign-in-wall`, which is already editing that file.

## Risks / Trade-offs

**The marketing URL edit is invisible from the repository.** → It is a console value;
nothing in CI can observe it. Mitigated by recording the expected value per locale in
the copy files, so the next audit is a comparison rather than a recollection, and by a
task that verifies each store's live field after the edit.

**A documented rule is still only a documented rule.** → A future change can delete
`/support/` and pass every gate in this repo. Accepted for now; the build-output
assertion described above is the follow-up that turns the rule into a gate. The `404`
page at least makes the breakage visible to whoever hits it.

**Turning off the host's fallback may not be a repository change.** → If Cloudflare
Pages is serving the home page because the project is in single-page-application
mode, adding `404.astro` alone will not fix it and a project setting has to change.
The tasks check the deployed behaviour rather than assuming the build output is
enough.

**The list of pinned routes goes stale the moment a new consumer appears.** → The
third requirement puts the update obligation on the change that creates the new
consumer, which is the only moment anyone knows the pin exists.

**Apple may treat the marketing URL edit as a metadata change needing review.** →
Marketing and support URLs are ordinarily editable on a live version without a new
submission, but this has not been verified for this account. The task list checks the
field's state after saving rather than assuming, and the edit is trivially revertible.

## Migration Plan

Nothing deploys and nothing rolls back in code. The sequence is: record the values in
the repo, then edit the two consoles, then verify each live listing field. If a store
rejects or queues the edit, the previous value (`https://cymbra.app`) still resolves
and still describes Cymbra — the failure mode is the status quo, not a broken link.

## Open Questions

- **Should `store-distribution` be renamed to `music-store-distribution` here?**
  `openspec/config.yaml` says legacy unprefixed capabilities are renamed "seulement
  quand un change les touche déjà", and this change touches it. Against doing it now:
  it widens a metadata change into a rename that also invites reconciling the sibling
  `music-macos-store-distribution`, and the rename is orthogonal to the URLs. Left for
  the reviewer to call; the delta as written does not depend on the answer.
- **Is a Lingua marketing URL needed at publication time?** `/lingua` exists and is
  ready to be a store listing's website field, but the extension is unpublished, so
  there is no field to fill. Worth revisiting in the change that publishes it.
