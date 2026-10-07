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
| `src/background.ts` | two engines of its own, the rpc one (memoised `hydrated`) and the sync one; `onInstalled` with `reason === "install"` opens the onboarding (best effort; Safari may not) |
| Surfaces and the store | the reading session restores on `onExternalChange`; the side panel restores unless it is reviewing; the review page watches the backup; the popup watches the store for its statistics only and hydrates its Réglages port once; the onboarding never watches |
| `src/reading/studied-languages-view.ts` | the studied languages' choice; `offerFor(native)` hides the block when fewer than two languages are offered; mounted in Réglages (« Langues étudiées ») and onboarding |
| `src/onboarding/onboarding.ts`, `src/popup/popup.ts` | the languages section, level rows, account offer; the popup's first-run level call to action in `#controls`, shown when a content script answers; the Safari popover |
| `src/reading/settings-view.ts` | tabs « Langue », « Apparence », « Pages & livres », « Données »; the block titles, in the catalogue after change 15 |
| `src/sync/sync.ts` `widen` | clears the cursors when the accepted languages grow; cards are keyed by studied language and lemma |
| Change 13 | the interface-language key, written by the store's owner on every backup write; a surface reads it before it renders its copy |

## Goals / Non-Goals

**Goals:**
- One choice, the native language, from three places, hidden while one native ships.
- Choosing rebuilds every engine for the native and keeps the reader's state.
- A studied language that becomes the native is no longer studied; nothing else is lost.
- An installed extension is never asked (M22).

**Non-Goals:**
- The second native's pack (change 21) and the choice's French copy moved with its surfaces
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
backup with the new profile, schema version 2. The background, which owns the store, serves a
runtime message `lingua-native-language` `{native}`:
1. refuse when `pairsOf(native)` is empty;
2. compute the studied languages: the current ones without `native`, or `defaultPair(native)`'s
   studied language when nothing is left;
3. `reprofileBackup` the stored backup, save it — the store's owner writes the
   interface-language key with it (change 13 D3) — and announce the store change;
4. drop its two engines (the rpc one and its `hydrated` memo, the sync one): each is rebuilt
   for the new native on its next use, resolving the native from the backup as today.

### D3 — Every port follows the backup's native language

`WasmAnalyzerPort.restore(backup)`: when the backup's profile names a native other than the
engine's, the port drops its engine and its added packs, resolves to that native and builds
again before restoring; a port built after the change resolves to the new native from the start
(`storedNativeLanguage`). The surfaces that restore on the store's change (the reading session's
`onExternalChange`, the review page, the side panel — once it is not reviewing) therefore
follow; the popup and the onboarding, which hydrate a port once, rebuild their port and re-read
the key on the announced store change — the one case the choice itself causes, in the surface
that made it. The content script re-reads the key then too and re-labels its HUD and card
(change 14's hand-over), which is what "an open surface when the key changes" means.

### D4 — Three places, one view

`src/reading/native-language-view.ts` mounts the choice: the native languages of
`shippedNatives()` named each in its own language (« Français », "English", « Español » — the
same in every interface language, on change 13's same-in-every-language list), the current one
selected, the consequence for the studied languages stated before confirming, a change sending
`lingua-native-language` and then rebuilding the host's port. Mounted:
- in Réglages, « Langue », above « Langues étudiées », its title a catalogue entry like the others
  (change 15 moved them; `lint-settings-hosts` reads the catalogue);
- in the onboarding, before the languages section: on a new install — no backup in the store —
  the preset is applied before anything paints: `navigator.language`'s primary subtag when it is
  one of `shippedNatives()`, English when that ships, French otherwise; the background writes the
  preset through `lingua-native-language` and the onboarding paints in it; the reader confirms or
  changes it in the same step (M3);
- in the popup's first run, in `#controls`' place when no content script has answered yet, as a
  call to action before the level's, when `cymbra-lingua-native-chosen` is unset AND the store
  holds no backup: an install from before this change has a backup and is never asked (M22); the
  marker is set by the three places and by the onboarding's preset.

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

- **An engine rebuilt under a page being read** → the content script restores on
  `onExternalChange` as it does for a synced backup today; the HUD and the card re-read the key
  and re-label.
- **A reader who studies only the language they choose as native** → the studied list is set to
  the native's first shipped pair's studied language, and said so in the choice.
- **Safari never opening the onboarding** → the popup's first-run call to action, for a store
  without a backup.
- **A device whose browser is German** → the preset is English when English ships, French
  otherwise; the reader changes it in the same step.
- **A backup restored from a file naming another native** → `restore` rebuilds (D3), as a sync
  does.

## Migration Plan

One release, silent: the choice is hidden until change 21 ships a second native. An install
updating to this build has a backup and is never asked; a new install is preset.
