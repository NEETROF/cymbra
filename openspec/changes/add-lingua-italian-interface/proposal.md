# add-lingua-italian-interface — the extension's interface written in Italian, before it is shown

## Why

Change 33 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3: Italian
speakers. The extension's interface is one typed catalogue per language — 24 modules each, French the
source, English and Spanish typed after it (the matrix's changes 13–17). Italian speakers need a fourth
set, written in I6's Italian (settled 2026-10-23: « tu », « QCER », Treccani's tense names, Italian
numbers), and reviewed by the dedicated Italian agent (I2). Measured on a simulated set: 3,631 French
words in 803 strings to translate across 23 text modules; Intl writes Italian numbers, dates and plurals
as I6 wants, but `formatPercent` would write « 110 % » for Italian today, unseen by the compiler; the one
message built from a contraction (« dall'inglese / dallo spagnolo ») needs a helper, as Spanish's `de()`;
change 32's split of the interface type from the native type can now be undone.

## What Changes

- **23 Italian text modules** (design D1), `typeof fr`, from the French, Music's Italian terms and the
  Spanish draft's choices; `languages` with Italian's articles and `QCER`; an Italian README section.
  `grammar.ts` is change 34's (D2).
- **The format helpers** (D3): `LOCALES.it`, Intl's Italian numbers and plurals, an Italian branch in
  `formatPercent` (« 96% »).
- **Elision, contraction and gender** (D4): a `da()` helper, dates and hours worded without an article,
  gender-free address, `READ_AS_NOUN.it`.
- **The interface alias restored** (D5), change 32's mapping dropped; the privacy and leaving pages
  English until the site's Italian ones.
- **Inert until change 45** (D6): `INTERFACE_LANGUAGES` unchanged.
- **Lints and tests** (D7); **the Italian reviewer** defined (`.claude/agents/lingua-italian-reviewer.md`)
  and its pass in the pull request (D8).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-interface-language`: ADDED *The Italian catalogue writes as Italian does, before it is shown*.
  MODIFIED *The interface's copy is kept in one typed catalogue per language* (held by no open change):
  Italian modules typed after the French, its entries held whole; *A count in Italian* added. *The
  interface language is the reader's native language, under its own key* holds as written until change
  45 adds Italian to the languages shown.

## Impact

- **Products.** Lingua's extension and the repository's agents: `.claude/agents/lingua-italian-reviewer.md`;
  `apps/lingua-extension/src/i18n/it/` (23 modules),
  `src/i18n/{index.ts, language.ts, README.md}`, the 18 files of design *Measured*, `state/store.ts`
  (change 32's mapping removed), the tests and lints of D7. No site, Apple host, backend or pack change.
- **Bundles**: ≈ +130–140 kB across entries before anything shows.
- **Order.** After change 32; before changes 34 (the Italian renderer), 39–41 and 45. Archived after the
  changes of `.openspec.yaml`.
- **Effort, against 4–7 ideal days**: 4–7 (design, *Effort*).
- **The owner**: Open Question 1; the reading on the five targets from change 45 (I2).
