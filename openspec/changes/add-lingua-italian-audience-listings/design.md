# Design — add-lingua-italian-audience-listings

## Context

Measured on main at `6ef0bad0c`. "ext" is `apps/lingua-extension/STORE-LISTING.md`; "apple" is
`apps/lingua-apple/STORE-LISTING.md`.

- **The precedent.** Matrix change 37 (`add-lingua-spanish-audience-listings`, #806) changed the two
  listing files and no code:
  - the Spanish description for the Chrome Web Store and AMO;
  - the summary, quoted from `_locales/es`;
  - the Spanish labels beside the English and French in the single English fields;
  - the App Store es-ES texts (es-MX a word-for-word copy);
  - the claims table (each claim → the file that makes it true);
  - the dashboards' procedure for `es` and its results table.

  Its tasks 3.1 and 3.2 (the owner's review, the first localised submission) are unticked, and its
  results tables still read « to record ». In practice the Spanish was reviewed by AI on the owner's
  instruction (#913).
- **How listings reach the stores**: by hand. Nothing in CI submits a listing (ext's header). The release
  workflows upload packages and binaries only, and a tag publishes nothing. I9's « pushed from the
  repository » therefore means pasted from these files.
- **The fields' room today**, against their caps:

  | Field | Today | Cap |
  |---|---|---|
  | Chrome Web Store / AMO description (Spanish) | 2,940–3,416 | 16,000 |
  | Test instructions (one field, both stores) | 988 after change 26 | 1,000 |
  | Review notes (App Store, one English field) | 3,994 after change 26 | 4,000 |
  | App Store keywords | | 100 bytes |
  | App Store subtitle | | 30 |
  | App Store promotional text | | 170 |
  | App Store description | | 4,000 |
- **What is true at change 45**, en-it alone:
  - English words highlighted, the page's percentage and the card's buttons in Italian (change 33);
  - glosses from the Italian Wiktionary's English section, then the tables (change 36);
  - the card's tense names (« passato semplice di go », « forma in -ing », change 34);
  - capture and the review panel named as `_locales/it` names them (change 39);
  - levels under « QCER », English's not estimated;
  - account deletion at `/it/elimina-account/` (change 45);
  - the activation steps the Italian page gives (change 40);
  - whether extended translation is offered (change 38).

## Goals / Non-Goals

**Goals:**
- An Italian visitor reads, in every store, an Italian listing that is true of what change 45 ships.
- The shared English fields stay within their caps.

**Non-Goals:**
- Submitting anything: the owner pastes and submits (I12).
- The French, English and Spanish texts: they say nothing of Italian speakers, as change 37 left the
  others unchanged.
- The capture harness's Italian mode (change 45, D4).

## Decisions

### D1 — The Italian texts

The Italian texts are written for Italian speakers studying English, in « tu », and modelled on the
Spanish text without fr-es (one studied language):

| Store | Field | Size |
|---|---|---|
| Chrome Web Store and AMO | description | about 2,900–3,200 characters, about 480 words |
| App Store it-IT | name | `Cymbra Lingua`, the brand, kept (« lingua » is an ordinary Italian word, so the name already indexes it) |
| | subtitle | a draft such as « Impara l'inglese leggendo » (25 of 30) |
| | promotional text | about 135 of 170, re-pasted with every version |
| | keywords | about 90 of 100 bytes |
| | description | about 3,000 of 4,000 |
| | What's New | for change 45's release |

The constants:
- the homepage `/it/lingua/`;
- the privacy policy `/it/privacy/`, which is a courtesy translation, the French prevailing (I13);
- the support link, the owner's (task 3.2): `/it/assistenza/`, Music's support page in Italian, or
  `/en/support/`.

The precedent's Support URL for Spanish is still unrecorded, and the owner records it with the Italian
one. Characters App Store Connect refuses (« ᵉ ») are avoided.

Each claim gets a row in ext's claims table with the file that makes it true at change 45, as change 37's
did. While change 38 has not decided en-it's translation, both variants are drafted, offered and
withheld, as the precedent did.

### D2 — The shared English fields

These fields are one per item, in English:
- **Single purpose**: names English for Italian speakers, about +30 characters.
- **Permission justifications**: the Italian labels beside the others, about +150.
- **Remote code**: names en-it's direct model for an Italian speaker reading English, about +100.
- **Test instructions**: the Italian path costs about +136 characters against 12 left. The field is
  restructured rather than trimmed: the per-language labels give way to one sentence per step naming
  where the interface's language is chosen, and the counts are recorded in the heading, as the file's
  rules require.
- **Review notes**: the Italian audience, the interface list and a device in Italian cost about +165
  against 6 left. They are restructured the same way.

### D3 — The summary, quoted

The Chrome Web Store and AMO summary is `_locales/it`'s description, « Leggi il web in inglese: parole
sconosciute evidenziate, percentuale reale. Offline e senza tracciamento. » (105 of 112 characters,
change 39), quoted in ext and never drafted here. The version check already fails a summary that names a
language Italian readers cannot study (`summaryProblems`), so changes 50 and 54 rewrite it, not this
change.

### D4 — Screenshots

- **The extension's Italian set** (4 or 5 shots) comes from change 45's capture run, which also writes
  the site's media: one `yarn capture` writes both. Change 45 gives the harness its one-language mode,
  since an Italian reader studies one language and the harness's « second » shot assumes two.
- **The App Store it-IT set**: about four shots per slot (iPhone 6.5", iPad 13", Mac): reading, the card
  on « went » with « passato semplice di go », « Ripasso », and the statistics under QCER. It is taken by
  hand in Safari from change 45's build, on a device in Italian, as the precedent's were. The host app
  declares Italian only from that build (change 40). The owner takes them following the list (task
  3.2); I2 has the reviewer agent check each shot's text.

This change lists both sets and where each is uploaded.

### D5 — The dashboards' procedure for Italian

A section « The dashboards' languages: Italian (change 45's first submission) » repeats the Spanish
steps:
1. upload change 45's package without publishing;
2. fill the Italian description;
3. submit;
4. run AMO's dispatch with the Italian description, and check AMO's Italian summary;
5. record each result in its table.

The precedent's tables are still « to record »; task 3.3 asks the owner to record them, so that the
Italian procedure builds on facts.

### D6 — Reviewed by the Italian reviewer agent

Every Italian text is reviewed in this change's pull request by change 33's agent (I2). The programme's
row 44 runs before row 46, so its pass does not cover this change. The points:
- « tu » and imperatives (« Scegli », « Apri », « Attiva », « Tocca »);
- no gendered participle;
- the interface's terms (the card's buttons, « Ripasso », « Impostazioni », QCER);
- `_locales/it`'s words (« percentuale reale », « senza tracciamento », « riquadro laterale »);
- change 34's tense names and change 40's activation steps;
- the brand « Cymbra Lingua » against the noun « lingua »;
- numbers (« 25,4 MB », « 20.000 », « 96% ») and Italian typography;
- privacy wording that claims nothing beyond the French policy (I13), « senza tracciamento » checked
  against the anonymous counts and the online-translation providers;
- the iOS settings path (« Impostazioni › App › Safari › Estensioni »), cited or marked unverified;
- no beta and no price.

### D7 — Timing

The owner pastes from main, so this change is on main before change 45's packages are submitted. It
ships no code, so it need not precede change 45's pull request, as changes 21 and 26 had to precede 20
and 25. It is written against change 45's branch, as change 37 was written for change 35 (#806 merged
before #814).

The requirement's archive order: change 31, which lists Italian for Spanish speakers and modifies the
same requirement, archives after this change when it is proposed, or this change's block is re-copied
over it.

## Risks / Trade-offs

- **Italian written without a native reader, in a public listing** → the reviewer agent (I2), the claims
  table, and change 55 for corrections.
- **A field over its cap** → D2's restructure, and the counts recorded in each heading.
- **An Italian summary over a foreign description** → D7's order and D5's procedure, upload without
  publishing before the description is filled.

## Migration Plan

Nothing deploys. The owner pastes the texts with change 45's submission. Rollback is a revert of the
listing files.

## Effort

2.25–4 ideal days, against the programme's 2–5.5:

| Part | Ideal days |
|---|---|
| The Italian texts, with the variant that withholds translation | 0.75–1.25 |
| The claims table | 0.25–0.5 |
| The shared English fields at their caps | 0.5–1 |
| The screenshot lists, the procedure and its table | 0.25–0.5 |
| The reviewer agent's pass | 0.25–0.5 |
| The spec delta, the programme row, the recount | 0.25 |
