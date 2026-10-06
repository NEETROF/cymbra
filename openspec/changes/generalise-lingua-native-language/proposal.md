# generalise-lingua-native-language — one native language per engine, the reader's

## Why

The language matrix ([docs/lingua/language-matrix-programme.md](../../../docs/lingua/language-matrix-programme.md))
adds pairs glossed in English and in Spanish. The core already models a native language, but
nothing reads it:

- `NativeLanguage` exists, and so does the profile's `native_language`.
- Every pack's `meta.native` is written but never checked.
- An engine accepts a pack glossed in any language.
- The extension picks a pair by its studied language alone, taking the first listed.
- The agent keeps the first file of each language.
- A full reset gives every reader French glosses again.
- An estimated ladder borrows English's figures only from an English pack, which a reader whose
  native language is English never holds.

This is change 4 of the programme, the first of stage 1. It ships as a silent release:
- Every listed pair is glossed in French, so French readers see nothing new.
- The en-fr and es-fr invariance baselines do not move, and neither does the version 1 backup
  fixture.
- Decisions M3 (the native language stays on the device, and a full reset keeps it) and M22
  (existing installs stay French without being asked) apply.

## What Changes

- **A pack names its native language.**
  - The core reads `meta.native`. It refuses a native language it does not know, and a pack
    whose native language is its studied language.
  - It reads a pack's pair, `<studied>-<native>`, from the metadata alone, as it already reads
    the studied language.
  - The extension's build refuses a listed pack whose native language is not its pair's.
- **One native language per engine.**
  - An engine takes its native language from its first pack. It refuses a pack glossed in
    another language, and keeps the packs it holds.
  - It reports that native language, and the native language of the reader's profile.
- **The native language is never studied.** A choice of studied languages that includes the
  native language is refused. One call sets the native language and the studied languages
  together, and it is refused when no pack the engine holds is glossed in that native language.
- **A new engine and a full reset follow the packs.**
  - A new engine's profile, and the profile after a full reset, study the first pack's language,
    with the engine's native language.
  - For an engine started on en-fr — every engine today — that is the default profile, so the
    backup stays at version 1.
- **The estimated ladder reads English's figures from the core.**
  - English's typical vocabularies become a constant of the core, frozen at the values the en-fr
    pack gives today: 0, 1,292, 3,359, 7,988, 16,326 and 20,556.
  - A Spanish ladder shows the same figures whatever packs the engine holds, and whatever the
    next English dictionary update does.
  - The English ladder still computes its own figures.
- **The extension chooses a pair by studied and native language.**
  - It reads the reader's native language from the stored profile, and uses French when the
    profile names none.
  - The engine starts on that native language's first listed pair. A pair glossed in another
    language is never loaded, accepted from the sync or offered in Réglages.
  - With today's list, en-fr then es-fr, nothing a reader sees changes.
- **The agent follows one native language.**
  - The plugin follows the packs glossed in the same language as `pack.lingua`, or as the first
    pack when there is no readable `pack.lingua`.
  - It skips the others, as it already skips a pack that does not load.
- **Pair keys read `<studied>-<native>`.**
  - The test-only `en->fr` keys go.
  - Italian and Portuguese leave the native languages: nothing could set them, and no backup
    can hold them.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-knowledge-model`: MODIFIED — *L1/L2 profile*.
  - It says knowledge state is keyed by pair; it is keyed by studied language.
  - It says only en-fr ships; es-fr does too.
  - It gains the rule that the native language is never studied.
- `lingua-decks-review`: MODIFIED — *The backup records the reader's language profile*.
  - It adds the native language's refusals and the joint choice.
  - It adds the full reset that keeps the engine's native language.
- `lingua-data-packs`: ADDED — *A pack names its native language*.
  - *The shipped pairs are one list* is modified by the open `enable-lingua-spanish` and is not
    rewritten here. The build's native-language check lives in the ADDED requirement.
- `lingua-analysis`: ADDED — *An engine serves one native language*, beside *One WASM engine
  serves every studied language*, which keeps its text.
- `lingua-browser-extension`:
  - ADDED — *A reader is served the pairs of their native language*. This is an umbrella over
    every requirement that speaks of the shipped, listed or default pair, including the open
    `add-lingua-language-sync-client`'s, which cannot be rewritten before it archives.
  - MODIFIED — *An estimated ladder shows English's typical vocabularies*.
- `lingua-agent-capture`: MODIFIED — *The languages followed are the packs installed*.

No requirement this change modifies is held by an open change.

## Impact

- **Products.** Cymbra Lingua only:
  - `crates/lingua-core`: native languages, pack metadata, the pack set, the profile, the
    ladder's constant;
  - `crates/lingua-wasm`: three bindings, `new` and `reset`;
  - `crates/lingua-pack`: a build-time check;
  - `apps/lingua-extension`: pair choice, the port, the stored-profile reader, the settings'
    offered languages, the build's pack check;
  - `apps/lingua-agent`: the pack library.

  ID, Music, Live, the back office and the site are untouched. Nothing new is consumed from them.
- **No server, `.proto`, sync or pack byte change.** Translation routes stay keyed by studied
  language and end in French; routes by pair are change 8.
- **Release.** Silent, the first of stage 1.
- **Compatibility.**
  - A backup naming English or Spanish as the native language is a version 2 backup that
    released builds already read: the profile's names do not change.
  - No pack byte moves.
- **Coverage.**
  - The rules live in `lingua-core`, which is host-tested.
  - The engine's refusals are tested with `wasm-bindgen`, since a `JsError` cannot be built off
    wasm.
