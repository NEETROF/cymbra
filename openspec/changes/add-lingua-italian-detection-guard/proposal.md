# add-lingua-italian-detection-guard — Catalan, Galician, Latin and the regional languages of Italy are not read as Italian

## Why

Change 8 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Detection
runs per block through whichlang, which knows sixteen languages — Italian among them, and none of
Catalan, Galician, Occitan, Latin, Corsican, Sardinian, Sicilian, Neapolitan, Venetian or Friulian.
A block in a language it does not know is filed under the nearest class it has, and for most of
these that class is Italian: the French and Spanish baselines record Catalan and Galician
sentences read as Italian, and the Italian corpus's `misto` page shows its Catalan and Galician
blocks analysed as Italian since change 5. While Italian was not studied, that was harmless. The
day it-fr ships (change 20), a French reader of Italian who opens a Catalan or Galician page, a
Latin quotation, a Sicilian song or a Venetian article would see it counted, glossed by the
Italian pack's nearest forms, and written into their statuses as Italian.

Spanish's guard refuses Catalan, Galician and Occitan (matrix changes of the Spanish programme and
42b); French's refuses Catalan, Occitan and Romanian (change 42). Italian needs the widest guard of
the four: its neighbours have no class of their own in the detector and share more of its surface.
This change adds it, modelled on French's — function words counted and compared, one pooled
neighbour table, measured word by word on real corpora before a word is kept — and bumps Italian's
analyser version.

## What Changes

- **A guard on Italian detection** (design D1). A block whichlang reads as Italian is not Italian
  when its neighbours' function words, counted together, outnumber its Italian ones; a tie, or no
  marker at all, stays Italian — French's rule. Two sorted tables, measured word by word (D3): the
  function words the neighbours write and Italian rarely does, and the Italian function words they
  rarely write.
- **The neighbours** (D2): Catalan and Galician (measured as read as Italian today), Latin, and the
  regional languages of Italy with a Wikipedia of their own — Sicilian, Neapolitan, Venetian,
  Sardinian, Friulian, Corsican — each kept in the pooled table only if the measurement shows
  whichlang reads a worthwhile share of it as Italian; Occitan and Romanian measured and kept or
  left on the same rule; Spanish, Portuguese and French have classes of their own and are not the
  guard's concern.
- **Read as French's guard reads a block** (D4): a neighbour's word counts only where it is written
  in lowercase (« El Niño », « Los Angeles », « Lo » opening a Sicilian title are names); a word
  joins across a hyphen, a middle dot or a full stop between letters; a run holding a digit is not
  a word; a decomposed word is composed before it is looked up; the apostrophe ends a word, and the
  Neapolitan elided articles (`'o`, `'a`, `'e`, `'nu`, `'na`) are read with their apostrophe as
  markers, as Occitan's elisions are in Spanish's guard.
- **Run for Italian alone** (D5): `detect` applies the guard of the language whichlang found among
  the candidates; English's, Spanish's and French's are untouched, and the six goldens prove it.
- **Italian's analyser version bumps** (`1.0.0` → `1.1.0`, D6): a block the guard refuses is one
  the analysis no longer counts; the fixture's manifest follows; the Italian golden is re-blessed —
  the `misto` page's Catalan, Galician and Latin blocks leave the analysis.
- **Measured before merging** (D3, D7): recall on Italian corpora (UD Italian-PUD and MarkIT,
  Tatoeba's Italian export, Italian Wikipedia articles, public-domain books), the share of each
  neighbour still read as Italian with and without the guard, the Italian blocks refused, each
  named in the pull request.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *Catalan, Galician, Latin and the regional languages of Italy are not
  read as Italian*. The version bump moves no requirement: *Italian is a studied language served by
  its own analysis* (change 7) says « at or above `1.0.0` ». This change archives after changes 5,
  6 and 7, whose golden and version it builds on.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core/src/analysis/language.rs`: the two Italian tables, `italic_neighbour` beside
  `iberian_neighbour` and `romance_neighbour`, `detect`'s Italian arm; `analysis/mod.rs`
  (`ITALIAN_ANALYZER_VERSION = "1.1.0"`); the guard's tests.
- `crates/lingua-wasm/tests/{italian_baseline.rs, support/italian.rs}` and `baseline/it-fr.golden`
  (re-blessed once); `scripts/lingua-data/testdata/it-fr/manifest.json`.
- `apps/lingua-extension/test/packs.spec.ts` (`coreAnalyzerVersion("it", modRs)` reads `1.1.0`).
- `scripts/lingua-data/measure/` gains the guard's measurement script and its corpus list, as
  change 42's did.
- Untouched: every committed table and pin, the six goldens, the English, Spanish and French
  guards and arms, the extension's bundle, the agent, the server, the site.
