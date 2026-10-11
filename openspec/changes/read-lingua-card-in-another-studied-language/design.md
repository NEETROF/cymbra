# Design — read-lingua-card-in-another-studied-language

## Context

See proposal.md (Why). What exists, on `origin/main` at `61430101`:

| Where | What |
|---|---|
| `lingua-browser-extension` *Each document is read in its own language* | with several accepted languages, the session chooses each document's language among them (`detectLanguage`, the declared `lang` as a hint) and asks every language-bound question about the document in it: analysis, glosses, cards, statuses, exposures |
| `lingua-analysis` *Per-block language detection*, *A document's language, chosen among the reader's* | a block detected in another language is left out of the analysis; a page with no block in a candidate language is not analysable, and its language is then the declared candidate or the first one |
| `src/reading/session.ts` | `language` (the document's), `languages` (the reader's accepted ones, the profile's order), `lang` = `port.for(language)`; `repaint` analyses and paints; `onGesture` writes the status, card and learnt-word count through `lang`; one `Speaker` bound to `language` |
| `src/reading/selection-card.ts` | `SelectionCards`: `openForToken` (a click, or a one-word selection over a page token: an unknown word with a gloss or none completes with its grammar under `GRAMMAR_WAIT_MS`, never drawn pending), `openLooseWord` (a one-word selection no token covers: `phraseGloss`, then the grammar, drawn pending under `ANSWER_TIMEOUT_MS`), `openExpression` (several words); `rarityText` turns the rank into one of five bands, « Rare — au-delà des 20 000 » when the pack ranks nothing |
| `src/analyzer/engine.ts` | `for(language)` is a view naming its language on every call; a language's pack is fetched and added to the one engine the first time it is asked (`engineFor`); `wordGrammar` adds `frequencyRank` |
| `src/reading/wordpopup.ts` | `WordPopupContent.language` says the studied language of the card's words (absent: English), for the grammar's names and the `lang` of the words; `Gesture` carries no language |
| `lingua-analysis` *Catalan and Galician are not read as Spanish*; changes 42 (`add-lingua-french-detection-guard`) and 42b (`add-lingua-spanish-occitan-guard`), open | a block whose Catalan, Galician (and with 42b Occitan) function words outnumber its Spanish ones is not read as Spanish; one whose Catalan, Occitan or Romanian ones outnumber its French ones is not read as French |
| Sync (`lingua-analysis` *Sync records carry their own language*, `lingua-sync` *A device pulls the cards of its reader's languages*) | statuses and cards are exported, pushed and pulled with their language; a device applies only those of the languages it accepts, and widening its languages pulls again from the start |

A reader studies at most two languages in this programme: the three languages minus their native
one. The case appears for every reader who studies two — French speakers today (English and
Spanish), English speakers (Spanish and French) and Spanish speakers (English and French) once
changes 34, 35 and 52 ship.

**How it was measured.** A Rust harness, never committed (scratchpad
`read-lingua-card-in-another-studied-language-work/harness`), links `lingua-wasm` and
`lingua-pack` from this worktree, builds each pack from the committed tables as the goldens do
(`lingua_pack::inputs_from_tables` + `build_pack`), and holds two packs of one native language in
one engine, as a reader's engine does. For each page it analyses the blocks in the document's
language and, for every unknown token without a gloss, asks `wordGrammar` and `frequencyRank` in
that language; a token with no reading, no other dictionary form, no gloss and no rank is one the
document's language does not know (the trigger, D1). It then asks `phraseGloss` of the word as
written in the other language and counts a re-read when the answer is one token, not a name, with a
gloss (D2). A second harness reads every word of a page as a loose selection (`phraseGloss` of each
line, no page gate), for the selection path and the guards (D7). The pages are the three baseline
corpora (`crates/lingua-wasm/tests/baseline/pages.txt`, `pages-es.txt`, `pages-fr.txt`) and
Wikipedia articles fetched as plain text (`action=query&prop=extracts&explaintext`) on 2026-10-11.
A fresh reader is measured: a word the document's pack does not rank cannot be presumed known by a
level, so the trigger does not depend on the reader's level. Each re-read was read by hand.

## Goals / Non-Goals

**Goals**

- A word the document's language does not know opens the card of the reader's other language when
  that language knows it, on every path that opens a word card.
- What the reader does on that card lands in that language, on this device and through the sync.
- Nothing moves for a reader who studies one language, for a word the document's language knows,
  or in any golden, snapshot, pack or analyser output.

**Non-Goals**

- Re-reading at paint: the page's analysis, highlights, percentage and exposures stay the document
  language's (D6; open question 1).
- Several words: an expression or a selection of several words keeps its card (D8; open question 2).
- Reading a block in a second language, or splitting a page into languages: detection is unchanged.
- A setting: the re-read follows the languages the reader studies, nothing else.

## Decisions

### D1 — When: a word the document's pack does not know at all

The re-read is asked for a word card whose word, read in the document's language, is **unknown**
(class `Unknown`), has **no gloss**, and whose grammar answer has **no reading, no other dictionary
form and no rank** — the pack's lexicon does not hold it as anything. That is exactly the card the
owner met: « hablaba » on a French page, headed « hablaba », « Rare — au-delà des 20 000… ».

Left alone, each with its card of today:

- a word the reader marked in the document's language (`Known`, `Learning`, `Ignored`): the reader
  decided what it is there;
- a name (`ProperNounOutOfLexicon`): a name is not a word of another language;
- a word the document's pack ranks or reads, with or without a gloss (a French word missing from
  fr-en's glosses is still French, and its card keeps « Peu fréquent… »);
- a selection of several words (D8).

The condition is computed in the extension from answers the card already asks — the token, then
`wordGrammar` and the rank — so the engine gains no call and no Rust file moves.

Measured, among unknown tokens with no gloss, the share the document's pack does not know at all:
French Revolution pages 164 of 336 (fr), 191 of 313 (en), 210 of 469 (es); a page about the other
language 365 of 482 (*Espagnol*, fr) and 602 of 1,086 (*Spanish verbs*, en). The rest — French or
English words without a gloss — keep their card.

### D2 — Where: the reader's other studied languages, the first that glosses the word

The candidates are the reader's accepted languages other than the document's, in the profile's
order. Each is asked `phraseGloss(word as written)`; the first whose answer is **one token, not a
name, with a gloss** wins, and its card is completed with `wordGrammar(word as written, its lemma)`
and its rank, as a card of that language is. A gloss is required: a word another pack ranks but
cannot gloss would trade one empty card for another.

With this programme's three languages a reader studies at most two, so there is exactly one other
language; the order only matters with a fourth, and then the reader's own order decides — no card
shows two languages.

The views are the existing ones (`port.for(other)`): the other pack is fetched and added to the
engine the first time the re-read asks it, and never for a reader who opens no such card. Measured
natively: adding es-en (2,612,075 B) to an engine on fr-en takes 32.6 ms, fr-en (3,003,601 B) to
es-en 37.5 ms, es-fr (2,234,330 B) to en-fr 27.8 ms; one re-read (the document's grammar and rank,
then the other's phrase gloss, grammar and rank) takes 8–12 µs.

### D3 — Within the card's bound, never a second wait

On the click path the card completes once its grammar arrives or `GRAMMAR_WAIT_MS` passes. When
that answer says the document's language does not know the word (D1) and another language is
accepted, the card waits for the re-read under one more `GRAMMAR_WAIT_MS`; within it, the re-read
card is shown and the document's card never is. Past it — a cold pack on a slow device — the card
of today is shown, and the re-read replaces it when it lands if that card is still the one shown
(the generation check `show` already makes for a translation). On the loose-selection path the card
is drawn pending, as today, and the re-read completes it under the same `ANSWER_TIMEOUT_MS`.

No re-read is asked at paint, so a page costs nothing more to read: the cost is one card's, when
the reader opens it.

### D4 — What the card shows

The card of the language it was read in, as that language's card is today:

- headed by its dictionary form (`hablar`), the form seen (« hablaba ») beside it;
- its gloss, in the reader's native language (one engine, one native: es-en's "to talk, to speak, …"
  for an English speaker);
- its grammar line, the form named as that language's (`WordPopupContent.language` = the card's
  language; the renderers of changes 18, 23, 24 and 51): "first- and third-person singular
  imperfect indicative of hablar";
- its frequency band from that pack (`hablar`, rank 181: "Common — among the 1,000 most frequent
  words");
- **one line naming the language** before the frequency line: « Mot espagnol », "Spanish word",
  « Palabra española » — `selection.readAs`, one slot message per interface language taking the
  labels module's `masculine` or `feminine` form (« mot » is masculine, « palabra » feminine), so no
  surface writes a language's name. The page around it is in another language; without the line the
  reader cannot tell a Spanish card from a French one.
- the word's and the dictionary form's listen buttons speak with that language's voice (the voice
  the reader chose for it, `A voice per studied language`); the sentence button keeps the
  document's, since the sentence is the document's;
- no translation is asked: the card has a gloss (`wordEngine`'s rule).

When no other language knows the word, the card is exactly today's, translation offer included.

### D5 — Recorded in the language it was read in

A re-read card's content carries `readIn` (the language it was read in; absent on every other
card, so their content and gestures are byte for byte today's), and its gesture carries it.
`onGesture` writes through `port.for(g.readIn ?? language)`: « Je connais » sets `hablar` known in
Spanish and retires a Spanish card; « + Deck » adds a Spanish card whose gloss is es-en's
(`cardGloss` asked of that view); the day's learnt-word count goes to Spanish. Nothing is written
under the document's language — « hablaba » never becomes a French status.

Across devices nothing new is needed: statuses and cards already carry their language to the
server and back. A device that accepts Spanish applies `hablar` known; one that does not skips it
and pulls it again if Spanish is added later (`lingua-sync`, *Widening a device's languages pulls
again from the start*). Whether a card is re-read depends only on the device's accepted languages —
the same choice that already decides what it pulls — so two devices with the same languages give the
same card, and no new setting or synced field exists to diverge.

### D6 — The page stays read in its language; the document remembers its re-read words

The analysis, the highlights, the HUD's percentage, the exposures and the daily figures stay the
document language's. « hablaba » still counts as an unknown word of the French page and is
highlighted until the reader opens it. Re-reading at paint was weighed and left out: it would ask,
on every repaint, every word the document's pack does not know — 89 distinct forms on the French
*Révolution française*, 327 on *Espagnol*, 373 on *Spanish verbs* — load the other pack in every
tab of every reader who studies two languages, and need a batched engine call so the messaging
port of a page that refuses WebAssembly does not carry hundreds of requests; it is open question 1.

What this change does instead: the session remembers, for the open document, each form a card
re-read (the form as written, lowercased → its language and dictionary form). At each repaint it
asks that language once per remembered form and paints those tokens by their status there — none
for known or ignored, the learning colour for learning, the unknown colour otherwise. So « Je
connais » on « hablaba » takes its highlight off at once, as it does for any word. The memory is the
document's: a new document, or the reader's library, starts empty. A handful of forms, asked once
per repaint; the page's figures do not move.

### D7 — Never into a language whose guard keeps the declared language out

The guards say which neighbours are not read as a studied language: Spanish keeps Catalan and
Galician out (and Occitan with change 42b), French keeps Catalan, Occitan and Romanian out (change
42). A word card selected on such a page, read word by word, would bring those languages back
through the re-read. Measured with the loose-selection harness, every word of the page selected:

| Page (declares) | Document's language → re-read | Words | Unknown there | Re-read (distinct) |
|---|---|---|---|---|
| ca *Barcelona* (`ca`) | fr → es | 12,542 | 3,939 | 670 (343): *arribar* "to dock", *barata* "cockroach" |
| ca *Barcelona* (`ca`) | en → es | 11,932 | 4,444 | 616 (321) |
| ca *Barcelona* (`ca`) | es → fr | 12,713 | 3,364 | 223 (99) |
| gl *Galicia* (`gl`) | fr → es | 8,862 | 3,617 | 1,930 (1,057) |
| gl *Galicia* (`gl`) | en → es | 8,912 | 3,697 | 1,775 (986) |
| oc *Occitan* (`oc`) | es → fr | 3,600 | 1,386 | 170 (33): *dins* × 59 |
| oc *Occitan* (`oc`) | en → fr | 3,467 | 1,685 | 127 (37) |

So: **no re-read into a language whose guard keeps the document's declared language out** — a page
declaring `ca`, `gl` or `oc` (`cat`, `glg`, `oci`) gets no Spanish re-read, one declaring `ca`, `oc`
or `ro` (`ron`, `rum`) no French one; `primaryLanguage` learns the three-letter codes. One table in
`session.ts` holds the lists, each entry naming the guard it mirrors. Those Wikipedias declare
their language, so every row above becomes 0.

Not covered, said as a risk: a page that declares nothing or declares wrongly, and a Catalan
quotation inside a page declared French (its block is left out of the French analysis, and a word
selected in it is a loose selection like any other). The sentence is not run through the guard: the
guard counts function words over a block, and a Spanish word quoted in a French sentence would be
judged by the French around it.

### D8 — One word only

A selection holding whitespace keeps the whole-selection card in the document's language: its
word-by-word rows, its expressions and its translation all read the document's language, and mixing
languages row by row would show a French « la » beside a Spanish *casa*. A word written as pieces
(French elision, change 51's D7) is one word when one piece is selected, and that piece is re-read
like any word. Whether « yo hablaba » should be read whole in Spanish is open question 2.

### D9 — Order

Implemented on `main` once #810 (change 34), #814 (change 35) and #880 (change 52) have merged: they
widen the studied-language type, the profile and the session the re-read reads, and a pull request
is never stacked on another without retargeting it. Nothing here waits on them in principle — French
speakers who study English and Spanish have the case today — but the owner's own case (es-en and
fr-en) needs change 52, and its dogfood needs all three natives.

`.openspec.yaml`'s `archiveAfter` lists the three enable changes and the two guards whose lists D7
reads: changes 42 (`add-lingua-french-detection-guard`) and 42b (`add-lingua-spanish-occitan-guard`)
ADD the requirements naming them, and this change's spec names their languages. The MODIFIED
requirement is held by no open change.

### D10 — What moves, and measured figures

Nothing in `crates/`, no golden (`en-fr`, `es-fr`, `es-en`, `en-es`, `fr-en`, `fr-es`), no pack,
pin, table, analyser version, committed snapshot or `word-grammar.spec.ts` line: the engine is asked
nothing it is not asked today, and the renderers are unchanged. What a reader sees moves only on a
card whose word the document's pack does not know, for a reader who studies two languages: that
card, in French for French speakers too — the programme's rule « the French interface does not
move » is departed from there, by the owner's decision of 2026-10-11. A reader who studies one
language sees nothing new.

Re-reads on analysed pages (click path), fresh reader, the document's language chosen as the
reader's would be:

| Page | Reader (packs) | Doc → other | Tokens | Not known there | Re-read (distinct) | Judged wrong |
|---|---|---|---|---|---|---|
| fr *Grammaire de la langue espagnole* | en (fr-en, es-en) | fr → es | 1,993 | 96 | 61 (46) | 1 (*acha* → *acho*) |
| fr *Espagnol* | en (fr-en, es-en) | fr → es | 4,412 | 365 | 133 (119) | 4 (*chapela* → *chape*, *palatal*, *ibex*, *cueto*) |
| fr *Anglais* | es (fr-es, en-es) | fr → en | 4,741 | 68 | 11 (11) | 0 |
| en *Spanish verbs* | fr (en-fr, es-fr) | en → es | 9,988 | 602 | 451 (295) | 2 (*dialectal*, a cognate) |
| en *Spanish grammar* | fr (en-fr, es-fr) | en → es | 5,896 | 381 | 218 (141) | 2 (*copular*, *pronominal*, cognates) |
| es *Idioma inglés* | fr (es-fr, en-fr) | es → en | 3,362 | 50 | 2 (2) | 0 |
| en *French Revolution* | es (en-es, fr-es) | en → fr | 12,079 | 191 | 24 (11) | 0 (French quotations: *parlements*, *levée*) |
| es *Revolución francesa* | en (es-en, fr-en) | es → fr | 15,459 | 210 | 57 (34) | 0 (*cahiers de doléances*, *enragés*) |
| fr *Révolution française* | en (fr-en, es-en) | fr → es | 17,161 | 164 | 5 (4) | 5 (*bicaméral*, *discordantes*, *tramé*, *vivante*) |
| fr *Révolution française* | es (fr-es, en-es) | fr → en | 17,161 | 164 | 1 (1) | 1 (*discordantes*, a cognate) |
| en *French Revolution* | fr (en-fr, es-fr) | en → es | 12,079 | 191 | 6 (5) | 6 (*noblesse*, *émigré*, *germinal*; *unicameral*, *venal*, cognates) |
| es *Revolución francesa* | fr (es-fr, en-fr) | es → en | 15,459 | 210 | 12 (3) | 12 (*tiers* × 10 → *tier*, *publics*, *vents*) |
| baseline `pages-fr.txt` (13 pages) | en / es | fr → es / en | 983 | 5 | 0 | — |
| baseline `pages.txt` | fr / es | en → es / fr | 820 | 12 | 0 | — |
| baseline `pages-es.txt` | fr / en | es → en / fr | 531 | 4 | 0 | — |

Read: where the other language is quoted, the re-read reads it, wrong about 1 % of the time and
then mostly on a cognate whose gloss says the same thing; where it is not, it reads at most 12 tokens
in 12,000–17,000 (≤ 0.08 %), each a word the document's pack lacks — a French form fr-en does not
list, or a third language's word quoted (French *tiers* in Spanish read as English *tier*) — and only
when the reader opens that word's card. The baseline corpora give none.

On a page in the reader's own language, which is not analysed, a selected word is read in the first
studied language and, when that does not know it, in the other: a French speaker selecting « hablaba »
on a French page gets *hablar*. Selecting a French word there can give a false friend — every word of
*Révolution française* selected, 303 of 15,429 (2.0 %) are read as Spanish (« demande » →
*demandar* « poursuivre en justice », « crée » → *creer* « croire ») — words of the reader's own
language, where today's card is empty anyway.

## Risks / Trade-offs

- **A false friend on a word the document's pack lacks** (*tiers* → *tier*). Bounded by D1 (only
  words the document's pack does not know at all, only on a click) and measured at ≤ 0.08 % of a
  long page's tokens; the language line says which language the card is in.
- **Neighbour languages on pages that do not declare them** (D7). The guards' lists act only on a
  declared language; a Catalan quotation in a French page stays exposed.
- **A first re-read on a cold pack** misses the bound on a slow device and shows today's card first,
  then the re-read (D3).
- **The highlight comes back on the next visit** until the reader opens the word again (D6; open
  question 1).
- **The French interface moves** for French speakers on those cards, by the owner's decision (D10).

## Migration Plan

None: no stored format, wire field or setting. The next release reads cards this way; a rollback
leaves statuses and cards recorded in Spanish, which every version reads.

## Effort

2–3.5 ideal days: the cards' re-read and its bound (1–1.5), the session's gesture, memory and guard
table (0.5–1), the card's line, voice and copy (0.25–0.5), tests and the dogfood record (0.25–0.5).

## Open Questions

1. **Should a word you marked in Spanish lose its highlight on every French page, or only on the page
   where you marked it?** Example: on Monday you read a French page about Spanish, open « hablaba »,
   press « Je connais » — its highlight goes (this change). On Tuesday another French page writes
   « hablaba »: with this change it is highlighted again until you open it, and its card says you
   know *hablar*. Removing it on every page means re-reading, at every paint, each word the page's
   language does not know (up to 373 distinct forms on a long page), loading the other language's
   dictionary in every tab, and one new engine call — about 1.5 more ideal days, in a change of its
   own.
2. **Should a selection of several words that the page's language does not know at all be read
   whole in the other language?** Example: on a French page you select « yo hablaba »: today, and
   with this change, the card lists « yo » and « hablaba » word by word with no gloss; read in
   Spanish it would show both Spanish words glossed, and « Traduction étendue » would translate it
   from Spanish. A rule is needed for a mixed selection (« la casa » holds the French « la »).
