# Design — add-lingua-native-language-choice

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `crates/lingua-core/src/knowledge/profile.rs` | `Profile {native_language, studied_languages}`; `set(native, languages)` changes both or neither, refuses an empty list, a duplicate, the native among the studied; `english_for_french` default |
| `crates/lingua-wasm/src/lib.rs` | `new(pack)` takes the native from the first pack; `addPack` refuses another native; `setProfile(native, languages)` errors when `native != packs.native()`; `restore(backup)` replaces the whole state, profile included; `reset` → `fresh_state(packs)` keeps the engine's native |
| `src/analyzer/engine.ts` `WasmAnalyzerPort` | `resolveNative` memoised in `native()`, `engine()` memoised, packs chosen by `pairFor(language, native)`; no rebuild path; `setProfile` declared, never called; the popup, the onboarding and the content script each build their own `WasmAnalyzerPort` (`popup.ts`, `onboarding.ts`, `create-port.ts`), the rpc host serves the messaging fallback only |
| `src/analyzer/pairs.ts` | `SHIPPED_PAIRS`, `pairsOf(native)`, `defaultPair(native)`, `pairFor`, `acceptedLanguages`; `DEFAULT_NATIVE = "fr"`; no `shippedNatives` |
| `src/state/storage.ts`, `src/state/store.ts` | `storedNativeLanguage(store)` from the backup's profile, `fr` otherwise; `hydrateEngine`; the background owns the store, `announceStoreChange` → the scheduler and every `watchStore`/`watchBackup` |
| `src/background.ts` | two engines of its own, the rpc one (memoised `hydrated`) and the sync one, both `const` ports whose engines are memoised; `onInstalled` fires on every browser with `reason` `install` or `update`, and on `install` opens the onboarding (best effort; Safari may not) |
| Surfaces and the store | the reading session restores on `onExternalChange`; the side panel restores unless it is reviewing; the review page watches the backup; the popup watches the store for its statistics only and hydrates its Réglages port once; the onboarding and the statistics page hydrate once and never watch; `hydrateEngine` writes a backup into an empty store on first use (the content script, the onboarding, the side panel, the statistics page) |
| `src/reading/studied-languages-view.ts` | the studied languages' choice; `offerFor(native)` hides the block when fewer than two languages are offered; mounted in Réglages (« Langues étudiées ») and onboarding |
| `src/onboarding/onboarding.ts`, `src/popup/popup.ts` | the languages section, level rows, account offer; the popup's first-run level call to action in `#controls`, shown when a content script answers; the Safari popover |
| `src/reading/settings-view.ts` | tabs « Langue », « Apparence », « Pages & livres », « Données »; the block titles, in the catalogue after change 15 |
| `src/sync/sync.ts` `widen` | clears the cursors when the accepted languages grow; cards are keyed by studied language and lemma |
| Change 13 | the interface-language key, written by the store's owner at start when absent and after a backup change (debounced, off the write path); a surface reads it before it renders its copy |

## Goals / Non-Goals

**Goals:**
- One choice, the native language, from three places, hidden while one native ships.
- Choosing rebuilds every engine for the native and keeps the reader's state.
- A studied language that becomes the native is no longer studied; nothing else is lost.
- An installed extension is never asked (M22).

**Non-Goals:**
- The second native's pack (change 21 commits it, change 34 ships it) and the choice's French copy moved with its surfaces
  (14, 15, 17).
- A native language with no shipped pair: not offered.
- Syncing the profile (never).

## Decisions

### D1 — `shippedNatives()` decides whether the choice exists

`pairs.ts` exports `shippedNatives(pairs = SHIPPED_PAIRS)`: the native languages with at least
one shipped pair, in listed order. The three places render the choice only when it counts two or
more; today it counts one, so nothing is shown (M22). The studied-languages view already hides
itself the same way.

### D2 — The backup's profile is rewritten by a pure engine function, then restored by a fresh engine

An engine cannot be moved to another native: its packs are glossed in one (change 4), and
`restore` replaces the profile with the backup's. So the change happens on the backup string,
not in a running engine: `lingua-wasm` gains `reprofileBackup(backup, native, studied)`, a pure
function (no packs needed) that validates the profile as `Profile::set` does and returns the
backup with the new profile, written at the version the state needs (`to_backup`'s rule). The background, which owns the store, serves a
runtime message `lingua-native-language` `{native}`:
1. refuse when `pairsOf(native)` is empty;
2. compute the studied languages: the current ones without `native` and without those no shipped
   pair glosses in `native`, or `defaultPair(native)`'s studied language when nothing is left;
3. `reprofileBackup` the stored backup, save it, write the interface-language key from the new
   profile itself — the store owner's mirror (change 13 D3) runs in its debounced reaction, and a
   page reloading on the announcement must read the new key — and announce the store change
   with its reason;
4. have its reading engine restore the backup before its next answer — the rpc port's `hydrated`
   memo is cleared, and the restore rebuilds the engine for the new native language (D3); the sync
   port restores the backup at the start of every run and follows by itself. No engine is dropped:
   a port without one would serve a fresh state until something restored it.

Steps 2 to 4 run with every sync held (the scheduler's `exclusive`, as the erasure does, and after
any change asked before): a sync restores the backup, applies what it pulled and saves the result,
and a change landing in between would be saved over — the native language reverted — or would save
over the pulls.

### D3 — Every port follows the backup's native language

`WasmAnalyzerPort.restore(backup)`: when the backup's profile names a native other than the
engine's, the port drops its engine and its added packs, resolves to that native and builds
again before restoring; a port built after the change resolves to the new native from the start
(`storedNativeLanguage`). What an open surface does when the native language changes is one
rule, for every surface: the background announces the change with its reason; an extension page
(the popup, the side panel, the statistics page, the onboarding, the reader, the account page)
reloads itself, so it reads the key, fills its copy and sets its `lang` as it does on opening;
the content script tears its `ReadingSession` down and builds a new one, reading the key first
(change 14's path), so the HUD, the drawer, the word card and the selection card are the new
language's with their hosts' `lang`. The reading session's `onExternalChange` keeps handling a
synced backup as today; a native change is the one case that rebuilds.

A context that started before the change — a session taken down while its engine answered, a page
about to reload, a port not restored yet, a tab the browser restores from its back/forward cache,
which heard nothing while it was frozen — may still save the backup its engine holds, at any time,
and saving it would undo the choice. So the store's owner records the native language last chosen
beside the backup, in the store it owns and in the same write (`cymbra-lingua-last-native`): the
record outlives the background, which Chromium stops when idle and Safari suspends, and no window of
time is involved. A backup written without a reason that names another native language is refused,
nothing written. Only the change and a restore from a file write the record. A restore from a file
carries its own reason: the native language it names becomes the reader's, and when it is another
one the restore is announced as a change of native language, so every surface follows it as it
follows a choice. A device where nothing was ever recorded — every installed extension, every reader
who never chose — has no context holding another native language's engine: nothing is refused
there, and no backup is parsed for it. The erasure (`SyncEngine.wipeLocal`, from « Effacer mes
données Lingua » or from a sync's `checkErasure`) restores the stored backup before it resets, since
an engine's reset keeps the native language of the backup it last restored: an erasure queued behind
a change starts the reader over in the language they chose.

### D4 — Three places, one view

`src/reading/native-language-view.ts` mounts the choice: the native languages of
`shippedNatives()` named each in its own language (« Français », "English", « Español » — the
same in every interface language, on change 13's same-in-every-language list), the current one
selected, the consequence for the studied languages stated before confirming, a change sending
`lingua-native-language` and then rebuilding the host's port. Mounted:
- in Réglages, « Langue », above « Langues étudiées », its title a catalogue entry like the others
  (change 15 moved them; `lint-settings-hosts` reads the catalogue);
- in the onboarding, before the languages section: on a new install the preset is applied before
  anything paints — `navigator.language`'s primary subtag when it is one of `shippedNatives()`,
  English when that ships, French otherwise; the background writes the preset through
  `lingua-native-language` (the backup `hydrateEngine` wrote, or the rpc port's engine's fresh
  `backup()`, reprofiled) and the onboarding paints in it; the reader confirms or changes it in the same step (M3);
- in the popup's first run, as a call to action of its own above `#controls` (which shows only
  once a content script answers), when `cymbra-lingua-native-chosen` is unset. New install and
  update are told apart by `onInstalled`'s `reason`, which fires on every browser: on `update`
  the background sets the marker at once — an installed extension is never asked (M22).
  Otherwise the marker is set by the reader's answer in one of the three places — confirming the
  preset counts, in the onboarding as in the popup — and never by the preset itself, so a popover
  closed before the answer asks again. Should a browser not report an update (Safari's host app
  may update the extension without it), a preset refuses a backup that holds the reader's data —
  a status, a card, a profile other than the default — and marks the choice as made instead; the
  first preset that found the device new sets `cymbra-lingua-native-preset`, so the reader's own
  first statuses never pass for an installed extension's. A store that merely holds a backup
  proves nothing: reading one page writes one, with its exposures.

### D5 — What follows a change

The accepted languages change: `widen` clears the cursors when they grow and a pull brings the
cards of a newly accepted language; when they shrink, nothing is pulled and the cards of the
now-unstudied language stay in the backup, not reviewed. The translation controller reconciles
on its next status (change 8 D5), downloading nothing until asked. The review follows the active
tab's language as it does, and the first studied language otherwise. A full reset after the
change keeps the native: `reset` builds the fresh state on the engine's native, the chosen one
(M3).

### D6 — Copy

The French strings of the three places (« Langue maternelle », « Je lis en… », the call to
action) are catalogue entries (change 13) with their English and Spanish; the native names are
on the same-in-every-language list.

## Risks / Trade-offs

- **An engine rebuilt under a page being read** → the content script tears its session down and
  builds a new one on the announced change (D3), as a synced backup restores today.
- **A reader who studies only the language they choose as native** → the studied list is set to
  the native's first shipped pair's studied language, and said so in the choice.
- **Safari never opening the onboarding** → the popup's first-run call to action, while the
  marker is unset.
- **A device whose browser is German** → the preset is English when English ships, French
  otherwise; the reader changes it in the same step.
- **A backup restored from a file naming another native** → saved with its own reason: that native
  language becomes the reader's, recorded as their last choice, and the restore is announced as a
  change of native language; `restore` rebuilds every port for it (D3).

## Migration Plan

One release, silent: the choice is hidden until change 34 lists a second native's pair in `packs.json`. An install
updating to this build is marked as chosen by the update and never asked; a new install is preset.
