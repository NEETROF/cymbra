## Why

Lingua's lemmatiser learns which form belongs to which word — *ran* → *run*, *children* →
*child* — from AGID, whose last release is **2016.01.19**. It has not seen a decade of English:
*smartphones*, *cryptocurrencies*, *influencers*, *apps*, *datasets*, *hashtags*, *bitcoins* are, to
it, words of their own. A reader who knows *smartphone* finds *smartphones* highlighted as unknown,
marks it again, and the percentage counts it twice.

AGID's maintainer moved inflections into **ESDB** (the English Speller Database, formerly
SCOWLv2): maintained (release `rel-2026.02.25`, commits in 2026), 1,800+ recent words, the same
permissive licence family. The product owner chose a maintained source over a frozen one.

Measured on 2026-09-25 — the extension's real engine on 160 English Wikipedia articles (177,902
tokens), the same raw sources, only the inflection source changed:

| vs AGID today | glosses shown | tokens resolved | tokens changing lemma |
|---|---|---|---|
| ESDB instead of AGID | +270 | +42 | 761 |
| AGID + kaikki `form_of` (regular inflections) | +324 | +15 | 354 |
| **ESDB + kaikki `form_of` (regular inflections)** | **+353 (+0.22 %)** | **+65** | **671 (0.38 %)** |

ESDB and the Wiktionary links kaikki already carries ("Pluriel de smartphone") each cover plurals
the other lacks; together they do best on every count.

## What Changes

- **ESDB replaces AGID** as the inflection source, taken at a pinned commit of `en-wl/wordlist`
  (`rel-2026.02.25`), its database built from that commit — the export is deterministic, measured —
  and checked by sha256, as `pin-lingua-pack-sources` pins every raw source.
- **kaikki's `form_of` links complete it**, for regular inflections only (`regular_inflection`
  already in the reducer): a Wiktionary entry saying a word is the plural or a verb form of
  another links them, when the spelling agrees.
- **Only primary and equal spellings count.** A form ESDB marks archaic, rarer or doubtful is not an
  inflection; US and UK spellings both are. Measured: keeping ESDB's lesser variants sends *born*
  to *bear* (123 tokens) and *art* to *be*.
- **The regressions the measurement found are fixed before release**, each with a test: *fewer*,
  *des*, *dis*, *renowned*, *vested*, *roses* — and the rule that today drops AGID's
  numbered variants by accident is made explicit.
- **No migration of readers' data in this change.** Lingua has no readers as of 2026-09-25 — it
  will certainly have some after that date. A status or a deck card keyed by a lemma that moves
  (*smartphones* → *smartphone*) is not carried over here; the update report lists every such move,
  and if readers are there when this lands, carrying their data over is decided then, as a change
  of its own.
- **The notices follow the source**: ESDB's copyright notice replaces AGID's in every pack and on
  the attributions page.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: **modified** — **Licence hygiene** (ESDB instead of AGID, its notice
  embedded; archaic and rarer variants are not inflections).

## Impact

**Products.** Cymbra Lingua only: every package's pack (Chromium, Firefox, Safari), the reducer and
the attributions page. The pack format, the builder and the core are unchanged. ID, Music, Live, the back office and the site are untouched.

**Consumed, not redeclared.** The pinned sources, committed tables, re-reduce and update modes of
`pin-lingua-pack-sources` — this change is applied through them, so its `forms.tsv` diff is
reviewed line by line. Knowledge stays keyed by `(language, lemma)`; sync, statuses and the deck are
untouched.

**Ordering.** Archives after `pin-lingua-pack-sources`, whose design records AGID as pinned and a
switch as a change of its own.

**Readers.** About 0.38 % of tokens get a different lemma, most of them merges of a plural into its
singular. No reader's statuses or deck exist to disturb as of 2026-09-25; readers are expected soon
after, so whether a carry-over is needed is checked when this lands.
