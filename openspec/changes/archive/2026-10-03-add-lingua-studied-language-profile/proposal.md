# add-lingua-studied-language-profile — the reader's studied languages, kept with their state

## Why

Every surface of the extension asks the engine in `STUDIED_LANGUAGE`, a constant set to `"en"`
(#628). Reading Spanish needs the language to come from the reader. The product owner decided
where that choice lives (D9 in `docs/lingua/spanish-programme.md`): in the reader's state and
backup, never synced, offered again on a new device. The programme also fixed how the backup
changes: a new schema version is written only when a non-English language is present, read
first by new builds, and the behaviour of older builds is written into the spec.

This is change 10 of the Spanish programme, the last of R2, a silent English release. Every
reader's list holds English alone, and nothing lets a reader change it yet: « Langues
étudiées » is `add-lingua-language-choice`. An English reader's backup stays byte for byte
what it is.

## What Changes

- **The reader's profile, in their state.** `LinguaState` holds the reader's language profile.
  It reuses the existing `Profile` type, the *L1/L2 profile* of `lingua-knowledge-model`, which
  nothing stored until now. The profile holds the studied languages, ordered with the primary
  first, and the native language. The default is English studied, French native. The profile
  lives in the backup and nowhere else: it is never synced. A full reset returns it to the
  default, and resetting statuses leaves it as it is.
- **Backup schema version 2, only when needed.** A backup whose profile is the default and whose
  every per-language record is English is written exactly as today: schema version 1, with no
  new field. Any other backup is written as version 2, profile included.
- **The version is read first.** A restore reads the schema version before the rest of the
  file and reads versions 1 and 2. Any other version is refused as unsupported, never reported
  as malformed.
- **What older builds do is stated.** A build released before this change refuses a version 2
  backup: as malformed when it holds another language's records, as unsupported otherwise. Like
  any refused restore, its reader then does not start, and the stored backup is left as it is.
- **The engine and the port expose the list.** `studiedLanguages()` and
  `setStudiedLanguages(languages)` are added on the engine and on the port's root, and the RPC
  forwards them. A list must be non-empty, without duplicates, and name only languages the
  engine knows. The native language stays French: every pair the programme ships is `*-fr`.
- **Each surface reads in the reader's language.** `STUDIED_LANGUAGE` goes. A surface reads in
  the first language of the list that the package ships, or the default pair's language when
  none is. It reads that language when it starts, and again after another context changes the
  backup. A reading-only (v1) store is still migrated in English: its data predates languages.
- **English does not move.** Every list is English alone, so every surface asks in English as
  before. The S0 baseline, which captures a populated English backup, does not change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-decks-review`: ADDED — *The backup records the reader's language profile* and *A
  restore reads the backup's schema version first*. *Lossless backup and restore* is not
  rewritten: the round trip stays identical, since an omitted profile is restored as the default.
- `lingua-knowledge-model`: no delta. Its *L1/L2 profile* requirement already describes the
  profile; this change gives it a home in the state.
- `lingua-browser-extension`: ADDED — *Each surface reads in the reader's language*.

The open changes on these capabilities hold requirements under other names, and nothing they
hold is rewritten.

## Impact

- **Products.** Cymbra Lingua only:
  - `crates/lingua-core`: the state (now holding the profile), its backup version, and the
    languages each per-language record exposes;
  - `crates/lingua-wasm`: two bindings;
  - `apps/lingua-extension`: the port, the engine mirror, the messaging port and every surface
    that named `STUDIED_LANGUAGE`.

  No server, `.proto`, pack or sync change. ID, Music, Live, the back office and the site are
  untouched.
- **Release.** The last change of R2. After it come the R2 dogfood pass and the silent English
  release. A version 2 backup is readable only by builds that carry this change, so R2 has to
  reach installed builds before any Spanish data can exist.
- **Coverage.** The profile, its validation and the version rule live in `lingua-core`, which is
  host-tested.
