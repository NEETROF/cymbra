# add-lingua-italian-detection-guard — Catalan, Galician and Italy's regional languages are not read as Italian

## Why

Change 8 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Detection
runs per block through whichlang 0.1.1, which knows sixteen languages — Italian among them, and none
of Catalan, Galician, Occitan, Latin, Corsican, Sardinian, Sicilian, Neapolitan, Venetian or
Friulian. A block in a language it does not know is filed under the nearest class it has. Measured
with whichlang itself on one sentence per neighbour (design, Context): Catalan, Galician, Venetian,
Sicilian, Neapolitan, Sardinian, Corsican and Friulian sentences are read as Italian; Latin ones
land on French, English or Spanish and reach Italian seldom. The French baseline records a Catalan
sentence read as Italian and a `language.rs` unit test a Galician one, and the Italian corpus's
`misto` page shows its Catalan and Galician blocks analysed as Italian since change 5. While
Italian was not studied, that was harmless. The day it-fr ships (change 20), a French reader of
Italian who opens a Catalan or Galician page, a Sicilian song or a Venetian article would see it
counted, glossed by the Italian pack's nearest forms, and written into their statuses as Italian.

Spanish's guard refuses Catalan, Galician and Occitan (the Spanish programme's guard and matrix
change 42b); French's refuses Catalan, Occitan and Romanian (change 42). Italian needs the widest
guard of the three: its neighbours have no class of their own in the detector and share more of
its surface. This change adds it, modelled on French's — function words counted and compared, one
pooled neighbour table, measured word by word on real corpora before a word is kept, a recall gate
on Italian — and bumps Italian's analyser version.

## What Changes

- **A guard on Italian detection** (design D1). A block whichlang reads as Italian is not Italian
  when its neighbours' function words, counted together, outnumber its Italian ones; a tie, or no
  marker at all, stays Italian — French's rule. Two sorted tables, measured word by word (D3): the
  function words the neighbours write and Italian never does, and the Italian function words the
  neighbours rarely write.
- **The neighbours** (D2): Catalan and Galician (read as Italian, recorded), Venetian, Sicilian,
  Neapolitan, Sardinian, Corsican and Friulian (read as Italian on their probe sentences, D2's
  table), each kept in the pooled table when the measurement on its Wikipedia confirms whichlang
  reads at least a tenth of its blocks as Italian; Latin, Occitan and Romanian measured on the same
  rule, left out by default (Latin's probes land elsewhere; Occitan and Romanian have French's and
  Spanish's guards). Spanish, Portuguese and French have classes of their own and are not the
  guard's concern. The requirement names no list: the design does, and the measurement prunes it.
- **Read as French's guard reads a block** (D4): a neighbour's word counts only where it is written
  in lowercase (« El Niño », « Os Lusíadas » are names); a word joins across a hyphen, a middle dot
  or a full stop between letters; a run holding a digit is not a word; a decomposed word is composed
  before it is looked up; the apostrophe ends a word. One Italian addition, in a second entry point
  that leaves French's reading byte for byte: a word opened by an apostrophe — the Neapolitan
  elided articles `'o`, `'a`, `'e`, `'nu`, `'na` — is handed to Italian's guard with its apostrophe,
  as Occitan's `qu'`, `m'`, `t'` are Spanish's guard's markers.
- **Run for Italian alone** (D5): `detect` already applies the guard of the language whichlang
  found among the candidates; the Italian arm is added, English's (none), Spanish's and French's
  untouched, and the six goldens prove it.
- **A recall gate** (D6): on the Italian corpora, at most one Italian block in a thousand refused
  and none of the baseline's `pages-it.txt`; every refused block listed in the pull request with
  its reason; a word kept only when it catches neighbour text and refuses no Italian block that is
  not a quotation or a name.
- **Italian's analyser version bumps** (`1.0.0` → `1.1.0`, D7): a block the guard refuses is one
  the analysis no longer counts; the fixture's manifest follows; the Italian golden is re-blessed —
  the `misto` page's Catalan and Galician blocks leave the analysis (its Latin block too, if the
  golden shows whichlang read it as Italian), and change 7's baseline scenario is MODIFIED to say
  so.
- **Measured before merging** (D2, D3, D6): the share of each neighbour's Wikipedia blocks
  whichlang reads as Italian, with and without the guard; the Italian corpora's blocks refused; the
  script and its corpus list committed under `scripts/lingua-data/measure/`, unlike change 42's,
  whose samples and script were not.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *Catalan, Galician and Italy's regional languages are not read as
  Italian*; MODIFIED — *An Italian invariance baseline runs beside the English, Spanish and French
  ones* (change 7's text; its « what the baseline shows today » scenario says the Catalan and
  Galician blocks are excluded). The version bump moves no requirement: *Italian is a studied
  language served by its own analysis* (change 7) says « at or above `1.0.0` ». This change
  archives after changes 5, 6 and 7, whose golden, version and requirement it builds on.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core/src/analysis/language.rs`: the two Italian tables, `italic_neighbour` beside
  `iberian_neighbour` and `romance_neighbour`, a second entry point of `guard_words` for the
  apostrophe-opened word, `detect`'s Italian arm; `analysis/mod.rs` (`ITALIAN_ANALYZER_VERSION =
  "1.1.0"`); the guard's tests.
- `crates/lingua-wasm/tests/{italian_baseline.rs, support/italian.rs}` and `baseline/it-fr.golden`
  (re-blessed once); `scripts/lingua-data/testdata/it-fr/manifest.json`.
- `apps/lingua-extension/test/packs.spec.ts` (Italian's version by its pattern).
- `scripts/lingua-data/measure/` gains the guard's measurement script and its corpus list.
- `docs/lingua/italian-programme.md`: row 8 and the *Detection* bullet name the neighbours as D2
  settles them.
- Untouched: every committed table and pin, the six goldens, the Spanish and French guards and
  arms, the extension's bundle, the agent, the server, the site.
