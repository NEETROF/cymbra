# add-lingua-review-translation-fallback — a review card always has a way to an answer, and keeps the one the reader accepts

## Why

Dogfood of the Cymbra Lingua app on an iPhone (2026-10-14): reviewing the Safari deck, the English
cards « slacken » and « oppressive » answer « Pas de traduction pour ce mot. », and the expression
card « colloquially referred to as » answers « Pas de traduction pour cette expression. » — the
card cannot be reviewed. The two words are not in the EN→FR pack (24.8k lemmas); the expression is
a free selection, not a pack expression. The extension's review already falls back on the device's
« Traduction étendue » (`add-lingua-review-translation`), but the app mounts the same review page
without a translator, so it has no fallback at all; and nowhere does the review offer the online
translation the selection card has. Worse, a fallback answer is thrown away: the same card asks
again at every review, on every device.

A second dogfood finding the same day: the card « would » has a pack gloss, but a useless one
(« Subjonctif passé du verbe auxiliaire will ; S'utilise pour indiquer que l'on trouve une
personne … sexuellement attrayant »). A card with a gloss is never translated (D1 of
`add-lingua-review-translation`), so the reader has no way past a bad gloss. The pack's English
modals are fixed separately by `fix-lingua-en-fr-modal-glosses`; this change gives the reader a way
past any bad gloss.

## What Changes

- **The app's review translates on the device, like the extension's.** The Cymbra Lingua app
  (iPhone, iPad, Mac) gets the on-device extended translation in its review page: the same engine,
  run in a worker of the page, on its own copy of the models, downloaded once with the reader's
  consent when « Traduction étendue » is on in the Safari extension of that device.
- **The review offers « Traduction en ligne »** on a revealed card when the reader's account has
  access to online translation and the setting is ticked — the selection card's rule. Unlike the
  selection card, a single word is allowed: its sentence is sent with the word marked by position.
  In the extension (drawer, side panel, EPUB reader) and in the app.
- **A card with a gloss offers the translation on demand.** After the reveal, a small link
  (French UI copy: « Traduire la phrase ») asks the engine, and shows the online button under the
  same rule. A card without a gloss keeps today's automatic engine translation.
- **The reader can keep a machine translation as the card's gloss.** An explicit action (French UI
  copy: « Garder « se relâcher » ») saves the translation of the marked word only — never the
  sentence — as the card's gloss, in the reader's native language, marked as chosen by the reader.
  When the mark is missing, split oddly or too long, the action opens a short field to type or
  correct the word first. Once kept, the card is answered by it and nothing is asked again. Not
  kept, nothing is stored (today's behaviour).
- **A gloss the reader chose wins**: it is never replaced by the pack's — not by a re-capture of the
  word, not by a pack-driven rewrite — it travels in the backup and to the reader's other devices,
  and from the app back to the extension. The mark is `gloss_by_reader`, exactly as
  `add-lingua-card-edit` defines it; whichever change is implemented first introduces it.
- **Review precedence for every card**: the reader's gloss, else the current pack's gloss, else the
  gloss stored at capture — display only, the card never rewritten — so a pack fix (such as
  `fix-lingua-en-fr-modal-glosses`' « would ») reaches cards already in decks. Cards without a gloss
  are already routed through the pack by `fix-lingua-selection-card-punctuation`.
- **The app may hold its own Cymbra session** for online translation only: the reader signs in
  inside the app with the Apple or Google sheet it already runs. The extension's session is never
  relayed.
- **« A machine translation is never stored » becomes « … stored only when the reader keeps it »**,
  and the privacy annex (French, English, Spanish) says what is kept and that the app downloads
  models and may send a sentence on request.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-translation`: the never-stored requirement renamed and relaxed to explicit keeping; the
  review's translation gains the online button, the on-demand translation of a glossed card, and
  keeping.
- `lingua-decks-review`: added — a gloss the reader kept is the card's gloss, and wins; modified —
  the review's gloss precedence for every card, and the card schema's mark.
- `lingua-sync`: the card synchronisation carries whether the reader chose the gloss; the container
  app may hold its own session for online translation.
- `lingua-apple-app`: the app's review translates on the device and online; the network requests it
  may make.
- `lingua-privacy`: the disclosures cover a kept translation and the app.

No archive ordering is needed against archived changes. `add-lingua-card-sentence-translation`
(docs only, not implemented) modifies the same never-stored requirement and must archive after this
change (task 1.1). `add-lingua-card-edit` modifies three of the same requirements; whichever of the
two archives second rebases its deltas on the first's (task 1.2).

## Impact

**Products.**
- **Cymbra Lingua — new behaviour.** The core (`crates/lingua-core`) and its WASM build
  (`crates/lingua-wasm`); the extension (`apps/lingua-extension`) on Chromium, Firefox and Safari;
  the Apple container app (`apps/lingua-apple`) — its page (the `app` build of the extension), its
  native bridge and sign-in.
- **Cymbra Lingua backend — extended, not new.** `backend/lingua`: `CardOp` gains one additive
  pass-through field (`gloss_by_reader = 13`) and `lingua.cards` one column (migration 0008) —
  shared with `add-lingua-card-edit`, introduced by whichever ships first. The online translation RPC is reused
  unchanged. No new RPC.
- **Cymbra ID — consumed.** The app signs in through the existing `SignInOidc` against the `lingua`
  audience and refreshes like the extension; its page origin is added to
  `CYMBRA_ALLOWED_WEB_ORIGINS` (configuration). Nothing redeclared from `id-*` or `platform-*`; the
  online access flag is the existing `lingua.online_translation.enabled`.
- **Public site — content only.** `apps/site`: the Lingua annex of the privacy policy (French,
  English, Spanish).
- Music, Live and the back office are untouched. The agent plugin (`apps/lingua-agent`) reads the
  new card field's default and is otherwise untouched.

**Code.** `crates/lingua-core/src/decks/card.rs`, `decks/review.rs`; `crates/lingua-wasm/src/lib.rs`;
`backend/lingua/proto/deck.proto`, `src/deck.rs`, `src/pg_deck.rs`, a migration;
`apps/lingua-extension/src/review/*`, `src/translate/*`, `src/state/app-relay.ts`, `src/app/*`,
`src/sync/*`, `build.mjs`, the drawer and side panel hosts; `apps/lingua-apple/Shared (App)/*` and
`LinguaSignIn`; `apps/site/src/pages/confidentialite.md`, `en/privacy.md`, `es/privacidad.md`.

**Data.** Additive: no existing card changes; the server column defaults to `false`. Review views
of cards whose stored gloss differs from the current pack's change (display only); the pinned
baselines are re-pinned.

**CI.** `rust`, `lingua-extension-check`, `lingua-apple-build`, `proto`, `backend-it` and
`site-check` already watch these paths; no new unit.
