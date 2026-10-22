# add-lingua-it-es-pivot-sources — the further sources it-es reads: a pivot of seven Wiktionaries, the Italian Wiktionary's Spanish section, Wikidata Lexemes

## Why

Change 27a of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2. it-es — Italian
studied by Spanish speakers — measured 57.8 / 46.8 / 35.0 % of the 5,000 / 10,000 / 20,000 commonest
Italian lemmas glossed with its three planned sources, about 25 points under fr-es, the thinnest pair
shipped. The owner had new sources researched; none written directly for the pair reaches fr-es's floor,
and on 2026-10-22 (I4, I11, #1004) the owner chose the full pivot: an Italian and a Spanish word listed
under the same sense of another Wiktionary's translation table, or the same entry when the table
names none, every pair kept, with Wikidata Lexemes and
the Italian Wiktionary's Spanish section — 84.8 / 76.3 / 65.8 together. This change brings those sources
into the pipeline, derived and pinned, with the three planned files no edition registers yet; change 27
reads them into it-es's glosses.

## What Changes

- **The planned three registered** (design D1): the Spanish Wiktionary's Italian section and Italian
  translations, the Italian Wiktionary's Spanish translations, in their editions' catalogues.
- **A pivot file per edition** (D2): `derive` gains a kind that keeps a translation table's Italian and
  Spanish words together under the sense the table names — its label, else its number, else the whole
  entry, marked so —; the English and French editions gain one file each, and the German, Portuguese,
  Russian, Polish and Dutch Wiktionaries join `EDITIONS`, one file each.
- **The Italian Wiktionary's Spanish section**, and **Wikidata Lexemes read as an edition of its own**
  (D3), through a reader of its own: the dated dump that `latest-lexemes.json.gz` serves recorded, its Italian–Spanish sense pairs derived and published
  in it-es's sources release.
- **`DUMPS["it-es"]` and the pin** (D4); **the cost** recorded (D5): ≈ 5.3 GB of dumps per it-es update,
  none cached, a re-reduction reading the release.
- **What change 27 takes from here** (D6): the measured filters and order, the decisions left open, the
  credits, and the main spec's no-pivot requirement to MODIFY for it-es.
- **Nothing read yet**: no pack, table or other pair moves until change 27.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *it-es's further sources are derived and pinned like every source*.

## Impact

- **Products.** Lingua's data pipeline only: `scripts/lingua-data/pack_sources.py` (`derive`'s pivot
  and lexeme kinds, `EDITIONS`, `DUMPS["it-es"]`), `test_pack_sources.py`, `SOURCES.md`. No pack, table,
  extension, site or backend change.
- **Order.** After change 16 (`EDITIONS["it"]`) and change 22; before change 27, which reads these files.
- **Effort**: 2.5–4 ideal days (design, *Effort*).
