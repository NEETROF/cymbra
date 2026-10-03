# add-lingua-spanish-analysis — Spanish's own tokenisation and lemmatisation

## Why

Spanish is analysed by the baseline since `generalise-lingua-analysis-by-language`: segmentation, the
rules that belong to no language, and the pack's form→lemma lookup, at analyser version `0.1.0`.
Real Spanish text needs more:
- **Contractions.** `al` and `del` are `a` + `el` and `de` + `el`. Each is two words, as `don't` is
  for English.
- **Enclitics.** Pronouns attach to infinitives, gerunds and imperatives: `dámelo`, `diciéndole`,
  `levántate`, `vámonos`. The combinations are endless, so no forms table can list them; and the
  attachment moves the written accent.
- **Old spellings.** Books in the public domain write `fué`, `dió`, `á`, `sólo`, `éste`. Today's
  forms tables drop those accents.
- **Plurals of words the pack does not list.** `luces`/`luz` and `ciudades`/`ciudad` would count
  as two words.
- **Closed classes.** A word-by-word gloss in Spanish glosses `de`, `el` and `que`, which carry
  nothing on their own.
- **Text encoding.** Text may come decomposed (a vowel then a combining accent). The pack is
  precomposed.

This is change 18 of `docs/lingua/spanish-programme.md`, the first change of G1, the internal Spanish
build. The rules are written now. The forms tables they run against come with
`add-lingua-spanish-forms-tables`, which measures them on real text. No reader studies Spanish yet:
English is untouched, and its baseline is byte for byte the same.

## What Changes

- **The Spanish pre-pass.** Text is normalised to NFC. `al` and `del` split into `a` + `el` and
  `de` + `el`, sharing their source span, with the first letter's case kept, as `don't` does. The
  baseline's rules stay as they are.
- **The Spanish cascade.** It runs in this order:
  1. the pack's forms;
  2. the same form without its acute accents (old spellings);
  3. the enclitic rule;
  4. a plural fallback for forms outside the lexicon;
  5. the form itself.
- **The enclitic rule.** It is a rule in the core, never a table, and it runs only when the whole
  form is not in the lexicon. It strips at most two clitic pronouns, then requires:
  - a base the lexicon lists, shaped to take them (an infinitive, a gerund, or another verb form of
    two syllables or more);
  - a monosyllabic imperative from a closed list;
  - a written accent where the stress shift puts it.

  It also restores the `-s` of `vámonos` and the `-d` of `sentaos`.
- **The plural fallback.** It covers `-ces` → `-z`, `-iones` → `-ión`, `-es` after a final
  consonant, and `-s` after a vowel. Singulars ending in `-is` or `-us` are left as they are.
- **Spanish closed classes** for the word-by-word gloss: determiners, pronouns, prepositions,
  conjunctions, auxiliaries and negation.
- **Analyser version `1.0.0` for Spanish.** English stays `1.1.0`.
- **Fixtures.** More than 100 Spanish fixtures, one per rule and per guard.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`:
  - ADDED: *Spanish tokenisation pre-pass*, *Spanish lemmatisation cascade*, *Spanish
    closed classes*.
  - MODIFIED: *Analysis by studied language*, whose scenarios described Spanish as the baseline
    language. No open change holds it.

## Impact

- **Products.** Cymbra Lingua's analysis core, `crates/lingua-core`:
  - `analysis/tokenize.rs`, `lemmatize.rs` and `function_words.rs`;
  - the Spanish version in `analysis/mod.rs`;
  - a dependency on `unicode-normalization`, already in the workspace and WASM-clean.

  The extension, the agent and the back office consume the core unchanged. No pack, server or proto
  change. ID, Music, Live and the site are not affected.
- **Release.** G1, internal. No reader studies Spanish, and the English baseline (S0) is unchanged.
