# Design — add-lingua-studied-language-profile

## Context

See proposal.md (Why). Today:

| Where | What it does |
|---|---|
| `crates/lingua-core/src/decks/backup.rs` | `LinguaState` carries a `schema_version` field, and `BACKUP_SCHEMA_VERSION` is 1. `to_backup` writes pretty JSON. `from_backup` deserialises the whole file, then refuses a version other than 1. A language a build does not know therefore fails the restore as malformed. |
| `crates/lingua-core/src/knowledge/profile.rs` | `Profile` holds `native_language` and `studied_languages`, with `english_for_french()` as the MVP's. It implements `lingua-knowledge-model`'s *L1/L2 profile*. It is defined and tested, but nothing holds it. |
| `KnowledgeState`, `ExposureCounters`, `Deck` | Every per-language record is a map keyed by `StudiedLanguage`: statuses, calibration, declared level and its time, last-change times, exposure counters, cards. |
| `crates/lingua-wasm` | `backup`, `restore`, `reset` (the whole state back to defaults) and `resetStatuses`. |
| `apps/lingua-extension` | `STUDIED_LANGUAGE = "en"`, read on 33 lines in 9 files (#628): the reading session, settings, statistics, review, onboarding, popup, side panel, the sync's post-erasure calibration, and the v1 store migration. The speaker takes it when it is created: as a class field in the session, at module scope in the side panel. |
| `hydrateEngine` (`state/storage.ts`) | It restores the stored backup into the engine, and throws when the restore is refused. Every caller stops there. The content script logs and allows a retry, an extension page does not mount, the background's RPC host keeps failing, and a sync fails before saving. A refused restore has never overwritten the store. |
| The store's envelope | `{ v: 2, backup }`: the extension's own storage version (`STORAGE_VERSION`), not the backup's schema version. |
| S0 (`english_baseline.rs`) | It probes `reader.backup()` of a populated English reader, so any byte that moves in an English backup fails the gate. |

## Goals / Non-Goals

**Goals:**
- The reader's profile, in the state, in the backup, and nowhere else.
- A backup format that only moves when a non-English language is present, and whose version a
  build reads before anything else.
- One rule, applied by every surface, for the language it reads in.
- English untouched: the same requests, the same backup bytes.

**Non-Goals:**
- A way to choose languages: the « Langues étudiées » block and onboarding. Level and voice per
  language come with them, including `needsLevelChoice` counting a level decision made in any
  language (`add-lingua-language-choice`).
- Detecting a page's language among the reader's languages (`add-lingua-language-routing`).
  Until then, a surface reads in one language.
- Sending the reader's languages to the server for the cards' pull filter
  (`add-lingua-language-sync-client`). The list itself is never synced (D9).
- Which pack an engine starts with. It stays the default pair's (`package-lingua-packs-per-pair`).

## Decisions

### D1 — The existing `Profile` lives in `LinguaState`

`LinguaState` gains `profile: Profile`, reusing the type `lingua-knowledge-model` already
specifies rather than adding a second notion of the reader's languages. Its default is
`english_for_french()`. It is `#[serde(default)]`, so a backup without it restores as the
default. It is also skipped when serialised as the default, so an English backup does not gain a
field.
- `set_studied_languages` refuses an empty list or a duplicate, and leaves the previous list in
  place. Order matters: the first language is the primary one.
- Nothing sets the native language: every pair the programme ships is `*-fr`.
- `reset()` rebuilds the default state, so the profile returns to the default. `resetStatuses()`
  leaves it alone: it is not a status.
- An erasure resets the profile too, whether made on this device or propagated from another
  (`wipeLocal` in the sync). The reader erased their Lingua data, the choice included, and
  onboarding will offer it again (`add-lingua-language-choice`).
- The round trip stays identical, since an omitted profile is restored as the default it stood
  for. *Lossless backup and restore* needs no rewrite.

Why the state: D9 puts the choice in the state and the backup. The backup is also the one durable
record that the background owns and every surface restores, and that « Effacer mes données »
already erases.

Alternative: a preference in `chrome.storage.local`, like the voice. It would need its own owner,
its own exclusion from sync and its own erase path, and D9 rules it out.

### D2 — The version belongs to the written file

`schema_version` stops being a stored field. `to_backup` writes 1 when the profile is the default
and every per-language map holds English alone, and 2 otherwise. "Holds" means its keys; each of
`KnowledgeState`, `ExposureCounters` and `Deck` exposes the languages it holds.

A version 1 file keeps today's field order and has no profile, so it is today's file byte for
byte.
S0 checks this without being re-blessed.

Alternatives:
- **Write version 2 from this build on.** After a rollback past this change, a build released
  before it would refuse every backup, English ones included. The programme keeps "v2 only when
  needed" in its never-cut list.
- **Version 2 only when the profile changes.** Spanish records under a default profile would be
  just as unreadable to an older build.

### D3 — The version is read first

`from_backup` first reads a header that holds only `schema_version`, then the rest:
- Versions 1 and 2 share one shape.
- Any other version is refused as `UnsupportedVersion`, naming the version found. A version 3
  written by a later build is never reported as malformed by this one.
- A file without a version, or that is not JSON, is still malformed.

### D4 — What a build released before this change does

Such a build deserialises before it checks:
- A version 2 backup with another language's records fails as malformed, because the build does
  not know the language.
- One with a non-default profile and only English records fails as an unsupported version: serde
  ignores the unknown profile field, then the version check refuses 2.

Either way `hydrateEngine` throws, the surface stops, and nothing writes the store (Context).

The spec states this behaviour; it promises no recovery. It can only happen once R4 has given a
reader Spanish data and the extension is then rolled back past R2. Shipping R2 to every
installed build first is what retires the case.

### D5 — The port's root carries the list

- **Engine:** `studiedLanguages(): string` (a JSON array of tags, primary first) and
  `setStudiedLanguages(tags: string[])`, which refuses an unknown tag, an empty list or a
  duplicate.
- **Port:** `studiedLanguages()` and `setStudiedLanguages(languages)` on the root `LinguaPort`.
  These are whole-reader calls: they name no language and load no pack. They are added to the
  engine mirror, `WasmAnalyzerPort`, `MessagingLinguaPort` and the test helpers' fake port.

The names differ from `languages()` on purpose: `languages()` lists the packs the engine holds,
`studiedLanguages()` the languages the reader studies.

In R2 nothing in the extension calls the setter. The specs use it, and
`add-lingua-language-choice` will.

### D6 — One rule for the reading language

`readingLanguage(port, pairs = SHIPPED_PAIRS)`, next to `pairFor` in `src/analyzer/pairs.ts`,
returns the first language of the list that a shipped pair studies. When none is, it returns the
default pair's language.

A build that does not ship a language in the list keeps reading in what it ships, and the
profile stays untouched in the backup. An R2 build restoring a later Spanish reader's backup is the case.

Each surface resolves the language after it hydrates, and again after an external restore:
- **Reading session:** the `lang` view reads a field set in `start()` and `onExternalChange()`.
- **Settings, statistics and review:** on each `refresh()`.
- **Onboarding, popup and side panel:** after hydrating.
- **Sync's calibration after an erasure:** after the reset, so for the default profile, English.
- **Speaker:** it takes its language as a getter, so the session and the side panel can still
  create it before hydrating. Voice per language belongs to `add-lingua-language-choice`.
- **`hydrateFromV1`:** English, named so, since a reading-only store predates languages.

`STUDIED_LANGUAGE` is removed, so the compiler lists any place that still reads it.

## Risks / Trade-offs

- **A version 2 backup is unreadable by builds released before this change.** It only happens
  after Spanish data exists and the extension is rolled back past R2 (D4). The behaviour is
  written into the spec, nothing is overwritten, and R2 ships silently first.
- **A surface keeping a stale reading language** after another context changed the list. Every
  surface re-reads it after an external restore. In R2 the list never changes.
- **One more root call per surface start and refresh** (a message round trip on Firefox and
  Safari). Negligible next to the hydrate it follows.
- **An English backup changing by a byte** (a field written, or reordered) would leak the new
  format to every reader. S0 compares the whole English backup, and a core spec pins the version
  1 output.
