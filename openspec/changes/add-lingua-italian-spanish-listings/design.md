# Design — add-lingua-italian-spanish-listings

## Context

Change 31 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2. Change 30
(`enable-lingua-italian-for-spanish-speakers`) lists it-es for Spanish speakers, through English (I4,
I11). This change makes what a Spanish reader is told about Cymbra Lingua know it. Change 26
(`add-lingua-italian-english-listings`) did the same for English readers of Italian, and
`add-lingua-french-listings` for Spanish readers of French.

Changes 21, 26 and 46 are not implemented yet; their drafts are this change's base. Where things stand:

- **The listing files**, `apps/lingua-extension/STORE-LISTING.md` and `apps/lingua-apple/STORE-LISTING.md`,
  each count by their own stated rules.
- **The site.** Change 21 gives it Italian's names in the Spanish table and `STUDIED.it`. The Spanish
  home card and the Lingua pages are built from the shipped pairs, so it-es reaches them through change
  30's coverage file with no code change.
- **Change 30** commits the Spanish summary and the host app's Spanish activation page.
- **No French or English description says what Spanish speakers study.** That is true on `main` and
  after changes 21 and 26: the Chrome Web Store French text names no other audience; its English text
  and the App Store's fr-FR and en-US last lines name French and English speakers only. When fr-es was
  listed, `add-lingua-french-listings` added no such line. Change 46's Italian texts are the only other
  descriptions that will (D3). The English single purpose and review notes do say it (D4).

## Goals / Non-Goals

**Goals:**
- Every Spanish text of the listings names Italian.
- Change 46's Italian line on what Spanish speakers study names Italian.
- The fields one per store follow, within their limits.
- The requirements that the Spanish listings, the Spanish home card and a nine- or ten-row table would
  make false are modified here, implemented before change 30 lists it-es.

**Non-Goals:**
- The Spanish summary and the activation page (change 30).
- A new line in the French or English texts about Spanish speakers: their precedent has none.
- The Italian interface and its listings (changes 45 and 46).

## Measured

Counted on `main` at `ef1fe963b` by each file's own rules, with changes 21's and 26's drafts applied
first. Every count either records comes out exact: Chrome Web Store Spanish 3,436, App Store es 3,447,
test instructions 988, review notes 3,994, single purpose 611.

| Text | Base | With Italian for Spanish speakers | Limit |
|---|---|---|---|
| Chrome Web Store and AMO, Spanish description | 3,436 | 3,563 | 16,000 |
| App Store es-ES / es-MX subtitle | 30 | 26 | 30 |
| App Store es-ES / es-MX promotional text | 146 | 159 | 170 |
| App Store es-ES / es-MX keywords | 94 bytes | 94 bytes (one word out) | 100 bytes |
| App Store es-ES / es-MX description | 3,447 | 3,576 | 4,000 |
| App Store es-ES / es-MX « What's New » paragraph (new) | — | 485 (430 without the voice) | 4,000 |
| Change 46's Italian text, Chrome Web Store and AMO, last paragraph (a draft by analogy, D3) | 242 | 254 | 16,000 in all |
| Change 46's Italian text, App Store it-IT, last line (a draft by analogy, D3) | 304 | 316 | 4,000 in all |
| App Store review notes | 4,000 after change 46 (3,994 before) | 3,999 (3,993) | 4,000 |
| CWS single purpose | 611 (641 after change 46) | 620 (650) | — |
| Test instructions (CWS, AMO) | 988 (996 after change 46) | unchanged | 1,000 |

- **Too long said directly:**
  - the keywords with `italiano` added: 103 bytes;
  - the review notes' credits naming each edition in full: +71;
  - the subtitle keeping « mientras lees »: `Inglés, francés o italiano mientras lees` is 40.
- **The sizes** depend on it-en's served size, which change 2 puts in the catalogue. en-es is
  25,373,354 B and fr-en 26,234,715 B, so French through English is 51,608,069 B, « 51,6 MB », whatever
  it-en is.
  - « 51,6 MB con el francés o el italiano » holds if it-en is under 26,276,646 B. Otherwise the clause
    splits: « 51,6 MB con el francés, 51,7 MB con el italiano ».
  - « 77,8 MB con los dos » holds if it-en is under 26,241,931 B, else « 77,9 MB », recomputed on its own.
- **The site**, simulated on a scratch copy with change 21's tables and an it-es row:
  - the build passes (35 pages) and `yarn check:routes` gives 59 passed and 2 skipped (the byte-pinned
    fixtures, until change 30 refreshes them);
  - only `/es/`, `/es/lingua/`, `/lingua/` and `/en/lingua/` move:
    - the Spanish home card « Lee la web en inglés, en francés o en italiano… »;
    - the Spanish page « Pensada para hispanohablantes que aprenden inglés, francés o italiano », a row
      « Italiano → español », and the order of its tense-and-gender and levels sentences;
    - the French and English pages gain it-es's row, and their hero notes (« Aussi pour les
      hispanophones… », "Also for Spanish speakers…") and 🧭 titles name Italian.
  - All of it is data-driven, and change 30's fixtures refresh pins it.
- **The table with nine rows**, in headless Chromium on the built pages: at 375 px the page is 375 px
  and the table 335 px in its 335 px box on all three pages; at 320 px it scrolls inside its box. A
  tenth row (en-it, if change 45 lists it first) adds height, not width.

## Decisions

### D1 — The site: nothing to write, one test, the scenarios true

The Spanish home card and the Lingua pages name Italian from the coverage file change 30 writes; no
code moves. Two tests are added, each feeding nine pairs with change 2's routes:
- `test/lingua-text.spec.ts` asserts the Spanish home card, the Spanish page's audience and its row;
- `test/astro/spanish-pages.spec.ts` asserts that en-es, fr-es and it-es render « Lee la web en inglés,
  en francés o en italiano… », with no audience and the button to `/es/lingua`, as it pins the
  English-only card today.

`apps/site/README.md`'s home-card examples follow. The requirements that would read false once it-es is
listed are modified here (D9). If the owner names Wikidata (D6), the `glossesOrigin` sentences of
`src/lib/lingua-text.ts` follow in French, English and Spanish (`:184`, `:298`, `:411`). Every Lingua
page shows it-es's row under its own sentence. Their pinned tests (`test/lingua-text.spec.ts:197`,
`:314`) follow too: the only strings this change would write.

### D2 — The Spanish texts name Italian

Drafts (M9: the owner's words).

**Chrome Web Store and AMO, Spanish** (3,436 → 3,563):
- « …las palabras en inglés, en francés o en italiano que todavía no conoces, sin cambiar el diseño de
  la página. »
- « La tarjeta también nombra el tiempo verbal, como se aprende en clase («pasado simple de go»; en
  francés y en italiano, «pretérito perfecto simple de indicativo»), y el género de los sustantivos
  franceses e italianos. » (change 18's Spanish name)
- « …; para el francés y el italiano, niveles estimados según la frecuencia de las palabras, a falta de
  una lista MCER de uso libre. »
- « …directamente desde el inglés y, desde el francés y el italiano, pasando por el inglés, en tu
  dispositivo, con el motor de Firefox Translations. »
- « …(25,4 MB para el inglés, 51,6 MB con el francés o el italiano, 77,8 MB con los dos); »
- « Cuántas palabras inglesas, francesas e italianas explica nuestro diccionario en español: nuestras
  cifras están publicadas en cymbra.app/es/lingua. Para el francés y el italiano, el diccionario es algo
  menos completo que para el inglés. »
- The last line stays as changes 21 and 26 leave it. « Elige en los Ajustes los idiomas que estudias… »
  does not move.
- The Spanish texts kept for a release without fr-es are history and are not edited, as change 26's D3
  leaves them.

**App Store es-ES and es-MX:**
- **Subtitle**: `Inglés, francés o italiano` (26), as change 21's fr-FR `Anglais, espagnol ou italien`
  and change 26's `Spanish, French or Italian`.
- **Promotional text** (159): « Las palabras en inglés, en francés o en italiano que todavía no conoces,
  resaltadas en la página que lees. El análisis se hace en tu dispositivo, sin conexión. »
- **Keywords** (94 bytes): `inglés,francés,italiano,vocabulario,lectura,traducción,extensión,MCER,tarjetas,diccionario`.
  `aprender`, the most general, gives way, as `words` did in change 26 (Open Question 2).
- **Description** (3,447 → 3,576), the same sentences as the Chrome text, adjusted to this text's own
  wording:
  - the first sentence;
  - step 3: « Para el francés o el italiano, márcalo en los Ajustes de la extensión, pestaña Idioma, y
    luego elige su nivel. » (change 30's activation page);
  - the card, the levels, the translation and the sizes;
  - « Para el francés y el italiano, nuestro diccionario en español es algo menos completo que para el
    inglés: … »;
  - the last line « La interfaz está en español para los hispanohablantes, que aprenden inglés, francés
    e italiano; … », the rest as changes 21 and 26 leave it.
- **« What's New »** for change 30's release, modelled on fr-es's Spanish paragraph (485): « Cymbra
  Lingua ahora también lee italiano, para los hispanohablantes: márcalo en Ajustes › Idioma. Las
  palabras italianas llegan con su significado en español; la tarjeta nombra el tiempo verbal y el
  género («pretérito perfecto simple de indicativo»), y tu nivel de italiano se estima según la
  frecuencia de las palabras. La lectura en voz alta lo lee con una voz de Italia, y la traducción
  ampliada traduce tu selección en italiano al español, pasando por el inglés, en tu dispositivo. »
  - Without the voice clause, « La traducción ampliada… » begins the last sentence (430).
  - fr-es's « , escrito por personas » is left out pending D6 (+22 if kept).

### D3 — Change 46's Italian line on what Spanish speakers study

Change 46's texts are modelled on the Spanish text without fr-es (its D1), so their last lines say
what other readers study. Its design quotes no sentence; these are the lines by that analogy, the I2
reviewer agent's to read:
- Chrome Web Store and AMO, Italian (242 → 254): « …; e l'inglese, il francese e l'italiano agli
  ispanofoni, in spagnolo. »
- App Store it-IT (304 → 316): « …e in spagnolo per gli ispanofoni, che imparano l'inglese, il francese
  e l'italiano. »

The line with Italian is pasted with change 30's release, never before (D8). Whichever of this change
and change 46 is implemented second keeps both forms in the listing files, labelled:
- without it-es, for change 45's submission if en-it is released first;
- with it-es, for change 30's.

This is how `add-lingua-french-listings` kept its texts with and without fr-es. If change 46 is
implemented first, this pull request adds the second form, and the reviewer agent's pass (change 44's
procedure) runs on it. No French or English description moves: none says what Spanish speakers study
(*Context*).

### D4 — The fields one per store, within their limits

- **Test instructions**: unchanged (988, or 996 after change 46).
  - The reviewer's path runs in the English interface, where step 6's « "French" or "Italian" » does not
    move.
  - « "Español" to study English » names the way to the Spanish interface, not what Spanish speakers
    study; it stayed as is when fr-es was listed.
  - Four characters of room would hold nothing anyway.
- **App Store review notes** (3,999 / 4,000 after change 46; 3,993 before it):
  - audience « …; English, French and Spanish speakers Italian too. » (+9);
  - route « (25.4-26.3 MB each; a pair through English chains two) » (−16), which covers it-es as it-en
    then en-es;
  - credits « Wikidata Lexemes (CC0) and kaikki.org extracts of nine Wiktionaries (en, fr, es, it, de,
    pt, ru, pl, nl) for the glosses » (+28), as change 27a's sources. Wikidata is not a kaikki.org
    extract, so it comes first;
  - to make room, step 4's « the pill shows the share of the page you know » becomes « the pill shows
    the share you know » (−12), and « by reading web pages and their own DRM-free EPUB books » becomes
    « …web pages and DRM-free EPUB books » (−10).
  - Every step, label, service and source is kept. Spare cuts, if change 46 has used one of these:
    « for the glosses » (−16), « Wikidata Lexemes (CC0) » → « Wikidata (CC0) » (−8). The alternative,
    which compacts the audience to one sentence, is Open Question 4.
  - The block is labelled for change 30's submission. A submission before it (change 45's) pastes the
    block without it-es.
- **Single purpose** (620; 650 after change 46): « …Spanish, French or Italian for English speakers;
  English, French or Italian for Spanish speakers; English, Spanish or Italian for French speakers… ».
  The file's usual note: « pasted with change 30's release; until then, this file's version before
  add-lingua-italian-spanish-listings ».
- **Remote code**: « it-fr's, it-en's and it-es's among them », and « for a Spanish speaker reading
  French, fr-en then en-es, and reading Italian, it-en then en-es (`it-en/base-memory/2.0`,
  `en-es/base-memory/2.1`), both going through English ». If change 21 keeps the `packs.json`
  parenthetical, it gains « ; it-es once its tables stand at or above their floor ». The same note as
  the single purpose's: pasted with change 30's release.
- **Notes not pasted**, each saying what Spanish speakers study:
  - in the extension file:
    - the *Listing languages* paragraph, the Spanish description's header and the *Description*
      introduction;
    - the claims table's lead sentence, its « Who learns what » row, and its « English a Spanish
      reader's » row, which states that it-es is listed after en-es and fr-es (the card and the table
      scenarios rely on that order);
    - the permissions' introduction, and `## Summary` quoting change 30's Spanish summary;
  - in the App Store file:
    - the locales note, the fields table, and the es description's lead note;
    - the review notes' lead paragraphs, whose third-party sources gain the nine Wiktionaries and
      Wikidata;
    - *What the app itself says*, which quotes the Spanish step 3 word for word from change 30's
      `copy.js`;
  - the review-notes blocks labelled by submission: this one for change 30's.

### D5 — Figures the texts quote, taken when they are known

- **Sizes**: from change 2's catalogue, by the thresholds of *Measured*.
- **The dictionary sentence**: names Italian beside French if it-es's committed coverage is under
  en-es's at each top, as *Spanish speakers learning Italian* says. Change 27's designed figures (84.7 /
  76.1 / 65.5 loose, against en-es's 93.0 / 85.0 / 71.7) make that all but certain. The committed
  figures are on `main` before this change is implemented (D8), and they decide.
- **Translation** (M15), at change 29's figures and the owner's answer at change 30:
  - on the tier, or offered without a mark: the drafts as written;
  - not offered: every Italian clause about translation and its sizes goes. In « What's New » only « y
    la traducción ampliada… » goes, the read-aloud sentence kept. The site says « Pendiente: del
    italiano al español » from change 30's data. The review notes need no variant: « a pair through
    English chains two » holds either way. These variants are written beside the drafts in the listing
    files, as change 21's were.

### D6 — Where the definitions come from, and the claims table

The Spanish texts say « Los significados proceden del Wikcionario, escritos por personas, nunca por una
traducción automática ». The site says « Las definiciones proceden del Wikcionario y de traducciones
escritas por personas », and its French and English pages say the same in their language.

For it-es, change 27's spec reads its sources in this order (I11, change 27a):
1. the Spanish Wiktionary's Italian section's definitions;
2. the Italian Wiktionary's Spanish translations;
3. the Spanish Wiktionary's Italian translations read backwards;
4. Wikidata's lexeme translation links and Spanish glosses of Italian senses, then its lexemes of one
   item (CC0), which is not Wiktionary;
5. the Italian Wiktionary's Spanish section read backwards;
6. last, the pivot: an Italian and a Spanish word written under the same sense of one of seven other
   Wiktionaries' translation tables.

Every word was written by a person, and nothing is machine-translated, so « nunca por una traducción
automática » holds. Two claims do not hold as they stand:
- « del Wikcionario » alone leaves out Wikidata;
- in the pivot and in Wikidata's same-item lexemes, the pairing is done by the reducer, not by a person.
  Change 27a measured the errors it makes, such as `sparato` « Bragueta ».

The options (Open Question 5):
- **(a) Name Wikidata** (recommended): « Los significados proceden del Wikcionario y, para el italiano,
  también de Wikidata, escritos por personas… » (+41 each; 3,604 and 3,617). The site's three sentences
  follow (D1).
- **(b) Name Wikidata and disclose the pairing**: (a), plus « …o emparejados a través de las tablas de
  traducción de otros Wikcionarios ». The length is counted when chosen; the Chrome text has room, and
  the App Store description has about 380 characters left.
- **(c) Keep the sentence as it is.**

The extension file's « what each claim rests on » gains rows:
- Italian highlighted for Spanish speakers: `pairsOf("es")`, it-es in `packs.json`;
- the meaning and its sources: it-es's glosses, change 27's README and NOTICE;
- « pretérito perfecto simple de indicativo »: change 18's Spanish renderer;
- Italian's levels estimated: change 14;
- it-es through English, and its size: change 2's route and change 30's D5;
- the figures at cymbra.app/es/lingua: change 30;
- step 3: change 30's `copy.js`.

### D7 — Screenshots and dashboards

- **App Store es-ES**: one capture more, by hand, from change 30's build, on an Italian page with the
  card open on a passato remoto form. es-MX reuses the set.
- **Chrome Web Store Spanish set**: at its five-capture cap, unchanged (Open Question 6).
- **Dashboards**: no language is added. For change 30's submission:
  - Chrome Web Store and AMO: the `es` description replaced, the new summary checked, the single
    purpose and remote code replaced, AMO's Spanish pasted;
  - App Store: es-ES and es-MX's subtitle, promotional text, keywords, description, « What's New » and
    capture, and the review notes;
  - the Italian texts' line, if change 46 is live;
  - a table of what each dashboard shows.

### D8 — Order

- **Implemented** after changes 21 and 26, on whose drafts it builds; after change 27's committed
  tables, change 29's figures and the owner's answer under M15, which settle D5; and before change 30
  lists it-es, as change 26 is before change 25. Its site tests and scenarios are true once it-es is
  listed.
- **Pasted** with change 30's release, after the site deploy that publishes it-es's figures (I12, M18).
- **Publication, not implementation, decides.** I5 lets change 45 be released before change 30. So
  every it-es mention this change writes (the Spanish texts, the Italian line, the single purpose, the
  remote code, the review notes) is pasted with change 30's release and not before. A submission before
  it pastes the variants without it-es, which the files keep beside the new ones, labelled by submission
  (D3, D4).
- **Against change 46**: whichever is implemented second keeps both forms of the Italian line (D3) and
  sets the review notes' final count (D4).
- **Against change 50** (fr-it), which extends this change's listings and table blocks: this change
  archives after change 30 whatever order they are implemented in, so if change 50 is implemented before
  change 30 or before this change, change 50 archives first, and its pull request amends this change's
  delta to carry change 50's scenarios and name it in `archiveAfter`, implemented or not; otherwise change
  50 copies this change's blocks and archives after it (change 50's D13).
- **Archived** after the changes of `.openspec.yaml`: change 46, whose *A listing in Italian* this block
  carries, and change 45, whose table block this one extends. `scripts/openspec_archive_order.py`
  ignores a named change that has no directory, so change 30 is waited for once proposed.

### D9 — Requirements true of their time

- ***The store listings name each studied language*** (held by change 46):
  - *fr-es held below its floor* holds « and neither fr-es nor it-es »;
  - *Spanish speakers learning French* holds « and not it-es »;
  - in *French speakers learning Italian* and *English speakers learning Italian*, the clause that no
    Spanish text offers Italian holds « while it-es is not listed »;
  - *Spanish speakers learning Italian* is added;
  - change 46's *A listing in Italian* is kept: its Italian readers still study English alone.
- ***The coverage table reads at a phone's width*** (held by change 45): *Eight pairs* and *Nine pairs
  on a phone* already hold « and no other pair ». *Today's pairs*, false since six pairs shipped, holds
  « and no other pair », as change 26 narrowed *Six pairs*. *Nine pairs on a phone, it-es the ninth* and
  *Ten pairs on a phone, en-it and it-es* are added. Stage 4's pairs on the Italian page will need names
  of their own.
- ***The Spanish home page describes Lingua for the readers it serves*** (held by
  `extend-site-spanish-locale`): *Spanish speakers learning English* holds « and no other pair glossed
  in Spanish », since the card already reads « en inglés o en francés » once fr-es ships;
  *Spanish speakers learning Italian* is added, with the same clause.

Each MODIFIED block keeps every other word and scenario, re-copied at implementation.

True as written:
- *The Lingua page says what each studied language's card and levels are*: change 45's *Italian
  studied* already covers Italian on every page.
- *The stores' summary names what its readers can study*: change 30 commits the Spanish summary.

## Risks / Trade-offs

- **Texts written before their figures** → D5.
- **Tight fields** → the review notes at 3,999 of 4,000 after change 46; spare cuts listed (D4).
- **A provenance claim the pivot does not meet** → D6, the owner's call.
- **The pivot's glosses read wrong in places** → change 28 before the list, and change 30's dogfood.
- **The Chrome Web Store's Spanish screenshots show no Italian** → Open Question 6; the App Store's do.

## Migration Plan

Nothing to migrate. The site tests and the scenarios land before change 30; the listings are pasted by
the owner with change 30's release. Rollback is a revert.

## Effort

Against 1.5–3 ideal days: 1.35–2.75.

| Part | Ideal days |
|---|---|
| The Spanish texts, recounted | 0.5–0.75 |
| The fields one per store (the review notes' cuts) | 0.25–0.5 |
| The claims table and the provenance sentence | 0.25–0.5 |
| The site tests, the scenarios and the 375 px check | 0.25–0.5 |
| The Italian line and the reviewer agent's pass | 0–0.25 |
| The dashboards' procedure and the documents | 0.1–0.25 |

## Open Questions

For the owner:
1. **The drafts** (D2–D4): as written, or other words (M9).
2. **The keywords**: `aprender` gives way (recommended), or `tarjetas`, both at 94 bytes.
3. **« What's New »**: with the voice clause (recommended), as changes 21 and 26; or without.
4. **The review notes**: the cuts of D4 (recommended), or the audience compacted to « Readers study
   English, Spanish, French or Italian, other than their own. » (−32). That alternative's route reads
   « French to Spanish and Italian to French or Spanish chain two » (+11), keeps the credits (+28) and
   step 4's cut (−12), and does not cut the purpose: 3,995 after change 46, 3,989 before. Its audience
   sentence over-claims for Italian speakers, as the one change 46 accepted already does.
5. **Provenance** (D6): (a) name Wikidata in the Spanish texts and the site's three sentences
   (recommended); (b) also disclose the pairing through other Wiktionaries' tables; or (c) keep
   « Wikcionario … escritas por personas ».
6. **The Chrome Web Store's Spanish screenshots**: unchanged (recommended), or the second capture moved
   to an Italian page.
7. **The es-ES and es-MX Support URL**: the listing file does not record it. Which page it names, as
   change 46 asks for it-IT.
