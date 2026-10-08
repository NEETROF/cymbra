# Design — add-lingua-french-baseline

## Context

See proposal.md (Why). Where a third studied language has to answer today:

| Seam | Today |
|---|---|
| `analysis/language.rs` | `StudiedLanguage { English, Spanish }`, `ALL`, `tag()`, `from_tag()`, `analyzer_version()`, `whichlang_target()`; whichlang has a French class |
| `analysis/mod.rs` | `ANALYZER_VERSION` (English, `1.1.0`), `SPANISH_ANALYZER_VERSION` (`1.2.0`) |
| `analysis/tokenize.rs` | `nfc_for` (Spanish in NFC, English as it came), `split_contraction` (`n't`; `al`/`del`); the `’` → `'` map and the rest belong to no language |
| `analysis/lemmatize.rs` | `lemmatize` dispatches to English's cascade or `spanish::lemmatize`; the baseline function of `generalise-lingua-analysis-by-language` D3 was retired when Spanish got its cascade (#655) |
| `analysis/function_words.rs` | tables per language |
| `engine.rs` | `document_names` for Spanish, none for English |
| `knowledge/profile.rs` | `NativeLanguage::French.studied()` is `None`, « until French is studied »; `check` refuses studying one's native language |
| `packs/pack.rs` | `meta.studied` through `from_tag`, the version compared within the language |
| `decks/backup.rs` | `BACKUP_SCHEMA_VERSION = 2`; `backup_version()` is 1 for the default profile with English records and French glosses, 2 otherwise; `from_backup` reads a header holding only `schema_version` first and refuses any version it does not read as `UnsupportedVersion { found, supported }` |
| `crates/lingua-wasm/tests/support` | one harness, a scenario per pair (`english.rs`, `spanish.rs`), the pair's pack built from `scripts/lingua-data/tables/<studied>/` + `<pair>/`; `tests/languages.rs` builds a small Spanish pack by re-stamping the testdata inputs |
| `apps/lingua-extension/tool/packs.mjs` | `ANALYZER_CONSTANTS = { en, es }`; `test/packs.spec.ts` parses `language.rs` and requires the map to name every variant's constant |
| `apps/lingua-extension/src/state/profile.ts` | reads the profile out of the stored backup without an engine; a studied-language name it does not know is dropped, English when none is known |
| `apps/lingua-agent/rust/src/vocab.rs` | `language_name` matches every variant |

An engine serves one native language (`An engine serves one native language`): it refuses a pack
glossed in another. A pack studying French is glossed in English or Spanish, so no engine holding
en-fr can hold it; an English-native reader's engine starts on es-en (`defaultPair("en")`) once
that pair is listed. The committed tables hold en, en-fr, es, es-en and es-fr; nothing French.

## Goals / Non-Goals

**Goals:**
- A third variant the whole core compiles and dispatches on, French served by the baseline
  analysis, with English and Spanish proven unmoved.
- A golden over raw French text that every later change of stage 3 re-blesses with a reason.
- A backup version that a released build refuses by name.

**Non-Goals:**
- Any French rule: elision, `au`/`aux`, the hyphenated inversion, NFC, the cascade, function
  words, names (changes 40, 41; M21).
- The Catalan and Occitan guard (42), the tables, grammar and levels (43, 45, 46; M7, M8), the
  packs (48, 49; M6), the card and the enable (51, 52).
- The extension's `StudiedLanguage` type, labels and `packs.json`; the back office's name for
  `fr`; the agent's French (55).

## Decisions

### D1 — French is appended; serialised names and order stay

`StudiedLanguage::French` goes after `Spanish`, as the type's own doc line asks: the derived order
keys the knowledge model's maps and serde writes variant names, so a state holding English or
Spanish serialises byte for byte as before. `ALL` has three entries, `tag()` answers `fr`,
`from_tag("fr")` answers French, `whichlang_target()` is whichlang's French class.
`FRENCH_ANALYZER_VERSION = "0.1.0"` stands beside the two others in `analysis/mod.rs`, read by
`analyzer_version()`.

*Rejected — a cfg-gated or feature-gated variant.* Every exhaustive `match` would need a gate, the
extension's spec parses `language.rs` for variants, and a gate is exactly the kind of switch a
silent release does not need: the variant is inert without a listed pair.

*Rejected — adding the variant in change 40 or 41.* Each would carry the plumbing beside its
rules, and there would be no golden to show what the rules changed.

### D2 — French's arm is the baseline analysis, exactly as the spec defines it

*Analysis by studied language* already says what a language whose rules are not written gets:
segmentation, the pre-pass rules that belong to no language (edge apostrophes, hyphenated
compounds, words with digits dropped, single letters counted only when the pack lists them), the
pack's form→lemma lookup; no exception table, no morphological rule, no contraction split, no
function words. French takes that, and nothing more:

- `nfc_for` returns French text as it came (Spanish's NFC stays Spanish's);
- `split_contraction` answers `None`: `au`, `aux`, `du`, `des` are one token each, and
  `l'homme`, `qu'il`, `aujourd'hui` are one token each — UAX #29 keeps a letter-apostrophe-letter
  run together, and `push_word` maps the typographic apostrophe `’` to `'` for every language
  before the pre-pass (`tokenize.rs`), so the corpus's real apostrophes read as the straight one;
- `lemmatize` dispatches to `lemmatize_baseline`: the pack's lemma for the lowercased form, else
  the lowercased form (D3);
- `is_function_word` has no French table: `pas`, `ne`, `le`, `de` count like any other word;
- `document_names` answers none for French, as for English.

What this does to the corpus is the point: `l'homme` comes out as one token lemmatised `l'homme`,
`au` as `au`, `dit-il` as a hyphenated compound. That is the before picture change 40 moves, and
the golden records it.

*Rejected — no analysis at all (the engine refuses French).* It would need a special case in
`analyse` instead of an arm, freeze nothing, and contradict the requirement above.

*Rejected — English's rules.* `n't`, the irregulars (`as`, `are`, `a`, `ate` are French words) and
the suffix rules would read French wrongly, and the requirement forbids a language's rule on
another's text.

*Rejected — NFC now.* It is a pre-pass rule of French's own, which the spec's baseline does not
have, and no live page is read as French before change 52. The corpus holds one block committed
in NFD on purpose (D5), so the golden shows its decomposed tokens today and change 41's NFC rule
moves them: a before and an after in the diff of that re-bless, rather than a rule nobody can see.

### D3 — `lemmatize_baseline` returns

The function `generalise-lingua-analysis-by-language` wrote for Spanish's first arm — the pack's
forms, else the lowercased form, with a doc comment listing what it lacks — is written again as
French's arm. Its tests take the words that would go wrong under English's cascade (`as`, `are`,
`ate`, `mes`, `has`, `a`) and under Spanish's (an acute-accent retry, the enclitic rule), and show
each coming back as itself, or as the listed lemma.

*Rejected — routing French through Spanish's cascade « for now ».* The enclitic rule and the
accent retry are Spanish's; running them on French is what the architecture forbids.

### D4 — A version per language, read by the extension's constant map

`FRENCH_ANALYZER_VERSION` follows the two others' pattern (`pub const …: &str = "…"`), so
`coreAnalyzerVersion` reads it with the same anchored expression. `ANALYZER_CONSTANTS` gains
`fr: "FRENCH_ANALYZER_VERSION"`: `test/packs.spec.ts` derives the map from `language.rs` and
would fail without it. The build reads the constant for listed pairs only, and none studies
French, so the entry is inert until change 48 lists fr-en.

`0.1.0` is the version the spec asks of a language served by the baseline. Change 40 bumps it for
the tokenisation (`0.2.0`), change 41 for the cascade (`1.0.0`, as Spanish's
`add-lingua-spanish-analysis` did); the fixture pack's manifest is re-stamped each time.

### D5 — A French invariance baseline over a 13-page raw-text corpus and a fixture pack

**The corpus** (`crates/lingua-wasm/tests/baseline/pages-fr.txt`), in the Spanish corpus's shape —
`=== name` opens a page, each non-empty line is one block — thirteen pages, each written around
what a later change must show:

| Page | What it holds |
|---|---|
| `actualites` | news prose: dates, figures, acronyms, `au`/`du`/`des` |
| `fiction` | authored narrative with dialogue, « » quotes and the dash, set in real French punctuation: the typographic apostrophe `’` and the narrow no-break space (U+202F) before `?`, `!` and `»` |
| `proust` | the opening of Marcel Proust, *Du côté de chez Swann* (1913; public domain, see below): « Longtemps, je me suis couché de bonne heure. … » — elision-rich literary French, the Quijote page's counterpart, set as a French edition sets it (`’`, the narrow no-break space before `?`, `!` and `»`) |
| `homographes` | *porte*/*porter*, *est* (verb, east), *fils*, *couvent*, *vis*, *as*, *car*, *pas* (noun), *son*, *été* — M8's cases, one lemma each at this stage |
| `elisions` | `l'`, `d'`, `j'`, `n'`, `qu'`, `s'`, `c'`, `m'`, `t'`, `jusqu'`, `lorsqu'`, `puisqu'`, `aujourd'hui`, `presqu'île`, and the forms before a vowel (*bel*, *nouvel*, *vieil*, *cet*, *mon amie*) |
| `contractions` | `au`, `aux`, `du`, `des` in both readings (article and preposition + article) — M21 |
| `inversions` | `dit-il`, `y a-t-il`, `est-ce`, `va-t'en`, `donne-m'en`, `allez-vous-en`, `celui-ci`, beside true compounds (`peut-être`, `rendez-vous`, `arc-en-ciel`, `porte-monnaie`) |
| `noms` | the first sentence of Victor Hugo, *Les Misérables* (1862; public domain, see below): « En 1815, M. Charles-François-Bienvenu Myriel était évêque de Digne. », then titles (M., Mme, Dr), hyphenated given names and places (Jean-Pierre, Saint-Étienne), rivers and towns that are common words |
| `informel` | chat French: `jsuis`, `chuis`, `bcp`, `mdr`, `t'es où ?`, missing accents |
| `recette` | a recipe's imperatives (Épluchez, Faites chauffer, Laissez reposer) and quantities |
| `technique` | technical prose with anglicisms, acronyms and `l'API`, file names, a 404; one block committed in NFD (accents as combining marks, as a terminal paste or an OCR writes them) |
| `mixte` | blocks in English, Spanish, Catalan, Occitan and Italian around French ones — change 42's guard and the document vote |
| `grammaire` | verb forms: passé simple, subjonctif, participles with agreement, `ne … pas / jamais / plus`, `eût`, `pût`, `soyez` |

The two excerpts are the only quoted texts; every other line is authored for the corpus, as the
Spanish pages are. Their attribution and the public-domain reasoning live in `support/french.rs`'s
doc comment (the corpus file has no comment syntax): Proust died in 1922, so *Du côté de chez
Swann* has been in the public domain in France since 1987 under the 50-year term plus the wartime
extensions (which end around 30 September 1987), was not re-protected by the 1997 move to 70
years, and is pre-1929 in the United States; Hugo died in 1885. The Proust paragraph is split into
two blocks at « Et, une demi-heure après ».

**Real punctuation and one NFD block.** The `fiction` and `proust` pages are set as French is
printed — the typographic apostrophe `’` and the narrow no-break space (U+202F) before `?`, `!`
and `»` — so that M21's pieces (change 40) are exercised on the punctuation a reader's page has,
not on the ASCII a test author types: `push_word` already maps `’` to `'` for every language, and
the golden records what the segmentation makes of the narrow no-break space. One block of the
`technique` page is committed in NFD, its accents as combining marks: today French text is read as
it came (D2), so the golden records the decomposed tokens, and change 41's NFC rule shows its
effect as the before/after of that block in its re-bless. The scenario asserts the block is still
decomposed, so an editor or a formatter that silently composes the file fails the test rather
than moving the golden unnoticed. The fixture lists one word of that block, `mémoire` (glossed
`memory`), which the corpus has nowhere else: read as it came, the decomposed word is not the
pack's and shows no gloss, so change 41's re-bless shows NFC as a gloss that appears in the
golden. Without it the block's words, none of them in the pack, would be unglossed before and
after, and NFC would only move bytes a reviewer cannot see in the diff.

**The pack.** fr-en has no committed tables before change 48, and an engine holding en-fr refuses
a pack glossed in English. The scenario therefore runs over a **fixture pack** built from
hand-written tables in `scripts/lingua-data/testdata/fr-en/`, in the layout the other testdata
fixtures have (`forms.tsv`, `freq.tsv`, `gloss.tsv`, `mwe.tsv`, `level.tsv`, `manifest.json`,
`NOTICE`): about two hundred form→lemma pairs covering the corpus's common words (forms of
*être*, *avoir*, *aller*, *faire*, *pouvoir*, *dire*, *venir*, *prendre*; articles and pronouns
as plain forms; plurals and feminines; the single-letter words `a`, `à` and `y`, which the
tokeniser drops when the lexicon does not list them — `single_letter_outside_lexicon`), ranks for
about a hundred lemmas, about sixty English glosses, a handful of expressions, 53 levels (A1 to
C2) so that every probe of the harness answers (`seed-level`, the ladder), no grammar and no senses. The
manifest studies `fr`, is glossed in `en`, carries `FRENCH_ANALYZER_VERSION`, a pack version that
says « fixture », and no `levels_estimated` flag: the levels are a fixture's, not a decision (M7).
The NOTICE says the tables are hand-written for the tests. The lexicon is small on purpose: the
baseline's two branches — the pack's lemma, the lowercased form — both appear in the golden.

**Beside es-en.** The engine starts on the real es-en pack, built from its committed tables, and
adds fr-en: what an English-native reader's engine does once both are listed. The `beside es-en`
line moves when es-en's tables are re-pinned (change 21's two settings), as es-fr's `beside en-fr`
line does on an en-fr update; `lingua-pack-update` already tolerates a baseline that moves only on
its pack lines.

*Rejected — the engine on fr-en alone.* Simpler, but the profile an engine starts with would
differ from what a reader's engine has, and the Spanish baseline set the precedent of loading the
packs as the extension does.

**The probes** are the harness's, unchanged: pages for a new reader and for one with a history,
glosses, phrase glosses, word grammar, levels, the review, the exports, the backup. What the
scenario asserts beside the golden, in `french_baseline.rs`:
- the corpus holds the thirteen pages in order, each with a block, the reader's pages among them,
  and the `technique` page's NFD block (its fourth) still holds a combining mark (U+0301): NFC
  changes it, NFD does not;
- the NFD block's `mémoire` is unglossed today, while the pack glosses the composed form;
- French is the baseline: analysing the `elisions` page gives `l'homme` as one token whose lemma
  is `l'homme`, the `contractions` page gives `au` whole, no token of any page is a function word,
  and the analysis reports `0.1.0`;
- the engine holds es-en beside fr-en and reports English as its native language;
- the reader's backup begins with `"schema_version": 3` (D7).

**Hand-over.** When change 48 commits `tables/fr/` and `tables/fr-en/`, the scenario's pack source
switches to the committed tables and the golden is re-blessed once, in that pull request, which
says so; the fixture folder stays for `crates/lingua-pack`'s tests, or goes with it. Until then,
changes 40 to 47 re-bless the golden over the fixture, and the fixture's manifest follows the
analyser version. The harness loads each pack with the core before an engine is given it, so a
manifest left behind its language's analyser version fails with the core's reason and the
manifest to bump, not with wasm-bindgen's host panic (an engine's `JsError` cannot be built
outside wasm).

### D6 — The harness takes the pair's pack from a source the scenario names

`Scenario` gains `pack: PackSource`, an enum with two arms: `Tables` (the committed
`tables/<studied>/` + `tables/<pair>/`, as today) and `Testdata` (`scripts/lingua-data/testdata/
<pair>/` through `inputs_from_dir`). The pack line of the golden reads the manifest from the same
place. The English and Spanish scenarios say `Tables`; the beside packs are always built from the
tables. Nothing else in the harness changes, so the two existing goldens do not move.

*Rejected — building the fixture inputs in Rust, as `tests/languages.rs` does.* Two hundred
form→lemma pairs read better as a `forms.tsv`, the fixture then serves `crates/lingua-pack`'s
tests too, and the committed tables will replace it by a one-word change of the source.

### D7 — Backup schema version 3 is written only when the state names French, and read first

`BACKUP_SCHEMA_VERSION` becomes 3. `backup_version()` answers 3 when French is among the
profile's studied languages or among the languages any of `KnowledgeState`, `ExposureCounters`
and `Deck` holds; else 2 or 1 by today's rule. A backup is written in the oldest version that
holds it, so:
- an English reader's backup is version 1, byte for byte (S0 checks it);
- a Spanish reader's, or an English-native reader's, stays version 2, byte for byte;
- a French native language is not a trigger — every installed reader has it;
- French records under a Spanish profile, or a French profile with no record yet, are version 3.

The trigger is held by the files too, not only by the maps `backup_version` reads: every language
name of a backup holding a record in each per-language map, turned into French one at a time and
restored, is written again below version 3 only when it was the native language, so a
per-language map added later and forgotten by the trigger fails `tests/backup_format.rs`. That
test also pins `fixtures/backup-v3-french.json`, which a reader of versions 1 and 2 refuses as
unsupported version 3.

`from_backup` reads 1 to 3. The header-first read of `add-lingua-studied-language-profile` D3 is
what makes the released builds' behaviour exact: every build released since 1.5.0 (2026-10-03,
the first with the header-first read) reads `schema_version` before the rest and answers
`UnsupportedVersion { found: 3, supported: 2 }` to a version 3 file. It never deserialises
`"French"`, so it never says malformed; `hydrateEngine` throws, the surface does not start, the
store is untouched, and a later build reads the file. A build older than the header — 1.0.2 on
the Chrome Web Store, 1.2.1 on AMO, 1.4.0 — would report malformed, as it does of a version 2
backup holding Spanish records; those builds were superseded by the silent releases, which the
Spanish programme shipped for this very reason.

*Rejected — folding French into version 2.* A released build reads 2, deserialises, meets
`"French"` and reports malformed: the failure the architecture names.

*Rejected — writing version 3 for every non-default state from this build on.* A Spanish reader's
backup would stop being readable by the current release for no reason. « v2 only when needed » is
on the Spanish programme's never-cut list (`docs/lingua/spanish-programme.md`); v3 keeps the rule.

Change 12 (`add-lingua-native-language-sync-client`) rewords the same requirement's last sentence
(the profile is never sent « as such »). This change archives after it (`archiveAfter`) and its
MODIFIED block carries change 12's wording with the version sentence amended, so that whichever
order the archive workflow walks, the spec ends with both.

### D8 — What the extension and the agent do with a variant they do not ship

- `profile.ts` reads the stored backup's profile without an engine and drops a studied-language
  name it does not know (`NAMES` is `{ English, Spanish }`), English when none is known. Nothing
  writes `French` into a stored backup before change 52, which widens the extension's
  `StudiedLanguage` type, `NAMES` and the labels together. This change leaves them alone, so the
  labels lint and `Record<StudiedLanguage, …>` tables do not move.
- The engine's `set_studied_languages(["fr"])` would now be accepted for an English- or
  Spanish-native reader; no surface calls it with `fr` (Réglages offers the listed pairs'
  languages), and `readingLanguage` would keep reading in the shipped language, the profile left
  as it is (*Each surface reads in the reader's language*).
- The agent plugin follows the packs installed in `~/.lingua/`; `language_name(French)` says
  « Français » so the match stays exhaustive. No French pack is published; a fixture pack copied
  there by hand would be read by the baseline, which is what it is.

### D9 — Gates: the French golden blessed, the others untouched

1. The English and Spanish baselines — en-fr, es-fr and, since change 23, es-en — pass without
   re-blessing; a re-bless of any in this pull request is a review failure. `git diff --stat
   origin/main -- crates/lingua-wasm/tests/baseline/en-fr.golden crates/lingua-wasm/tests/baseline/
   es-fr.golden crates/lingua-wasm/tests/baseline/es-en.golden scripts/lingua-data/tables` is
   empty.
2. The French golden is blessed once, in this pull request, and committed.
3. `cross_native.rs` is unchanged: French's cross-native test needs two natives (48, 49).
4. The existing tests pass with French arms added and one expectation flipped by design: *French
   is not studied yet* in `profile.rs` becomes « a French-native reader cannot study French ».
5. `lingua-extension-check` runs the baselines, French's beside en-fr, es-fr and es-en (change 23's,
   kept as merged); `lingua-pack-update` re-blesses all four beside change 23's snapshots;
   `sonar-project.properties` adds `support/french.rs` to the copy-paste exclusions beside the
   English and Spanish scenarios.

OpenSpec: two ADDED requirements in `lingua-analysis`, three MODIFIED ones held by no open change
but one — *The backup records the reader's language profile*, held by change 12, hence
`archiveAfter`. `openspec_archive_order.py` exits 10 naming that change alone. The second ADDED
requirement, *A French invariance baseline runs beside the English and Spanish ones*, is a new
kind of requirement for this capability: it binds the tests and the pull requests that move the
golden, not the engine. It is kept because it states the rule every later French change obeys —
re-bless, bump the version, say what moved — where a reviewer of changes 40 to 48 will look for it.

## Risks / Trade-offs

- [A seam a third language must answer is missed] → The matches are exhaustive, so the crate does
  not compile; the extension's packs spec parses `language.rs` and fails first on the constant
  map; `ALL`, `tag()` and `from_tag()` round-trip in a test.
- [The golden is taken for French support] → `0.1.0`, the test's doc, the fixture's pack version
  and D2's list of what French lacks say otherwise; no pair studies French.
- [es-en's re-pin moves the `beside` line] → Accepted, as es-fr's `beside en-fr` line is; the
  update workflow's check tolerates a baseline that moves on its pack lines only.
- [A version 3 backup reaches a released build] → Only a test writes one before change 52; the
  build refuses it by name and leaves it; the spec states the behaviour.
- [`profile.ts` drops `French` from a stored profile] → Nothing writes it before change 52, which
  widens the type; the behaviour today (English when none is known) is the spec's.
- [The two excerpts] → Both are in the public domain: Proust died in 1922 (France since 1987, 50
  years plus the wartime extensions, which end around 30 September 1987; not re-protected in 1997;
  pre-1929 in the United States), Hugo in 1885; attributed, with that reasoning, in the scenario's doc. The owner may
  swap either before the golden is blessed (task 5.1).
- [An editor composes the NFD block] → The scenario asserts the `technique` page still holds a
  combining mark, so a silent NFC of the corpus file fails the test instead of moving the golden.
- [Thirteen authored pages take longer than planned] → The corpus is the top of the estimate's
  range; a page can be shorter than the Spanish one, the phenomena are what matter.

## Migration Plan

Nothing to migrate. No stored format moves for an existing reader; no wire field, pack byte or
pin changes. Rollback is a revert: a version 3 backup cannot exist outside tests before change 52.

## Open Questions

For the owner, none blocking:
1. The engine beside es-en (D5) rather than fr-en alone — es-en's re-pin will move one line of
   the French golden.
2. The two excerpts (Proust, Hugo) — any other public-domain text is as good; the owner may name
   one before the golden is blessed.
3. NFC left to change 41 (D2) — it could be the baseline's from day one, but it is a French rule,
   the spec's baseline has none, and the `technique` page's NFD block gives that change a visible
   before/after in its re-bless.
4. M21, M7, M8 and M6 stay open; the golden shows `au`/`aux`/`du`/`des` and every elision whole as
   « not yet decided », not as a decision.
