# Design

## Context

See proposal.md — Why. What constrains the shape:

- `StudiedLanguage` is `English`, `Spanish`, `French`, in that order; the derived order keys the
  knowledge model's serialised maps, so a variant is appended, never inserted. Each variant names
  its whichlang class, its tag and its analyser version constant (`analysis/mod.rs`); `ALL` lists
  them; the extension's `tool/packs.mjs` maps each tag to its constant and `test/packs.spec.ts`
  holds that map to `language.rs`.
- Every seam that branches on the language — `nfc_for`, `split_contraction`, `lemmatize`,
  `is_function_word`, `document_names`, the detection's candidates, the baselines' harness — is an
  exhaustive match: a fourth variant must be answered at each, and the French baseline (matrix
  change 39) answered it verbatim for French with the baseline analysis the spec defines for a
  language whose rules are not written (`lemmatize_baseline`, no pre-pass, no table).
- whichlang knows Italian (`Lang::Ita`) and nothing of Catalan, Galician, Latin, Corsican or
  Sardinian, which it files under its nearest class — Catalan and Galician as Italian, as the
  French and Spanish baselines' comments record. Italian has no guard until change 8.
- `NativeLanguage` is `French`, `English`, `Spanish`; `studied()` maps a native language to the
  studied language of its tag. Italian is no native language before change 32.
- The backup's schema version is the oldest that holds the state: 1 for an English reader with
  French glosses, 3 when French is studied, 2 otherwise; `BACKUP_SCHEMA_VERSION` is 3; the restore
  reads the version first, so a build refuses a later version by name. `tests/backup_format.rs`
  pins the three fixtures and rotates every language name of a backup through the per-language
  maps.
- The harness (`crates/lingua-wasm/tests/support`) runs a scenario over a pack from a named source
  (`PackSource::Tables` or `Testdata`), with beside packs from the tables, and writes the golden
  under `LINGUA_BLESS=1`; `lingua-extension-check.yml` and `lingua-pack-update.yml` list the six
  baselines by test name. The French baseline ran over a hand-written fixture
  (`scripts/lingua-data/testdata/fr-en/`) until its tables were committed.
- Change 4 gives every testdata fixture a `tags.tsv` (the sorted union of its grammar's tags, or
  one tag when it has none) so that test packages build as parts; a fixture added after it follows.
- The agent plugin names each studied language in French (`language_name`); the extension's
  `StudiedLanguage` type is `en | es | fr`, read from the listed pairs.

## Goals / Non-Goals

**Goals:**

- Italian exists as a studied language with its own version, and no rule yet: every seam answers
  for it as the baseline analysis, and nothing of English, Spanish or French moves.
- A golden freezes what the engine makes of raw Italian text, page by page, so that changes 6 to
  8 and 16 each show their effect as a re-bless diff.
- A backup that holds Italian is refused by name by every build released since 1.5.0.

**Non-Goals:**

- Any Italian rule: elision pieces, articulated prepositions, enclitics, truncation, the
  apostrophe for an accent (change 6), NFC, the cascade, closed classes, names (change 7), the
  guard (change 8), tables (changes 9 to 14), the pack (16), the card (18).
- Italian as a native language (change 32), the interface (33).
- A listed pair: `packs.json` and the extension's bundle do not change.

## Decisions

### D1 — Italian is appended; serialised names and order stay

`StudiedLanguage::Italian` after `French`, tag `it`, `whichlang::Lang::Ita`, `ALL` of four,
`ITALIAN_ANALYZER_VERSION = "0.1.0"` in `analysis/mod.rs` with a doc comment saying what the
baseline lacks. Every stored state keeps its bytes: the maps keyed by language serialise the names
in declaration order, and no state names Italian yet.

### D2 — Italian's arm is the baseline analysis, exactly as the spec defines it

At each seam Italian takes the arm French took in change 39 before its rules: `nfc_for` returns
the text as it came; `split_contraction` answers `None`; `lemmatize` takes `lemmatize_baseline`
(the pack's lemma for the lowercased form, else the lowercased form); `is_function_word` and
`is_preposition` answer false; `document_names` answers none; detection names Italian among the
candidates a learner of Italian asks for and excludes it from the others'. So « l'uomo »,
« dell'acqua », « del », « dirmelo » and « po' » are one token each, lowercased, and the Spanish
accent retry, the French elision pieces and the English contractions never run on Italian.

*Alternative.* Writing the first Italian rules here: the golden would be born moved, and I7's
decisions, settled, deserve a change that shows each as a diff.

### D3 — Italian is no native language yet

`NativeLanguage` does not change: a pack glossed in `it` is refused as glossed in a native
language the core does not know (`UnknownNative`), as today; `NativeLanguage::studied()` keeps
mapping a native language to the studied language of its tag, and no native maps to Italian. The
pack-glossed-in-its-own-language rule cannot fire for Italian before change 32 adds the native;
the spec scenario says so.

### D4 — A version per language, read by the extension's constant map

`ANALYZER_CONSTANTS` in `tool/packs.mjs` gains `it: "ITALIAN_ANALYZER_VERSION"`;
`test/packs.spec.ts` holds the map to `language.rs`, so the variant fails the extension's tests
first, and the one-line addition passes them. Nothing else of the extension reads the variant: no
listed pair studies Italian, `StudiedLanguage` stays `en | es | fr`, no label names Italian.

### D5 — An Italian invariance baseline over a thirteen-page raw-text corpus and a fixture pack

**The corpus** (`crates/lingua-wasm/tests/baseline/pages-it.txt`), in the French corpus's shape —
`=== name` opens a page, each non-empty line is one block — thirteen pages, each written around
what a later change must show:

| Page | What it holds |
|---|---|
| `notizie` | news prose: dates, figures with the Italian separators (20.000; 3,5 %), acronyms (ONU, RAI, PIL, UE), articulated prepositions in passing |
| `narrativa` | authored narrative with dialogue, « » quotes and the dash, set in real Italian punctuation: the typographic apostrophe `’`, `È` with its accent |
| `manzoni` | the opening of Alessandro Manzoni, *I promessi sposi* (1840 edition; public domain, see below): « Quel ramo del lago di Como, che volge a mezzogiorno, … » — articulated prepositions, elisions and relative clauses in literary Italian, its first paragraph split into two blocks |
| `omografi` | *ancora* (anchor, still), *pesca* (peach, fishing), *legge* (law, reads), *porta* (door, carries), *sale* (salt, climbs), *vite* (lives, vine), *canto*, *era* — M8's cases, one lemma each at this stage |
| `elisioni` | `l'`, `un'`, `dell'`, `nell'`, `all'`, `dall'`, `sull'`, `c'è`, `dov'è`, `com'è`, `quest'`, `quell'`, `anch'io`, `senz'altro`, `tutt'altro`, and the truncations `po'`, `fa'`, `va'`, `di'` — I7 |
| `preposizioni` | `del`, `dello`, `della`, `dei`, `degli`, `delle` in both readings (preposition + article, partitive), `al`, `allo`, `alla`, `ai`, `agli`, `alle`, `dal…`, `nel…`, `sul…`, `col`, `coi` — I7 |
| `clitici` | `dirmelo`, `dammelo`, `vattene`, `andarsene`, `eccolo`, `farcela`, `parlargli`, `scrivimi`, beside words that end alike (*cinema*, *sistema*, *tema*, *vela*) — change 7's enclitic rule |
| `nomi` | the first lines of Carlo Collodi, *Le avventure di Pinocchio* (1883; public domain, see below): « C'era una volta… — Un re! — diranno subito i miei piccoli lettori. — No, ragazzi, avete sbagliato. C'era una volta un pezzo di legno. », then titles (Sig., Sig.ra, Dott., Prof., Ing.), compound given names, place names, a name after an elided piece (`l'Italia`, `dell'Europa`) |
| `informale` | chat Italian: `xké`, `cmq`, `nn`, `ke`, the apostrophe for an accent (`perche'`, `e'`, `piu'`, `citta'`, `E'`), missing accents (`perche`, `cosi`) — change 6's spellings |
| `ricetta` | a recipe's imperatives (Sbucciate, Fate soffriggere, Lasciate riposare, Aggiungete) and quantities (200 g, 1 l, q.b.) |
| `tecnico` | technical prose with anglicisms, acronyms and `l'API`, file names, a 404; one block committed in NFD (accents as combining marks, as a terminal paste or an OCR writes them) |
| `misto` | blocks in English, Spanish, French, Catalan, Galician and Latin around Italian ones — change 8's guard and the document vote; today the Catalan and Galician blocks are analysed as Italian |
| `grammatica` | verb forms: passato remoto (`fu`, `ebbe`, `disse`, `andò`), imperfetto, congiuntivo (`sia`, `fosse`, `abbia`), condizionale (`sarebbe`), gerundio, participles with agreement, `non … mai / più / ancora`, trapassato |

The two excerpts are the only quoted texts; every other line is authored for the corpus, as the
French pages are. Their attribution and the public-domain reasoning live in
`support/italian.rs`'s doc comment: Manzoni died in 1873 and Collodi in 1890, so both works have
been in the public domain in Italy for more than a century under the 70-year term and were
published before 1929, so they are in the public domain in the United States. The owner reads the
pages and may swap a text before the golden is blessed (task 5.1).

**Real punctuation and one NFD block.** The `narrativa` and `manzoni` pages are set as Italian is
printed — the typographic apostrophe `’`, « » quotes, accented capitals — so that change 6's
pieces are exercised on the punctuation a reader's page has; the `informale` page has the ASCII
apostrophe where an accent belongs, which no page of the French corpus needed. One block of the
`tecnico` page is committed in NFD; the scenario asserts it is still decomposed. The fixture lists
one word of that block, `città` (glossed « ville »), which the corpus has nowhere else: read as it
came, the decomposed word is not the pack's and shows no gloss, so change 7's NFC rule shows its
effect as a gloss that appears in the golden.

**The pack.** it-fr has no committed tables before change 16, and an engine holding en-fr accepts
a pack glossed in French. The scenario runs over a **fixture pack** built from hand-written tables
in `scripts/lingua-data/testdata/it-fr/`, in the layout the other testdata fixtures have
(`forms.tsv`, `freq.tsv`, `gloss.tsv`, `mwe.tsv`, `level.tsv`, `manifest.json`, `NOTICE`, and the
one-line `tags.tsv` change 4 gives a fixture without grammar): about two hundred form→lemma pairs
covering the corpus's common words (forms of *essere*, *avere*, *andare*, *fare*, *potere*,
*dovere*, *volere*, *dire*, *venire*, *prendere*; articles and pronouns as plain forms; plurals and
feminines; the single-letter words `a`, `e`, `è`, `o`, which the tokeniser drops when the lexicon
does not list them), ranks for about a hundred lemmas, about sixty French glosses, a handful of
expressions, 53 levels (A1 to C2) so that every probe of the harness answers, no grammar and no
senses. The manifest studies `it`, is glossed in `fr`, carries `ITALIAN_ANALYZER_VERSION`, a pack
version that says « fixture », and no `levels_estimated` flag: the levels are a fixture's, not a
decision (I8).

### D6 — The harness takes the pair's pack from a source the scenario names

The scenario names `PackSource::Testdata` for it-fr and `en-fr` beside it from the tables: the
engine starts on the real en-fr pack, as a French-native reader's engine does, and adds the
fixture. Change 16 switches the source to the tables and re-blesses once, as change 48 did for
French. The golden's header lines name each pack's source, size and versions, as the others' do.

### D7 — Backup schema version 4 is written only when the state names Italian, and read first

`BACKUP_SCHEMA_VERSION` becomes 4. `backup_version()` answers 4 when Italian is among the
profile's studied languages or among the languages any of `KnowledgeState`, `ExposureCounters`
and `Deck` holds; else 3, 2 or 1 by today's rule. A backup is written in the oldest version that
holds it, so an English reader's stays 1, a Spanish or English-native reader's 2, a French
reader's 3, byte for byte; Italian records under any profile are version 4. The trigger is held
by the files too: `tests/backup_format.rs` rotates every language name through the per-language
maps and pins `fixtures/backup-v4-italian.json`, which a reader of versions 1 to 3 refuses as
unsupported version 4. `from_backup` reads 1 to 4. Every build released since 1.5.0 reads the
version before the rest and answers `UnsupportedVersion { found: 4, supported: 3 }` — never
malformed; `hydrateEngine` throws, the surface does not start, the store is untouched.

*Rejected — folding Italian into version 3.* A released build reads 3, deserialises, meets
`"Italian"` and reports malformed: the failure the architecture names.

### D8 — What the extension and the agent do with a variant they do not ship

Two compile-forced lines: D4's constant, and the agent's `language_name(Italian)`, « Italien »,
in the plugin's French copy. The agent follows the packs installed in its directory; no Italian
pack exists, so nothing changes for it.

### D9 — Gates: the Italian golden blessed, the others untouched

`italian_baseline` is blessed once in this change and listed where the six others are
(`lingua-extension-check.yml`'s baselines step, `lingua-pack-update.yml`'s re-bless step — an
en-fr update moves its `beside en-fr` line alone). The six goldens and S0 pass without
re-blessing; every pin is unchanged; `cross_native` and `languages` pass.

## Risks / Trade-offs

- [whichlang files Catalan, Galician, Latin and the regional languages under Italian] → recorded
  by the `misto` page's golden; change 8's guard moves it before any Italian pack ships.
- [An excerpt's public-domain status] → both authors died more than seventy years ago and both
  works predate 1929; the owner reads the attribution (task 5.1).
- [The fixture's one-tag pin] → harmless before change 4 (no grammar, nothing reads it) and what
  change 4 requires after; the golden holds no grammar probe.
- [Backup v4 refused by an installed build] → by name, never as malformed, as v3 is; no reader
  holds Italian data before a pair ships.

## Migration Plan

Nothing deploys and nothing is released; no listed pair studies Italian, so no package changes
beyond two compile-forced lines. Rollback is a revert.
