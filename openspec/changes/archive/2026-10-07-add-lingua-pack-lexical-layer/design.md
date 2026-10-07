# Design — add-lingua-pack-lexical-layer

## Context

See proposal.md (Why). Measured on origin/main 7ed3da03, in a scratch copy:

| Where | What it does |
|---|---|
| `crates/lingua-core/src/packs/pack.rs` (`dictionary_words`, ~422-437) | A dictionary word is a ranked lemma with a CEFR level or a gloss: `rank != 0 && (leveled \|\| glosses.contains_key(id))`. It is the universe of `vocabularyEstimate` and of a CEFR-list ladder's typical figures (Spanish's estimated ladder reads the frozen constant since change 4). en-fr: 25,372 = 24,799 glossed + 573 levelled unglossed. es-fr: 22,755, all glossed. |
| `crates/lingua-core/src/engine.rs` (`document_names`, ~178-198) | Spanish only. A capitalised mid-sentence form never written in lowercase is set aside when `pack.gloss(lemma).is_none()`. |
| `crates/lingua-core/src/packs/format.rs`, `pack.rs::load` | `read_container` accepts any section name, and `load` looks sections up by name, so an unknown section is never read and old cores load a pack that carries one. `FORMAT_VERSION` is 1, and any other value is refused. |
| `crates/lingua-pack/src/lib.rs` (`grammar_sections`, ~505-541; `encode_tag_pool`, ~643) | One sorted `BTreeSet` of reading tags ∪ sense-run tags. `paradigms.zst` stores indices into it. en-fr: 27 tags (13 reading + 14 sense-only). es-fr: 106 (89 + 17). |
| `crates/lingua-pack/src/lib.rs` (`extra`, ~340-356) | Glossed lemmas outside forms ∪ ranks are added to the lemma pool, so a gloss set can move lemma ids. The real tables have none. The testdata en-fr fixture has three (`leaf`, `can`, `put`), on purpose. |
| `scripts/lingua-data/reduce-es-fr.py` (`noun_class_runs`, ~506-523) | Writes `NOUN\|Gender=X` into es-fr's sense runs. Deriving it from `grammar.tsv` (the noun read as itself, exactly one gender) reproduces all 13,435 runs. |
| Baselines | `english_baseline` and `spanish_baseline` probe the universe, the ladder, names and classes. Both pins are reproduced by the builder. |

Every gloss read that attaches or shows the reader's-language gloss (token and phrase glosses, the
card, sense groups, expressions, seeded cards) stays native-side and is not touched.

## Goals / Non-Goals

**Goals:**
- Analysis, names, levels, the vocabulary estimate and the stored readings depend on the studied
  tables alone.
- en-fr and es-fr keep their bytes; both baselines pass without re-blessing.

**Non-Goals:**
- A reference set drawn from the studied language alone (for example kaikki's non-name entries), or
  leaving names out of the dictionary words (3,046 en-fr and 1,693 es-fr dictionary words are glossed
  only as proper nouns). Either would move both baselines; that is a change of its own, with a
  re-bless.
- The reducers. Editing one moves the rule digest and every pin; generalising them is change 6
  (`generalise-lingua-gloss-reducer`). Moving the studied tables to `tables/<studied>/` is change 7.
- Any pack that uses the table: es-en and en-es (changes 21, 22).

## Decisions

### D1 — An optional `lexical` section, written only when it says something

`section::LEXICAL`:
- It is one bit per lemma id, least significant bit first, `ceil(lemma_count / 8)` bytes: 5,086 B
  for English, 7,500 B for Spanish. zstd saves only 3–7 %, so the bitset stays raw, with an O(1)
  lookup.
- A section of the wrong length is `PackError::Malformed("lexical")`.
- `FORMAT_VERSION` stays 1, because a bump would make every older core refuse the pack.
- No metadata field is added: the section's presence is the signal.

The builder reads an optional `lexical.tsv` (lemma strings, one per line, byte-sorted) and writes
the section only when its bits differ from the pack's own glossed lemmas. A `lexical.tsv` that
repeats the glosses therefore builds byte for byte the pack built without it; measured on en-fr and
es-fr.

Alternatives:
- **Lemma ids in the file.** They are the builder's positions in a sorted pool and move whenever a
  lemma does.
- **Always writing the section.** It adds 5–7 KB to en-fr and es-fr and moves both pins, for no
  change in output.

### D2 — `is_dictionary_word`, with gloss presence as the fallback

`Pack::is_dictionary_word(lemma)` reads the bit when the section is there, and gloss presence when
it is not. Both uses are rewired:
- `dictionary_words` becomes `rank != 0 && (leveled || is_dictionary_id(id))`;
- `document_names` reads `!pack.is_dictionary_word(lemma)`, the bit alone, as it read gloss
  presence alone. On es-fr the two predicates agree, since no Spanish lemma is levelled and
  unglossed.

With no section, every output is today's, so no analyser version moves. A core older than this
change ignores the section and falls back to the glosses. Extension packs always ship with their
own core. Only an agent plugin user who copies a newer pack into an older plugin would see the old
rule.

Alternative: bump the analyser versions. Every analysed probe of both baselines would move, for no
behaviour change on the shipped packs.

### D3 — The reference is the first pack's glossed lemmas

A studied language's dictionary words are the lemmas its reference pack glosses: en-fr for
English, es-fr for Spanish, and for a later language its first pack. A non-reference pair's
`lexical.tsv` copies its reference's gloss lemma column, and a test over the committed tables holds
the copy to its source (« two packs of one language disagree »). The same test holds the pair's
studied tables — `forms.tsv`, `freq.tsv`, `level.tsv`, `grammar.tsv` — byte-equal to the
reference's, and its `tags.tsv` to the reference's pin: with the dictionary words, they decide the
lemma ids and how a form's readings are stored, so a pair that copied the wrong snapshot fails
there, naming both pairs and the table.

Stage 1 ships no `lexical.tsv` at all: no non-reference pair exists yet. Lemmas a new native
glosses and the reference does not are not dictionary words. For es-en, that is `augusto` among
10,259 such lemmas: they keep their card glosses, but they count in no vocabulary size and can be
set aside as names. A native-independent union can come later, when the reference packs move
anyway.

The builder refuses a dictionary word or a glossed lemma outside forms ∪ ranks only when it writes
the table. The testdata fixture's deliberate outliers (`leaf`, `can`, `put`) stay legal, and real
tables have none.

### D4 — A pinned tag pool, ordered rather than refused

A pair whose tables carry a `tags.tsv` builds its pool in this order:
1. the pin, in its own order;
2. the tags of readings outside it, sorted;
3. the tags only senses carry, sorted.

Readings, and so `paradigms.zst`, are then stored alike whatever tags the senses use. The en-fr
and es-fr pins are today's pools, committed by hand beside their tables. Measured: both rebuild
their pinned sha256.

A pack built without a `tags.tsv` keeps the single sorted pool. That covers the testdata packs, so
their fixtures do not move.

Refusing rather than ordering was rejected:
- Refusing sense tags outside the pin would block en-es, whose senses use `NUM` 19 times while
  en-fr's pool has no `NUM`.
- Refusing reading tags outside it would break the archived scenario *A Romance pack fits the
  vocabulary*, whose test pack reads `me` with a tag es-fr lacks.

`tags.tsv` is an input that no reducer writes: `build.sh` and `lingua-pack-update` must keep it
when they re-reduce a pair, so no rule digest moves.

### D5 — A noun's gender from its readings

For every noun sense run, the builder looks up the readings of the run's lemma read as itself:
- if they give exactly one gender, it writes `NOUN|Gender=X`;
- if a run already carries a gender those readings contradict, it refuses the build, naming the
  noun;
- if they give both genders, it writes a bare `NOUN`, whatever gender the run names;
- if they give none, it keeps the run as the sense table writes it: bare from a reference pair's
  reducer, which writes no gender without a gendered reading. A gendered run is kept too, because
  the archived scenario *A Romance pack fits the vocabulary* reads `leche`'s feminine from its run
  alone, with no reading of `leche`. Only a sense table that genders a noun its readings do not
  gender can then make the card's gender depend on the native side, and no reducer writes one.

Measured: es-fr's 13,435 runs are reproduced exactly (byte-neutral), and en-fr has no gendered
readings. An es-en reducer then gets the gender without repeating `noun_class_runs`, which change 6
can remove from the reducer.

### D6 — The invariance test

`crates/lingua-wasm/tests/cross_native.rs`, host only. For English and for Spanish, it builds the
reference pack from the committed tables, and a second pack over the same studied tables:
- native metadata of another language;
- about 70 % of the glosses kept (a deterministic hash), plus glosses on lemmas the reference does
  not gloss;
- fewer expressions and senses;
- one sense tag no reading uses;
- the reference's `lexical.tsv` and `tags.tsv`.

It answers every probe of the language's baseline scenario through both packs, strips glosses,
senses and expressions, and requires byte equality. It also requires byte-equal `forms`, `lemmas`,
`freq`, `levels`, `tags` (up to the pinned prefix and reading tags) and `paradigms.zst`.

The baselines' own goldens are not touched.

## Risks / Trade-offs

- **[A reference pair's update must be copied to every other native's tables]** — its gloss lemma
  column to their `lexical.tsv`, its studied tables to theirs. → Nothing copies them automatically:
  the committed-tables test fails the update's pull request on a mismatch, naming both pairs, and a
  person copies the files in that pull request. The risk is latent: no non-reference pair exists in
  stage 1, and change 7 (stage 1) moves the studied tables and `lexical.tsv` to
  `tables/<studied>/` before change 21 adds the first one, ending the copies.
- **[`tags.tsv` is lost when a pair is re-reduced.]** → `build.sh` and `pack_sources.py` treat it as
  a kept input. A test fails if a committed pair lacks it.
- **[An older core reads a new pack's dictionary words from its glosses.]** → Only the agent plugin
  can meet that mismatch. The fallback is graceful and documented.
- **[Proper-noun-only glosses count as dictionary words]** (`london`, `madrid`). → This is today's
  behaviour, kept on purpose (Non-Goals).

## Migration Plan

None: no stored state, wire format, pack byte or analyser version changes. Rollback is reverting
the change.
