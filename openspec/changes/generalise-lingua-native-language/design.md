# Design — generalise-lingua-native-language

## Context

See proposal.md (Why). Today:

| Where | What it does |
|---|---|
| `crates/lingua-core/src/knowledge/profile.rs` | `NativeLanguage` has five variants: French, English, Spanish, Italian, Portuguese. It has `tag()`, but no `from_tag`, no `ALL`, and no link to `StudiedLanguage`. `Profile` holds the native language and the studied languages. `set_studied_languages` refuses an empty list or a duplicate, never the native language. No setter for the native language exists. `LanguagePair::key`, `pair_for` and `pairs` give `en->fr`, and only tests call them. |
| `crates/lingua-core/src/packs/meta.rs`, `pack.rs` | `PackMeta.native` is a free string. `read_meta` checks `studied` (`UnknownLanguage`) and the analyser version, never `native`. `studied_in(bytes)` reads the studied language from the metadata alone. `pair_key()` gives `en->fr`, test-only. |
| `crates/lingua-core/src/packs/set.rs` | `PackSet` holds one pack per studied language, keyed by it. `add` refuses only a second pack for a held language. |
| `crates/lingua-core/src/decks/backup.rs` | The profile is written only when it is not `english_for_french`, under serde's variant names (`"native_language": "French"`). Version 1 means the default profile and English records only; anything else is version 2. A restore does not validate the profile. |
| `crates/lingua-wasm/src/lib.rs` | `new` and `reset` both set `LinguaState::default()`, so every engine and every reset gives (French, [English]). `levelLadder` makes an estimated pack borrow the held English pack's live figures, and keeps its own figures when no English pack is held. Today's figures, identical in both goldens: 0, 1,292, 3,359, 7,988, 16,326, 20,556. |
| `apps/lingua-extension/src/analyzer/pairs.ts` | `pairFor(language, pairs)` returns the first listed pair studying the language. `DEFAULT_LANGUAGE`, the engine's first pack and every fallback come from `pairs[0]`. Nothing in `src/` reads a pair's native side. |
| `apps/lingua-extension/src/analyzer/engine.ts` | `WasmAnalyzerPort` builds on `pairs[0]` and adds another pair's pack the first time its language is asked. |
| `apps/lingua-extension/src/state/profile.ts` | `studiedLanguagesOf(backup)` reads `profile.studied_languages` from the stored backup without an engine, for the background's translation models. |
| `apps/lingua-extension/tool/packs.mjs`, `build.mjs` | The build checks each listed pack's studied language and analyser version, scanned from its header, but never its native language. |
| `apps/lingua-agent/rust/src/engine.rs` | `Library` keys installed packs by `Pack::studied_in`. The first file of a language wins in file-name order, and the native language is never compared. |

Every pack and fixture in the tree is glossed in French.

## Goals / Non-Goals

**Goals:**
- The native language is read, validated and held in one place per engine.
- The extension, the sync and the agent pick packs by (studied, native).
- Nothing moves for a French reader:
  - the same requests, packs, offered languages and backups;
  - both baselines and the version 1 fixture unchanged without re-blessing.

**Non-Goals:**
- A way to choose the native language, and the interface language that follows it:
  `add-lingua-native-language-choice` (change 20) and `add-lingua-interface-language` (change 13).
- The gloss independence of the studied side (« a dictionary word », the pinned tag pool):
  `add-lingua-pack-lexical-layer` (change 5).
- Translation routes by pair: `generalise-lingua-translation-routes-by-pair` (change 8).
- What the server learns about the native language: changes 10–12.

## Decisions

### D1 — The native language is validated at load, and the metadata keeps its shape

`PackMeta.native` stays a string, with its field order, so no pack byte and no `pin.json` moves.

`Pack` gains a parsed `native: NativeLanguage`. `read_meta` checks it after the studied language
and the analyser version, so the existing errors keep their precedence. It refuses:
- an unknown tag, as `PackError::UnknownNative`;
- a native language equal to the pack's studied language, as `PackError::NativeStudied`.

`Pack::pair_in(bytes)` reads (studied, native) from the metadata alone, beside `studied_in`. It
returns a `LanguagePair`, whose `key()` reads `<studied>-<native>`. `PackMeta::pair_key` follows
the same format.

`lingua-pack`'s `build_pack` refuses the same packs, so no build produces a pack the core would
refuse.

Alternative: type `PackMeta.native` as `NativeLanguage`. Serde would then write `"French"`, which
would change every pack's metadata, both pins and both baselines.

### D2 — The native languages are French, English and Spanish

Italian and Portuguese leave `NativeLanguage`:
- Nothing could set them, so no backup holds them.
- They are not part of the matrix.
- Keeping them would make a pack glossed in `it` loadable while no part of Lingua could serve it.

The remaining variants keep their names and their order, since backups write the names.
`NativeLanguage::ALL` and `from_tag` mirror `StudiedLanguage`'s. A native language maps to the
studied language with the same tag: English and Spanish do, and French maps to none until
French is studied (stage 3).

### D3 — A pack set is bound to one native language

`PackSet` takes its native language from its first pack. `add` refuses a pack of another native
language with `PackSetError::OtherNative { held, pack }`, which names both. It checks this before
`AlreadyHeld`, and leaves the set unchanged. The existing errors keep their messages.

### D4 — The profile never studies its native language, and changes as a whole

- `set_studied_languages` refuses a list that holds the native language's studied counterpart,
  with `ProfileError::NativeStudied`.
- `Profile::set(native, studied)` validates both together and replaces both, or neither.

There is no lone native-language setter. A reader who switches from French to English while
studying English must change both at once, and either order of two separate calls is refused
half-way.

The engine's binding `setProfile(native, languages)` also refuses a native language that no held
pack is glossed in. Changing the native language therefore means building an engine on that
native language's packs first, which change 20 does.

A restore stays lossless and does not validate the profile; that is today's behaviour.
- The extension builds its engine for the native language stored in the backup (D7), so the two
  agree.
- An engine handed a backup of another native language still restores it, and
  `profileNativeLanguage()` shows the mismatch.

### D5 — A new engine and a full reset follow the packs

`new` and `reset` set the profile to the pack set's native language, studying its default
language: the first pack's.
- `reset` used to rebuild `LinguaState::default()`, which gave every reader French glosses. It now
  keeps the engine's native language (M3), whose packs the extension built it on.
- For an engine started on en-fr, this is exactly `english_for_french()`, the default profile. The
  backup stays at version 1, `is_default` and the serde default are untouched, and so is the
  version 1 fixture.

Alternative: keep the profile's native language through a reset. After a mismatched restore
(D4), that native language could equal the default studied language, which is an invalid profile.

### D6 — English's typical vocabularies are a frozen constant

`ENGLISH_TYPICAL_VOCABULARY = [0, 1_292, 3_359, 7_988, 16_326, 20_556]`, in CEFR order, lives in
`knowledge/vocabulary.rs`.
- Its doc names its provenance: the en-fr pack `2026.09.26+627146e`, frozen on 2026-10-07.
- An estimated ladder of a language other than English uses it, and keeps writing
  `typicalFrom: "en"`. The rows are the same JSON, so the es-fr baseline does not move.
- English's own ladder keeps computing from its own pack, so the en-fr baseline does not move
  either. An English dictionary update moves English's ladder alone.

No test ties the constant to the live en-fr figures. Such a test would fail on the next English
dictionary update, and removing that coupling is the point of freezing the figures. The es-fr
golden pins the values.

Alternatives:
- Keep borrowing the held English pack. A reader whose native language is English never holds
  one, and would get Spanish's own figures.
- Put the figures in each estimated pack's metadata. That changes pack bytes, and each pair would
  carry a copy that could drift.

### D7 — The extension chooses a pair by studied and native language

`pairs.ts` gains:
- `DEFAULT_NATIVE = "fr"` (M22) and `nativeOf(pair)`;
- `pairsOf(native)`, the listed pairs of a native language in listed order;
- `defaultPair(native)`, the first of them;
- `pairFor(language, native)`, which names the native side explicitly, so `tsc` finds every caller.

`readingLanguage` and `acceptedLanguages` read the port's native language, filter by it, and fall
back to the native language's default pair. `DEFAULT_LANGUAGE`, a placeholder before hydration,
stays `"en"`.

The reader's native language comes from the stored backup:
- `nativeLanguageOf(backup)` reads `profile.native_language` under the core's names.
- It gives `"fr"` when there is no profile, when the name is unknown, when the JSON does not
  parse, or when the name is a native language no listed pair is glossed in. The same reader
  serves the background, as `studiedLanguagesOf` already does.

`WasmAnalyzerPort` takes a native-language resolver, called once before the first pack is
fetched:
- `create-port.ts` and `background.ts` wire it to the stored backup.
- It builds on `defaultPair(native)`, and adds only that native language's pairs.
- Its refusal for a language no pair serves names the native language's pairs. For French
  readers the message is byte-identical.
- `LinguaPort.nativeLanguage()` answers the native language the port was built for, and loads no
  pack. The RPC forwards it like every whole-reader call.

« Langues étudiées » offers the studied languages of the reader's native language's pairs.

Alternative: hard-wire French in the port until change 20. The resolver would then arrive in the
same change as the choice, and nothing would exercise it before it matters. One storage read per
engine build is the cost, and hydration already makes the same read.

### D8 — The build checks the pack it ships

`packMeta` reads the container's metadata exactly: the JSON at offset 14, whose length is the u32
at offset 10. It no longer scans the bytes with a regular expression.
- `assertPacksMatchEngine` refuses a listed pack whose `native` is not its pair's native side, and
  the message names the rebuild command.
- `shippedPairs` refuses a pair whose two sides are the same (`fr-fr`).
- The check reads the built pack rather than the tables, because the pack is what ships.

### D9 — The agent follows `pack.lingua`'s native language

`Library` reads each file with `Pack::pair_in`. Its native language is:
- `pack.lingua`'s, when that file reads;
- otherwise the first readable file's, in file-name order;
- with `$LINGUA_PACK`, that pack's.

A file glossed in another language is skipped, as an unreadable one is. `lingua vocab` names it
after its listing, while the hooks and the statusline stay silent.

The anchor is `pack.lingua` rather than plain file-name order because any `xx-en` or `xx-es` name
sorts before `pack.lingua`: one stray file would otherwise evict the French packs.

### D10 — One umbrella requirement in the extension

The extension requirements that speak of the shipped, listed or default pair are:
- *A language's pack is loaded the first time it is needed*;
- *Each surface reads in the reader's language*;
- *The reader chooses the languages they study*;
- the open `add-lingua-language-sync-client`'s pull requirement.

They are not rewritten one by one. One ADDED requirement says they mean the pairs of the reader's
native language. The sync requirement belongs to an open change whose capability is not in
`specs/` yet, so it cannot be modified; the umbrella covers it. Change 13 uses the same pattern
for interface copy.

## Risks / Trade-offs

- **[A generalisation moves a French reader's output.]** → `english_baseline` and
  `spanish_baseline` must pass without `LINGUA_BLESS`, and `backup_format.rs` must keep its
  fixture. New assertions are separate tests, never probes, since a probe would move a golden.
- **[The extension reads a native language the core renamed, and silently falls back to
  French.]** → A core test pins the serde names that `nativeLanguageOf` reads, as the existing test
  does for `studied_languages`.
- **[A content script pays one more storage round trip before its first pack.]** → It is a read the
  hydration already makes. Measured on the dogfood build; it can be merged into the hydration
  read if it shows.
- **[A backup of another native language reaches an engine built for French]**, through a file
  restore on a device that never chose. → The restore keeps it. The extension rebuilds on the
  stored native language at the next start, and finds no pair for it before change 21 ships one,
  so it falls back to French (D7). The profile itself is kept.
- **[The agent's anchor surprises a user who installed only `es-en.lingua` beside `pack.lingua`.]**
  → `lingua vocab` says which file it skipped and why.

## Migration Plan

None: no stored format, wire format or pack changes. Rollback is reverting the change. A
version 2 backup written in between is already readable by the previous build.
