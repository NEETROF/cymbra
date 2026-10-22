# Design — add-lingua-italian-english-listings

## Context

Change 26 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2. Change 25
(`enable-lingua-italian-for-english-speakers`) lists it-en for English speakers; this change makes what
an English reader is told about Cymbra Lingua know it, as change 21 (`add-lingua-italian-listings`) did
for French readers and `add-lingua-french-listings` for English readers of French. Change 21 is not
implemented yet: its drafts are this change's base. Where things stand:

- **The listing files**, `apps/lingua-extension/STORE-LISTING.md` and `apps/lingua-apple/STORE-LISTING.md`,
  each counting by its own stated rules; #994 (the owner's to merge) records the review notes for the
  next submission, on which change 21 builds.
- **The site**: change 21 gives it Italian's names and `STUDIED.it`; the English home card and the
  Lingua pages are built from the shipped pairs, so it-en reaches them through change 25's coverage
  file with no code change.
- **Change 25** commits the English summary (« The web in Spanish, French or Italian: … », 108) and the
  host app's English activation page.

## Goals / Non-Goals

**Goals:**
- Every English text of the listings names Italian; every French and Spanish text that says what
  English speakers study says it; the fields one per store follow, within their limits.
- The requirements the English listings, the English home card and an eight-row table would make false
  are modified here, implemented before change 25 lists it-en.

**Non-Goals:**
- The English summary and the activation page (change 25).
- Spanish speakers studying Italian (changes 27–31), the Italian interface and its listings (stage 3).

## Measured

Counted on `main` at `58b4ee27e` by each file's own rules, with change 21's drafts (its D3–D5) and #994's
block applied first — every count change 21 records reproduced (Chrome Web Store French 2,924; test
instructions 993; review notes 3,989; App Store fr-FR description 3,299), and the English and Spanish
texts with its D4 sentences (Chrome Web Store 2,912 and 3,426; App Store en-US 3,046, es 3,437).

| Text | Change 21's base | With Italian for English speakers | Limit |
|---|---|---|---|
| Chrome Web Store and AMO, English description | 2,912 | 2,998 | 16,000 |
| App Store en-US / en-GB subtitle | 30 | 26 | 30 |
| App Store en-US / en-GB promotional text | 130 | 139 | 170 |
| App Store en-US / en-GB keywords | 99 bytes | 96 bytes | 100 bytes |
| App Store en-US / en-GB description | 3,046 | 3,141 | 4,000 |
| App Store en-US / en-GB « What's New » paragraph (new) | — | 437 (387 without the voice) | 4,000 |
| Chrome Web Store and AMO, Spanish description | 3,426 | 3,436 | 16,000 |
| App Store es-ES / es-MX description | 3,437 | 3,447 | 4,000 |
| App Store fr-FR description | 3,299 | 3,310 | 4,000 |
| Test instructions (CWS, AMO) | 993 | 988 | 1,000 |
| App Store review notes | 3,989 | 3,994 (3,983 without Morph-it!) | 4,000 |
| CWS single purpose | 602 | 611 | — |

- **Too long said directly**: the subtitle keeping « … as you read » (36 or 38); the keywords with
  `italian,` added (107 bytes; without `words` or `deck` alone 101 or 102; without `extension` 97,
  `reading` 99); the test instructions with step 6
  widened (1,003); the review notes with English speakers named (4,001).
- **The site**, simulated on a scratch copy with change 21's tables and an it-en row: the build and
  `yarn check:routes` pass; only `/en/`, `/en/lingua/`, `/lingua/` and `/es/lingua/` move — the English
  home card « Read the web in Spanish, French or Italian with the words you do not know yet highlighted
  in place… », the English page « Made for English speakers learning Spanish, French or Italian », a row
  « Italian → English », the French and Spanish pages naming Italian among English speakers'
  languages; the card-and-levels sentences are the seven-pair build's. The byte-pinned fixtures mock
  their data and do not move here.
- **The table with eight rows**, in headless Chromium on the built pages: at 375 px the page is 375 px
  and the table 335 px in its 335 px box on all three pages, as with six; at 320 px it scrolls inside
  its box. The new row headers (« Italian → English » 117.9 px…) are under each page's widest.

## Decisions

### D1 — The site: nothing to write, one test, the scenarios true

The English home card and the Lingua pages name Italian from the coverage file change 25 writes; no
code moves. `test/lingua-text.spec.ts` gains a test feeding eight pairs with change 2's routes that
asserts the English home card and the English page's audience and row. The requirements that would read
false once it-en is listed are modified here (D9).

### D2 — The English texts name Italian

Drafts (M9: the owner's words):
- **Chrome Web Store and AMO, English**: « the Spanish, French or Italian words you don't know yet »;
  the card's examples gain « "past historic (passato remoto)" in Italian » (change 18's English name);
  the statistics « levels estimated from word frequency, as no freely licensed CEFR list exists for
  Spanish, French or Italian »; translation « straight from Spanish, French or Italian… the translation
  model of each language you study (26.2–26.3 MB each) »; « How many Spanish, French and Italian words
  our dictionary explains in English: our figures are published at cymbra.app/en/lingua. ». The sentence
  about French speakers stays as change 21 leaves it.
- **App Store en-US and en-GB**:
  - subtitle `Spanish, French or Italian` (26), as change 21's fr-FR `Anglais, espagnol ou italien`;
  - promotional text "The Spanish, French or Italian words you don't know yet, highlighted on the page
    you're reading. The analysis runs on your device, offline." (139);
  - keywords `spanish,french,italian,vocabulary,reading,translation,extension,learn,CEFR,flashcards,dictionary`
    (96 bytes): `words`, the most general, and `deck`, which `flashcards` covers, give way (Open Question 2);
  - description: the first sentence as the Chrome text's; step 3 "For French or Italian, check it in
    the extension's Settings, under Language, then choose its level." (change 25's activation page);
    READ as the Chrome text; REVIEW « for Spanish, French and Italian, levels estimated from word
    frequency »; EXTENDED TRANSLATION and the dictionary sentence as the Chrome text; the last line
    « …for English speakers, who learn Spanish, French and Italian, and in French for French speakers,
    who learn English, Spanish and Italian. »
  - « What's New » for change 25's release, modelled on `add-lingua-french-listings`' English paragraph
    for French's release:
    "Cymbra Lingua now reads Italian too, for English speakers: check Italian in Settings › Language.
    Italian words come with their meaning in English, written by people; the card names the tense and
    the gender ("past historic (passato remoto)"), and your Italian level is estimated from word
    frequency. Read aloud speaks it with a voice from Italy, and extended translation turns your Italian
    selection straight into English, on your device." (437; without the voice clause, as French's, 387 —
    Open Question 3).

### D3 — The French and Spanish texts say what English speakers study

One sentence each, nothing offered to their own readers: the Chrome text in Spanish « …a los
francohablantes, en francés; y español, francés e italiano a los anglohablantes, en inglés »; the App Store's es-ES/es-MX « …y en inglés para
los anglohablantes, que aprenden español, francés e italiano »; the App Store's fr-FR last line « …et en
anglais pour les anglophones, qui apprennent l'espagnol, le français et l'italien ». The Chrome French
text names no other audience. The texts kept for a release without fr-es are not edited.

### D4 — The fields one per store, within their limits

- **Test instructions** (988 / 1,000; 995 if a dashboard counted a line break as two): step 6
  « "Languages studied" > "French" or "Italian" adds its pages »; to make room, « signing in only syncs
  words across devices » becomes « signing in only syncs words ». No step or label lost.
- **App Store review notes** (3,994 / 4,000): the audience « …; English and French speakers Italian
  too. »; to make room, step 2's « e.g. B1 » becomes « (B1) » and « No account is needed » « No account
  needed ». The route note already covers it-en (direct, within « 25.4-26.3 MB each »), the sources
  already credit both Wiktionaries. Widening step 6 to Italian for the reviewer, as French's, fits only
  with two more cuts (« for the glosses », « a pinned » → « pinned »), which read worse: Open Question 4.
- **Single purpose** (611): "— Spanish, French or Italian for English speakers; English or French for
  Spanish speakers; English, Spanish or Italian for French speakers —", the groups set apart by
  semicolons.
- **Remote code**: « it-fr's among them » becomes « it-fr's and it-en's among them », and « for an English
  speaker reading Italian, it-en alone (`it-en/base-memory/2.0`), the direct model of its pair ».
- **Notes not pasted**, each saying what English speakers study: in the extension file, the *Listing
  languages* paragraph, the English description's header, the *Description* introduction, the claims
  table's lead sentence and its « Who learns what » and « Spanish stays the language an English reader
  starts in » rows (it-en last, change 25), the test instructions' rationale (988, the cut), the single
  purpose's note (611), the permissions' introduction, `## Summary` quoting change 25's English summary;
  in the App Store file, the locales and fields table, the en-US description's lead note (change 25's
  step 3, « 26.2–26.3 MB each »), the review notes' lead paragraphs, *What the app itself says* quoting
  change 25's English step 3. The review-notes blocks are labelled by submission: #994's for the next,
  change 21's for change 20's, this one for change 25's.

### D5 — Figures the texts quote, taken when they are known

- **Size**: it-en's served size from change 2's catalogue; « 26.2–26.3 MB each » becomes « 26.2 MB each »
  if it rounds there.
- **The dictionary sentence**: « For Italian, the dictionary is a little less complete than for Spanish
  and French. » if it-en's committed coverage is under both es-en's and fr-en's at each top; none
  otherwise, as fr-en got none. The study (92.0 / 85.1 / 76.7 loose, against 93.0 / 86.5 / 76.5 and
  93.9 / 87.2 / 76.8) favours it-en — 85.3 / 77.5 / 68.7 strict —, so the committed figures decide.
- **Translation**: if change 24's figures and the owner withhold it-en's translation, its clauses go.

### D6 — The claims table

The extension file's « what each claim rests on » gains rows: Italian highlighted for English speakers
(`pairsOf("en")`, it-en last in `packs.json`); the meaning from Wiktionary, written by people (it-en's
glosses, change 22's README); "past historic (passato remoto)" (change 18's renderer, change 22's
`word-card-it-en` snapshot); Italian's levels estimated (change 14); it-en straight into English and its
size (change 2's route, change 25's D5); the figures at cymbra.app/en/lingua (change 25); step 3 (change
25's `copy.js`).

### D7 — Screenshots and dashboards

- **App Store en-US**: one capture more, by hand, from change 25's build, on an Italian page with the
  card open on a passato remoto form; en-GB reuses the set.
- **Chrome Web Store English set**: at its five-capture cap (`STUDIED.en = ["es", "fr"]`), unchanged;
  moving its second capture to Italian needs Italian capture fixtures and moves the site's English alt
  text (Open Question 5).
- **Dashboards**: no language is added. For change 25's submission: `en` description replaced and the
  new summary checked, `es` and `fr` one sentence each, the single purpose, the remote code and the test
  instructions replaced, AMO's English and Spanish pasted; on the App Store, en-US and en-GB's subtitle,
  promotional text, keywords, description, « What's New » and the capture, es-ES, es-MX and fr-FR's
  description, the review notes; a table of what each dashboard shows.

### D8 — Order

Implemented after change 21, after changes 22, 23 and 24 and the owner's answer to change 25's Open
Question 2 — the figures of D5 —, and before change 25 lists it-en, as change 21 is before change 20: its site test and its
scenarios are true once it-en is listed, and its listing texts are pasted with change 25's release after
the site deploy that publishes it-en's figures (I12, M18).

### D9 — Requirements true of their time

- *The store listings name each studied language* (held by change 21): its *French speakers learning
  Italian* says « no English … text offers Italian to its readers »; that clause holds « while it-en is
  not listed », its French texts' obligations unchanged, and *English speakers learning Italian* is added.
- *The coverage table reads at a phone's width* (held by change 21): *Seven pairs on a phone* holds « and
  no other pair »; *Eight pairs on a phone* is added. *Six pairs on a phone* is narrowed the same way in
  change 21 itself, by this pull request, since it-fr's listing already makes it false.
- *The English home page describes Lingua for the readers it serves* (held by
  `add-lingua-french-listings`): *English speakers learning Spanish and French* holds « and no other pair
  glossed in English »; *English speakers learning Italian* is added.

Each MODIFIED block keeps every other word and scenario, re-copied at implementation. True as written:
*The Lingua page says what each studied language's card and levels are* (the eight-pair sentences are
the seven-pair build's), `add-site-lingua-matrix-pages`' *English speakers learning Spanish* (Spanish
stays first), *The stores' summary names what its readers can study* (change 25 commits the English
summary).

## Risks / Trade-offs

- **Texts written before their figures** → D5.
- **Tight fields** → every cut listed (D4); the owner reads them (M9).
- **The English screenshots on the Chrome Web Store show no Italian** → Open Question 5; the App Store's do.

## Migration Plan

Nothing to migrate. The site test and the scenarios land before change 25; the listings are pasted by the
owner with change 25's release. Rollback is a revert.

## Effort

Against 1.5–3 ideal days: the English texts and the French and Spanish sentences, recounted, 0.5–0.75;
the fields one per store 0.25–0.5; the site test and the scenarios 0.25–0.5; the claims and the
dashboards' procedure 0.25; docs 0.1. 1.35–2.1.

## Open Questions

For the owner:
1. **The drafts** (D2–D4): as written, or other words (M9); a subtitle that keeps "read",
   `Read Spanish, French, Italian` (29), fits too.
2. **The keywords**: `words` and `deck` give way (recommended), or one longer word alone (`extension`,
   97 bytes; `reading`, 99).
3. **« What's New »**: with the voice clause, as change 21's French paragraph (recommended), or without,
   as French's English one.
4. **The review notes**: without an Italian step (recommended), or with one and two more cuts.
5. **The Chrome Web Store's English screenshots**: unchanged (recommended), or the second capture moved
   to an Italian page.
