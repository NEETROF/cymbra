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
- The Italian interface: its 24 modules, `INTERFACE_LANGUAGES` gaining `it` (change 33); `_locales/it`
  (39); the Apple host's copy (40); the site's `/it/` (41); showing it and listing en-it (45).
- The names of the other languages in Italian, and `NAMED` / `words()` (change 15's).

## Measured

On a scratch copy of `main` at `a5969fa2b`, nothing of stages 1–2 implemented:

- **Rust**: the enum is `crates/lingua-core/src/knowledge/profile.rs` (`ALL`, `tag`, `from_tag`,
  `studied`); 14 files name it, two match it exhaustively. With `Italian` appended after Spanish
  (`ALL[4]`, tag `it`): one compile error outside the core — `apps/lingua-agent/rust/src/vocab.rs`
  `native_name` —, two test arrays pinning three names (`decks/backup.rs`, `profile.rs`); then 979 tests
  pass and 2 fail, both using `it` as their example of an unknown native (`profile.rs`'s round trip,
  `lingua-pack`'s `spec_scenario_a_native_language_the_core_does_not_know_is_refused_at_build`). Every
  golden, `backup_format.rs` and `cross_native.rs` pass unchanged.
- **Serialised forms**: the variant is written by name (`"native_language": "Italian"`); no map is keyed
  by it, so nothing stored moves. An Italian native's backup is version 2 (studying English or Spanish;
  version 3 with French). But a build without the variant reads it as malformed — « malformed backup:
  unknown variant `Italian`, expected one of `French`, `English`, `Spanish` » —, not as unsupported:
  every release from 1.8.0 (which removed Italian from the enum, #733) to the one before this change.
- **wasm**: `reprofileBackup(…, "it")` accepted; `setProfile("it")` refused until an Italian-glossed
  pack is held; `build_pack` accepts a native `it` (what change 36 needs).
- **The extension**: the native type is `src/analyzer/types.ts`, and `InterfaceLanguage` is an alias of
  it (M2). Widening the native type with the alias kept gives 33 type errors, all but three in tables
  keyed by interface language (the catalogue, locales, privacy URLs, leaving pages) — change 33's, 41's
  and 43's. With the interface type kept to fr, en and es, five remain: two change 15's
  (`language-labels.ts` `NAMED`, `words()`), one change 20's (a test-local map) and two casts in
  `apple-activation-page.spec.ts`. vitest then fails two « drafts are whole » checks until « Italiano »
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
native take `pt`; the two that pin the enum's names and tags gain Italian. Once change 5's
`StudiedLanguage::Italian` exists, `NativeLanguage::Italian.studied()` is `Some(Italian)`: change 5's
tests that hold « no native language answers Italian » and « an `it` pack glossed in `it` is refused as
an unknown native » become « refused as its own studied language » — rewritten here.

### D2 — The extension's native type, decoupled from the interface's until change 33

`NativeLanguage` gains `it`; `InterfaceLanguage` becomes `Exclude<NativeLanguage, "it">`, so every
table keyed by interface language stays three-keyed until change 33 writes `src/i18n/it` and
restores the alias; `INTERFACE_LANGUAGES` does not move (adding `it` there breaks eight tests that are
M12's question, change 33's and 45's). Where the store mirrors the native language as the interface
language, a mapping sends `it` to English — unreachable while no pair is glossed in Italian.

### D3 — The guards read a native list

A `NATIVE_LANGUAGES` list of four, beside `INTERFACE_LANGUAGES`, for the guards that mean "a native
language": `isNativeLanguage` (the offer and the message), `recordedNative`, `reasonOf`, and the Safari
app's day counts — so an `it` label from another device is kept, not dropped; `NATIVE_NAMES` reads a
stored profile's `Italian`; `ownNames.italian` « Italiano » in the three catalogues and
`language-labels.ts`'s own-name table, « Italiano » joining `SAME_EVERYWHERE`. The offer stays
`shippedNatives` ∩ the native list: French, English and Spanish while no pair is glossed in Italian.

### D4 — Released before anyone can choose Italian

A build without the variant reads an Italian native's backup as malformed (*Measured*): only a file
imported on such a build, or a store rollback, could meet one, since the profile is never synced. So the
release that carries this change is live on the Chrome Web Store, addons.mozilla.org and the App Store
before change 45 lets a reader choose Italian, as change 5's backup v4 precedes change 20 — a rollback
then lands on a build that knows the variant. A schema bump would not help: builds between change 5 and
this one read v4 but not the variant.

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
