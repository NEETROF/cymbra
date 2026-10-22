# Design — enable-lingua-spanish-for-italian-speakers

## Context

Change 54 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 4. Nothing Italian
is implemented on `main`; every fact about changes 20–53 is their proposals'. This change follows change
50 (`enable-lingua-french-for-italian-speakers`) step for step, Spanish in place of French; only what
differs is written here, the rest being change 50's decisions read with es-it for fr-it.

- **Change 50** lists fr-it after en-it, offers French to Italian speakers, names French in every Italian
  text, and takes the Italian captures again with French as the second studied language.
- **Changes 51–53** build es-it: the full pivot, at about 85.7 / 75.6 / 63.9 on the study, a floor of
  about 83.7 / 73.6 / 61.9 fixed at change 51's task 0.1 (51); refined (52); its marks through es-en then
  en-it judged in Italian by two agent judges, with no trial run before the criteria (53).
- **The order rule** of the list: the first pair glossed in a native language sets that reader's
  default; en-it stays ahead of es-it.

## Goals / Non-Goals

**Goals:**
- es-it listed; Spanish offered to Italian speakers after English; a Spanish page read, glossed,
  recorded, read aloud and translated for them.
- Everything an Italian reader is told names Spanish.
- Nothing moves for French-, English- and Spanish-native readers.

**Non-Goals:**
- es-it's tables, refinement and marks (changes 51–53).
- Anything change 50 already does for Italian readers.

## Decisions

### D1 — The list, es-it at the end

`packs.json` gains `"es-it"` at the end of the list, `SHIPPED_PAIRS` the same; en-it is ahead of it, this
change following change 45. No testdata is added: change 51 writes `testdata/es-it/`.
`committed_tables.rs`'s « listed in no package while the list does not name it » for es-it flips.

### D2 — es-it by its floor, after its refinement

As change 50's D2, for es-it: `ListedFloorsTest` names it, it is « published once listed », and it is
listed once change 52 has merged (Open Question 1).

### D3 — The extension

As change 50's D3: `shippedLanguages(pairsOf("it"))` gives English, French and Spanish where fr-it is
listed, English and Spanish where it is not. The card names Spanish tenses in Italian (change 34: the
pretérito indefinido « passato remoto », « congiuntivo futuro (futuro de subjuntivo) », « condizionale
presente », « gerundio »); `GUARDED.es` keeps the neighbours' pages out; online translation accepts
Spanish into Italian since change 20's D5.

### D4 — Translation: two models, es-en then en-it

As change 50's D4, at change 53's figures; if not offered, `routes["es-it"]` is dropped and the route
texts are MODIFIED from their latest holders.

### D5 — The memory sentence's Italian cases

es-en is 26,241,052 B; en-it's served size comes from change 2's catalogue (change 49's trial measured
26,509,835 B):
- Spanish alone: es-en and en-it, about 340 MB, the download their sum (≈ 52,750,887 B, « 52,8 MB »);
- English, French and Spanish: en-it, fr-en and es-en, held two at a time, about 340 MB (≈ 78,985,602 B,
  « 79,0 MB »);
- English and Spanish: the same two models as Spanish alone, en-it serving both routes.

### D6 — The Italian description and activation page

`_locales/it`'s `extensionDescription` names every language Italian readers study, within 112
characters (drafts for the reviewer agent, I2):

| Listed | Draft | Characters |
|---|---|---|
| fr-it and es-it | « Il web in inglese, francese o spagnolo: parole sconosciute evidenziate, percentuale reale. Senza tracciamento. » | 110 |
| es-it, not fr-it | « Il web in inglese o spagnolo: parole sconosciute evidenziate, percentuale reale. Offline e senza tracciamento. » | 110 |

With three languages « Offline e » gives way (120 with it), as change 50's D6 foresaw. The activation
page's `it` `lede` and `step3` name Spanish beside French: « Leggi il web in inglese, in francese o in
spagnolo: … », and « Per il francese o lo spagnolo, selezionalo nelle <strong>{settings}</strong>
dell'estensione, scheda <strong>{language tab}</strong>, poi scegli il suo livello. » The page grows by
a few words; the dogfood reads it on an iPhone.

### D7 — The Italian listings name Spanish

Drafted by analogy with change 50's French sentences, for the reviewer agent, counted when change 46's and
50's texts exist: the languages named (« in inglese, in francese o in spagnolo »); the card (« in
spagnolo, «passato remoto» », the gender of Spanish nouns); the levels (« per il francese e lo spagnolo,
livelli stimati… »); translation (« dal francese e dallo spagnolo, passando per l'inglese »); the sizes
(« 52,8 MB con lo spagnolo, 79,0 MB con i due »); the dictionary sentence naming Spanish if es-it's
committed coverage is under en-it's at each top; the App Store it-IT subtitle « Inglese, francese o
spagnolo » (28 / 30), promotional text, keywords (`spagnolo`, a word giving way if over 100 bytes) and
« What's New ». The shared English fields: the single purpose « …; English, French or Spanish for Italian
speakers »; the remote code « …and reading Spanish, es-en then en-it ». Every new Italian sentence
passes change 44's glossary test.

### D8 — The site

Data-driven, as change 50's D8. The Italian captures keep French as the second studied language where
fr-it is listed; where es-it is listed and fr-it is not, they are taken again with Spanish
(`STUDIED.it = ["en", "es"]`, the seed state recorded again, the second shot's alt text rewritten). The
Italian media stay within change 41's 50 MB budget.

### D9 — Tests and documents

Change 50's D9, with es-it for fr-it and Spanish for French: the tests that read the list, keeping the
earlier lists as explicit arguments; the Italian-interface cases (`profile`, `language-labels`,
`settings-language`, `onboarding-level-row`, `popup`, `stats-view`, `wordpopup` with « passato remoto »,
`engine`, `native-language`, `model-residency`, `translation-setting`, `model-controller`,
`translate-relay` if change 53 marks es-it); the site's tests; `committed_tables.rs`;
`test_gloss_coverage.py`.

### D10 — The dogfood pass, before merge

Change 50's D10 with Spanish: offered after English (and French), Spanish pages, the card in Italian,
read-aloud, extended translation with Spanish alone and with three languages, online translation from
Spanish into Italian, an Italian-native install of change 50's release updated in place, the other
natives unchanged, sync, the rollback to change 50's release, the host app's Italian page on an iPhone,
the Italian Lingua page and home card. A sample of es-it's pivot glosses is read against their Spanish.

### D11 — The release

On its own, TestFlight first, after change 50's. The site deployed after the merge with es-it's figures,
then the listings pasted.

### D12 — Requirements true of their time

MODIFIED, each copied whole from change 50 as it then stands: the list; the memory sentence; *Guided
activation*; the listings (*Italian speakers learning French* narrowed to « and es-it is not »); the
coverage table; the showcase; the Italian home card. ADDED *Spanish is studied by readers of Italian*.
RENAMED change 51's « …, and not shipped ». Still true as change 50 leaves them: change 40's host
requirement (narrowed by change 50), change 50's *French for an Italian speaker* (« es-it is not »), its
*The Italian page with French*, and its home card's *Italian speakers learning French* (« and no other
pair glossed in Italian »).

### D13 — Against change 50

Change 50's D13 rule: whichever of changes 50 and 54 is implemented later re-copies the earlier's shared
blocks and names it in `archiveAfter`. This change is recommended after change 50 and copies its blocks.
If it is implemented first, it re-copies from change 50's holders (changes 30, 31, 45, 46, 41, 25 and
40, by change 50's own D13), drops change 50 from `archiveAfter`, and its list names es-it without
fr-it; change 50 then re-copies this change's blocks and names it, and French is offered after English
and Spanish in the list's order.

## Risks / Trade-offs

- **es-it reads through a pivot** → its floor, change 52 before the list, the dictionary sentence, the
  dogfood's sample (D10).
- **Three languages in 112 characters** → « Offline e » gives way (D6), the reviewer agent's wording.
- **Translation short of the tier** → M15 at change 53's figures (D4).

## Migration Plan

Nothing to migrate: Spanish is offered, never added. Rollback is change 50's release, which knows `es`
and `it` but lists no es-it.

## Effort

4.5–9 ideal days, against the row's 3–6 (change 50 was raised to 4.75–9.5; the captures are taken again
only where fr-it is not listed):

| Part | Ideal days |
|---|---|
| The list, the tests that read it, the coverage write, the fixtures | 0.75–1.25 |
| The Italian reader's offer, memory and residency tests | 0.5–1 |
| `_locales/it` and `copy.js` with their tests | 0.25–0.5 |
| The listings | 0.75–1.25 |
| The captures, where fr-it is not listed | 0–0.5 |
| The site's scenarios and tests | 0.25–0.5 |
| The spec blocks | 0.25–0.5 |
| The reviewer agent's pass | 0.25–0.5 |
| The checks | 0.25–0.5 |
| The M15 branch | 0–0.25 |
| The dogfood | 1.25–2.25 |

## Open Questions

For the owner:
1. **es-it after its refinement** (D2): listed only once change 52 has merged. Recommended.
2. **es-it under 75 % correct or over 30 % withheld** (D4), at change 53's figures: offered without a mark,
   or not offered.
3. **The description** (D6): the drafts, the reviewer agent settling the wording (I2).
4. **The release** (D11): on its own, TestFlight first, after change 50's. Recommended.
