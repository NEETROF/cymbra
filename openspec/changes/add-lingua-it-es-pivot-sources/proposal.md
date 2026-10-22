# add-lingua-it-es-pivot-sources — the further sources it-es reads: a pivot of seven Wiktionaries, the Italian Wiktionary's Spanish section, Wikidata Lexemes

## Why

Change 27a of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2. it-es — Italian
studied by Spanish speakers — measured 57.8 / 46.8 / 35.0 % of the 5,000 / 10,000 / 20,000 commonest
Italian lemmas glossed with its three planned sources, about 25 points under fr-es, the thinnest pair
shipped. The owner had new sources researched; none written directly for the pair reaches fr-es's floor,
and on 2026-10-23 (I4, I11, #1004) the owner chose the full pivot: an Italian and a Spanish word listed
under the same sense of another Wiktionary's translation table, every pair kept, with Wikidata Lexemes and
the Italian Wiktionary's Spanish section — 84.8 / 76.3 / 65.8 together. This change brings those sources
into the pipeline, derived and pinned; change 27 reads them into it-es's glosses.

## What Changes

- **A pivot file per edition** (design D1): `derive` gains a kind that keeps a translation table's sense
  with its Italian and its Spanish words; the English and French editions gain one file each, and the
  German, Portuguese, Russian, Polish and Dutch Wiktionaries join `EDITIONS`, one file each.
- **The Italian Wiktionary's Spanish section** (D2), in `EDITIONS["it"]`.
- **Wikidata Lexemes, mirrored** (D3): a dated dump published as a release asset, pinned, its Italian–Spanish
  sense pairs derived.
- **`DUMPS["it-es"]` and the pin** (D4); **the cost** recorded (D5): five more dumps per it-es update.
- **Nothing read yet**: no pack, table or other pair moves until change 27.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *it-es's further sources are derived and pinned like every source*.

## Impact

- **Products.** Lingua's data pipeline only: `scripts/lingua-data/pack_sources.py` (`derive`'s pivot
  kind, `EDITIONS`, `PINNED["it-es"]`, `DUMPS["it-es"]`), `test_pack_sources.py`, `SOURCES.md`. No pack,
  table, extension, site or backend change.
- **The owner**: the release asset mirroring the Wikidata Lexemes dump (D3).
- **Order.** After change 16 (`EDITIONS["it"]`) and change 22; before change 27, which reads these files.
- **Effort**: 2.5–4 ideal days (design, *Effort*).
