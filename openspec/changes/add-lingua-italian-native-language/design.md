# Design — add-lingua-italian-native-language

## Context

Change 32 of the [Italian programme](../../../docs/lingua/italian-programme.md), the first of stage 3:
Italian speakers. Before an Italian interface (change 33), an Italian-glossed pack (change 36) or a
listing (change 45) can exist, the core and the extension must know Italian as a native language —
inert, as the matrix's `generalise-lingua-native-language` (change 4) and
`add-lingua-native-language-choice` (change 20) made the native language a variable before a second
one shipped. Stages 1 and 2 add Italian as a *studied* language (change 5's variant, change 20's
extension type); nothing yet knows it as a native one.

## Goals / Non-Goals

**Goals:**
- `NativeLanguage::Italian` in the core and `it` in the extension's native type, every enumeration
  following, nothing a reader sees moving while no shipped pair is glossed in Italian.
- The release that carries it reaches every store before any reader can choose Italian (D4).

**Non-Goals:**
- The Italian interface: its 24 modules and the restored alias (change 33); `_locales/it` (39); the
  Apple host's copy (40); the site's `/it/` (41); showing it, `INTERFACE_LANGUAGES` gaining `it` and
  listing en-it (45).
- The names of the other languages in Italian, and `NAMED` / `words()` (change 15's).

## Measured

On a scratch copy of `main` at `a5969fa2b`, nothing of stages 1–2 implemented:

- **Rust**: the enum is `crates/lingua-core/src/knowledge/profile.rs` (`ALL`, `tag`, `from_tag`,
  `studied`); 14 files name it, two match it exhaustively. With `Italian` appended after Spanish
  (`ALL[4]`, tag `it`): one compile error outside the core — `apps/lingua-agent/rust/src/vocab.rs`
  `native_name` —, two test arrays pinning three entries (`decks/backup.rs`'s names, `profile.rs`'s
  tags); then 979 tests
  pass and 2 fail, both using `it` as their example of an unknown native (`profile.rs`'s round trip,
  `lingua-pack`'s `spec_scenario_a_native_language_the_core_does_not_know_is_refused_at_build`). Every
  golden, `backup_format.rs` and `cross_native.rs` pass unchanged.
- **Serialised forms**: the variant is written by name (`"native_language": "Italian"`); no map is keyed
  by it, so nothing stored moves. An Italian native's backup is version 2 (studying English or Spanish;
  version 3 with French). But a build without the variant reads it as malformed — « malformed backup:
  unknown variant `Italian`, expected one of `French`, `English`, `Spanish` » —, not as unsupported:
  every release of the extension from 1.8.0, and of the host app from 1.6.0, which removed Italian from
  the enum (#733), to the one before this change. Either failure keeps the reader's data: the restore
  throws, the stored backup is left as it was. Before #733, no build had a way to set a native language
  (only the studied ones).
- **wasm**: `reprofileBackup(…, "it")` accepted; `setProfile("it")` refused until an Italian-glossed
  pack is held; `build_pack` accepts a native `it` (what change 36 needs).
- **The extension**: the native type is `src/analyzer/types.ts`, and `InterfaceLanguage` is an alias of
  it (M2). Widening the native type with the alias kept gives 33 type errors, all but three in tables
  keyed by interface language (the catalogue, locales, privacy URLs, leaving pages) — change 33's, 41's
  and 43's. With the interface type kept to fr, en and es, eight remain: change 15's two
  (`language-labels.ts` `NAMED`, `words()`), change 20's test-local map, two casts in
  `apple-activation-page.spec.ts`, the store's two mirrors of the native as the interface language
  (D2) and the own-names table (D3); D2 and D3 applied, the type check and the lint are clean. vitest then fails two « drafts are whole » checks until « Italiano »
  joins `SAME_EVERYWHERE`; after that 4,347 of 4,347 pass.
- **Runtime guards** read "is an interface language" for "is a native language": `isNativeLanguage`
  (the offer and the native-language message), `recordedNative` and `reasonOf` (`state/store.ts`), the
  Safari app's day counts (`state/app-relay.ts`), and the profile's `NATIVE_NAMES`. With a native list
  of four, the offer stays French, English and Spanish and an Italian browser is preset to English while
  no pair is glossed in Italian.
- **The backend** refuses nothing (`language_core.rs` `normalise_or`, a free `TEXT` column, Cymbra ID's
  `SupportedLocale::It`); the back office shows no native language; the site keys nothing by native
  language that must move now.

## Decisions

### D1 — The variant, appended

`NativeLanguage::Italian` after Spanish, `ALL` and `tag` (`it`) with it; the agent's `native_name`
« italien », as change 5 named the studied language there. The two tests that used `it` as the unknown
native drop it (`lingua-pack`'s takes `pt`); the two that pin the enum's names and tags gain
Italian. Once change 5's `StudiedLanguage::Italian` exists, `NativeLanguage::Italian.studied()` is
`Some(Italian)`: change 5's test that no native language answers Italian becomes « Italian answers
`studied()` as Italian », and its test that an `it` pack glossed in `it` is refused as an unknown native
becomes « refused as its own studied language » — rewritten here, as change 5's D3 hands the case to
this change; the main spec's *A pack names its native language* (« …or is the pack's own studied
language ») covers it. Changes 5's and 7's scenario « …glossed in `it`…, before Italian is a native
language » becomes vacuously true. The agent reads an `*-it` pack placed by hand in `~/.lingua` and
lists it as skipped (« glossé en italien ») instead of ignoring it silently; none is shipped to it (M17).

### D2 — The extension's native type, decoupled from the interface's until change 33

`NativeLanguage` gains `it`; `InterfaceLanguage` becomes `Exclude<NativeLanguage, "it">`, so every
table keyed by interface language stays three-keyed until change 33 writes `src/i18n/it` and
restores the alias; `INTERFACE_LANGUAGES` does not move — it gains `it` with change 45, when the
interface is shown (adding it breaks eight tests that are M12's question, I10). Where the store mirrors
the native language as the interface language — its two typed calls and `rememberInterfaceLanguage`,
which writes through an untyped record the compiler does not see — one `interfaceOf(native)` sends `it`
to English, the fallback M13 names, unreachable while no pair is glossed in Italian; change 33 removes it
with the alias.

### D3 — The guards read a native list

A `NATIVE_LANGUAGES` list of four, beside `INTERFACE_LANGUAGES`, for the guards that mean "a native
language": `isNativeLanguage` (the offer and the message), `recordedNative`, `reasonOf`, and the Cymbra Lingua
app's day counts — so an `it` label from the app is kept, not dropped; `NATIVE_NAMES` reads a
stored profile's `Italian`; `ownNames.italian` « Italiano » in the three catalogues and
`language-labels.ts`'s own-name table, « Italiano » joining `SAME_EVERYWHERE`. The offer stays
`shippedNatives` ∩ the native list: French, English and Spanish while no pair is glossed in Italian.

### D4 — Released before anyone can choose Italian

A build without the variant reads an Italian native's backup as malformed (*Measured*): only a file
imported on such a build, or a store rollback, could meet one — no build before #733 could set a native
language, the profile is never synced, the app shares its extension's backup, and the
native-language message for `it` is refused while no pair is glossed in Italian. So an extension
release and a host-app release that carry this change (change 40's at the latest) are live on the
Chrome Web Store, addons.mozilla.org and the App Store before change 45 lets a reader choose Italian, as
change 5's backup v4 precedes change 20 — a rollback then lands on a build that knows the variant. Not a
new schema version: the programme settles that an Italian native alone is a version 2 file; both
failures keep the reader's data, so only the message would differ; and `lingua-decks-review` already
accepts « malformed » from builds older than a format. The release order closes the window.

### D5 — Nothing else moves

No `packs.json`, `_locales`, catalogue, site, back office or server change; every golden and snapshot as
committed; the backend already accepts `it` for a card's gloss language and a day's native language.

## Risks / Trade-offs

- **The interface type apart from the native type for one change** → change 33 restores the alias; the
  mapping of D2 is unreachable until then, and a test holds it so.
- **A downgraded build meets an Italian native** → D4's release order.

## Migration Plan

Nothing to migrate: no reader can choose Italian. Released by the owner with the extension's next
release, before change 45's.

## Effort

Against 1–2 ideal days: the variant, the agent and the Rust tests 0.25–0.5; the extension's types,
guards, names and tests 0.5–1; the checks 0.25. 1–1.75.
