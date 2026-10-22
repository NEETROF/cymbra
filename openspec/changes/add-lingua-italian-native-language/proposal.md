# add-lingua-italian-native-language — Italian as a native language the core and the extension know, inert

## Why

Change 32 of the [Italian programme](../../../docs/lingua/italian-programme.md), the first of stage 3:
Italian speakers. Before an Italian interface, an Italian-glossed pack or a listing can exist, the core
and the extension must know Italian as a *native* language, as the matrix's changes 4 and 20 made the
native language a variable before a second one shipped. Today `NativeLanguage` is French, English and
Spanish (Italian was removed from it by #733); a pack glossed in `it` is refused at build and at load,
and a day counted under `it` by the Safari app is dropped. Measured on a scratch copy, appending the
variant breaks one exhaustive match outside the core and four tests that pin or use `it`; in the
extension, the native type is an alias of the interface type, so widening it alone would require the
Italian interface's 24 modules — change 33's.

## What Changes

- **The variant** (design D1): `NativeLanguage::Italian`, appended after Spanish, tag `it`; the agent's
  name for it; the tests that used `it` as an unknown native take `pt`; change 5's tests that hold
  Italian never native rewritten once both exist.
- **The extension's native type** (D2): `it` added; the interface type kept to French, English and
  Spanish until change 33, `INTERFACE_LANGUAGES` unchanged.
- **The guards and names** (D3): a native list for the offer, the native-language message, the stored
  record and reason, and the Safari app's day counts; the profile's name `Italian`; « Italiano » as
  Italian's own name in every interface.
- **Released before anyone can choose Italian** (D4): a build without the variant reads an Italian
  native's backup as malformed, so this change reaches every store before change 45.
- **Nothing a reader sees moves** (D5): no pair is glossed in Italian, so the offer stays French,
  English and Spanish.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-knowledge-model`: MODIFIED *L1/L2 profile* (held by no open change): a native language is
  French, English, Spanish or Italian; *An Italian native* added.
- `lingua-browser-extension`: ADDED *Italian is a native language the extension knows before it is
  offered*. `add-lingua-native-language-choice`'s *The reader chooses their native language* holds as
  written: it offers the natives a shipped pair is glossed in.

## Impact

- **Products.** Lingua only: `crates/lingua-core` (`knowledge/profile.rs`, its tests, `decks/backup.rs`'s
  test), `crates/lingua-pack` (one test), `apps/lingua-agent/rust/src/vocab.rs`, `crates/lingua-wasm`
  (nothing but the regenerated binding); `apps/lingua-extension` (`analyzer/types.ts`,
  `i18n/language.ts`, `state/native-language.ts`, `state/store.ts`, `state/app-relay.ts`,
  `state/profile.ts`, `analyzer/language-labels.ts`, `i18n/{fr,en,es}/languages.ts`, their tests). No
  backend, back office, site or Apple host change.
- **Order.** After changes 5, 15 and 20 (it rewrites change 5's tests, completes change 15's names,
  touches change 20's); before 33 (which restores the interface alias), 36 (which builds en-it) and 45.
  Released before change 45's release (D4).
- **Effort, against 1–2 ideal days**: 1–1.75 (design, *Effort*).
