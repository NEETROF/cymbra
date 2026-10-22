# Design — add-lingua-italian-interface

## Context

Change 33 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3: the extension's
interface in Italian, written before it is shown. I6 (settled 2026-10-23): « tu », never « Lei »;
« QCER »; Treccani's names for tenses and moods; Italian numbers (20.000; 96%). I2: a dedicated agent
reviews the Italian wording, the drafts landing in the pull request. Change 32 made Italian a native
language with the interface type kept to French, English and Spanish (`InterfaceLanguage =
Exclude<NativeLanguage, "it">`, an `interfaceOf` mapping `it` to English); change 45 shows the interface
and adds `it` to `INTERFACE_LANGUAGES`. The matrix's changes 13–17 built the catalogue (24 modules per
language, `typeof fr`, `Intl.PluralRules`, the parity test, the labels lint): this change adds a fourth
set to a structure that exists.

## Goals / Non-Goals

**Goals:**
- `src/i18n/it/`: the 23 text modules, typed after the French, in I6's Italian, reviewed by the Italian
  agent before merge.
- The format helpers and the lints know Italian; the interface alias restored; nothing shown.

**Non-Goals:**
- `it/grammar.ts`, the renderer of readings and the tense names for an Italian reader (change 34); until
  then an Italian interface would render readings with the English renderer, M13's fallback — unreachable
  before change 45.
- `INTERFACE_LANGUAGES` gaining `it`, the account's locale (M12, I10) and showing the interface (45);
  `_locales/it` (39); the Apple host (40); the site's `/it/` pages (41).

## Measured

On `main` at `152c3d547`, with a simulated Italian set (a copy of the English or Spanish one):

- **The catalogue**: 24 modules per language (French 1,299 lines, 8,916 words; English 6,657; Spanish
  7,416), each typed `typeof fr`; the text actually translated is 3,772 French words in 911 strings
  (3,631 in 803 without `grammar.ts`), of which 18 counted messages and 104 other messages built from
  parts.
- **Intl for Italian** (Node 22, ICU 78.2): plurals one / many / other (0 is `other`, unlike French; round
  millions `many`, as French and Spanish); 20000 → « 20.000 », 1234567 → « 1.234.567 », but 5000 →
  « 5000 » (no grouping under five digits); a short date « 04/10/2026 », a long one « 4 ottobre 2026 », a
  time « 14:05 »; Intl's percent style « 96% ».
- **A latent bug**: `formatPercent` has no Italian branch, so `it` falls into the French one, and the
  callers that ask for the spaced form (the reader's text size) would write « 110 % »; the compiler does
  not see it.
- **Elision and contraction**: the one contraction site is the note on levels borrowed from another
  language (`borrowedTypicalNote`: « dall'inglese / dallo spagnolo / dal francese / dall'italiano »),
  the precedent Spanish's `de()`; two messages put a date after an article (« il 4 » but « l'8 »,
  « l'11 »), two an hour (`card.onlineLimit`, `review.onlineLimit`: « all'1:05 »), and five a count after a
  plural article (« les ${n} mots »: `selection.fairlyCommon`, `selection.common`, `selection.veryCommon`,
  `settings.knowCommonest`, `stats.ofDictionary`).
- **Gender**: nine French messages make the reader masculine (« Connecté », « tu es connecté »), which
  Italian must word without a gender, as Spanish did, and « Bienvenue » (genderless in French) becomes
  « Benvenuto » in Italian; `READ_AS_NOUN` needs « Parola spagnola » (feminine); the review's ordinal
  takes « º »/« ª ».
- **Restoring the alias**: 23 sites in 18 files gain an Italian entry (the account's copy, the language
  labels' modules, the app, `LOCALES` « it-IT », the onboarding, the popup, the reader, the native-language
  view, the reading and settings copies, the review, the side panel, the badge, the ladder);
  `PRIVACY_POLICY_URLS` and `LEAVING_PAGES` need Italian pages the site does not have. Wired, `src`
  type-checks and the suite passes (4,347 tests on `main`; the Italian cases of D7 add to them).
- **Bundle cost before anything shows**: ≈ +127 to +143 kB across entries (content +36 kB, reader +38,
  side panel +27, popup +21), estimated from the English and Spanish sets.
- **Lints**: the labels lint's names miss Italian's (« spagnolo/a/i/e », « inglesi », « francesi »,
  « italiano/a/i/e »); the parity test and the settings-hosts lint loop over fr, en and es.

## Decisions

### D1 — 23 text modules, from the French

`src/i18n/it/` holds the 23 text modules, each `typeof fr`, translated from the French source — the
typed source the parity test compares with — using the Spanish draft's choices already made in a
Romance language (gender-free address, the colon for « — », the `many` forms, typography) and the
English to check the meaning; Music's Italian catalogue (`apps/music/lib/l10n/app_it.arb`, « tu »)
gives the product's terms (Impostazioni, Accedi, Esci, Annulla, Riprova, Elimina account). French
typography is undone: « » kept, as Italian writes them, without the inner no-break spaces; no space
before « : ; ? ! ». The Windows and macOS menu paths of the voice help are written from a cited source or
marked unverified, since no agent can see them. `languages` carries Italian's names with
their articles (« l'inglese », « lo spagnolo », « il francese », « l'italiano »), the masculine and
feminine forms where they differ, and `levelScale: "QCER"`. The README gains an Italian section.

### D2 — `grammar.ts` stays change 34's

Row 34 owns the Italian renderer of readings. `reading-copy.ts` maps an Italian interface to the English
renderer until then (M13's fallback); the parity test expects every surface but `grammar` for Italian.

### D3 — The format helpers

`LOCALES.it = "it-IT"`; `formatNumber` and `formatCount` use Intl's Italian (« 20.000 »); under five
digits, « 5000 » or « 5.000 » as the owner answers Open Question 1 — « 5000 » pinned with
`useGrouping: "min2"` (`"ES2023.Intl"` joining the extension's `tsconfig.json` `lib`, whose ES2022 types
accept a boolean alone), so that no browser's ICU groups it, « 5.000 » with grouping always; `formatPercent`
gains an Italian branch writing « 96% », closing the latent bug, and becomes an exhaustive switch so the
compiler flags the next language; plurals through `Intl.PluralRules("it")`, the counted messages taking
`many` where Spanish does.

### D4 — Elision, contraction and gender

A private `da(the)` helper in `it/languages.ts` for the borrowed-levels note, as Spanish's `de()`; the two
dates worded without an article (« Ultima sincronizzazione: 04/10/2026 »); the two hours after « ore »,
whose article agrees with it and never elides (« di nuovo disponibile alle ore 13:05 »); the five counts
after the feminine plural article, which never elides (« le ${n} parole »); the nine masculine messages
worded without a gender, « Bienvenue » as « Ti diamo il benvenuto »; `READ_AS_NOUN.it = "feminine"`; the
ordinal « º »/« ª ». These are the reviewer's list too (D8).

### D5 — The alias restored, change 32's mapping dropped

`InterfaceLanguage = NativeLanguage` again; change 32's `interfaceOf` and the test that held it go; the 23
sites gain their Italian entries. `PRIVACY_POLICY_URLS.it` and `LEAVING_PAGES.it` point at the English
pages (`/en/privacy/`, `/en/lingua/goodbye/`), already pinned; `deleteAccountUrl` already sends `it` to
the English page. Change 45, after change 41 has created the Italian pages — the leaving page pinned before
the extension build that compiles it into `LEAVING_PAGES.it` ships —, moves the three links to `/it/`.

### D6 — Inert until change 45

`INTERFACE_LANGUAGES` stays French, English and Spanish: a stored `it` reads as French, the guards stay
closed, the account locale (M12) unchanged, the capture tool unchanged. Dropping `interfaceOf` changes
change 32's unreachable fallback from English (M13) to French (M22). From this change on, a build listing
an Italian-glossed pair without `it` in `INTERFACE_LANGUAGES` would show French: a test holds every native
of the shipped pairs among the interface languages, so change 45 cannot list one without the other, and a
test holds a stored `it` reading French.

### D7 — The lints and the tests

The labels lint learns Italian's names (« spagnol[oaie] », « ingles[ei] », « frances[ei] »,
« italian[oaie] »); the settings-hosts lint and the parity test loop over Italian too, with
`SAME_AS_FRENCH.it` (« Email ») and `SAME_SHAPE_AS_FRENCH.it`, a test's Italian copy named `itCopy`, not
`it`, which vitest's `it` would shadow; the helpers' cases (plurals for 1, 3 and a
million, « 20.000 », four digits per Open Question 1, « 96% », the dates, `regionName`); an explicit Italian case in
`language-labels.spec.ts`, whose loops over `INTERFACE_LANGUAGES` would not reach Italian; the URL cases.
The compiler forces six tests' tables to gain Italian. No snapshot is per interface language.

### D8 — The Italian reviewer (I2)

`.claude/agents/lingua-italian-reviewer.md` defines the reviewer, which no change has yet (change 15
already relies on it): read-only tools; its criteria I6, Treccani and the Accademia della Crusca for usage,
Music's Italian terms and D4's list; a fixed report — per module, each finding with its line, the rule it
breaks and the correction. The drafts are committed in the pull request; the agent reads every module and
the README section; its findings are applied in the same pull request and its report pasted in the pull
request's description, before merge. Row 44 (`review-lingua-italian-wording`) is its later pass across
every surface — interface, card, listings, site — once they all exist; this change's pass covers its own
drafts. The owner's reading on the five targets is possible from change 45, or stays unverified (I2).

## Risks / Trade-offs

- **≈ 130–140 kB of Italian in every bundle before it shows** → the matrix's change 13 shipped its drafts
  the same way; the gain is that change 45 only switches.
- **Italian written by an agent** → I2's reviewer before merge; the owner's reading at change 45.
- **An Italian-glossed pair listed without `it` in the interface languages** → D6: change 45 does both.

## Migration Plan

Nothing to migrate; nothing shows. Released with the extension's next release.

## Effort

4–7 ideal days: the 23 modules (≈ 3,600–3,900 words) 2–3.5; the helpers, the alias and the 23 sites
0.75–1.25; the lints and the tests 0.75–1.25; the reviewer's pass and its fixes 0.5–1.

## Open Questions

For the owner:
1. **Four-digit numbers**: « 5000 » (Intl's default, recommended, pinned with `useGrouping: "min2"`) or
   « 5.000 » (grouping always).
