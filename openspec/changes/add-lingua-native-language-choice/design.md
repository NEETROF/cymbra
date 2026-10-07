# Design — add-lingua-native-language-choice

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `crates/lingua-core/src/knowledge/profile.rs` | `Profile {native_language, studied_languages}`; `set(native, languages)` changes both or neither, refuses an empty list, a duplicate, the native among the studied; `english_for_french` default |
| `crates/lingua-wasm/src/lib.rs` | `new(pack)` takes the native from the first pack; `addPack` refuses another native; `setProfile(native, languages)` errors when `native != packs.native()`; `reset` → `fresh_state(packs)` keeps the engine's native |
| `src/analyzer/engine.ts` `WasmAnalyzerPort` | `resolveNative` memoised in `native()`, `engine()` memoised, packs chosen by `pairFor(language, native)`; no rebuild path; `setProfile` declared, never called |
| `src/analyzer/pairs.ts` | `SHIPPED_PAIRS`, `pairsOf(native)`, `defaultPair(native)`, `pairFor`, `acceptedLanguages`; `DEFAULT_NATIVE = "fr"`; no `shippedNatives` |
| `src/state/storage.ts` | `storedNativeLanguage(store)` from the backup's profile, `fr` otherwise; `hydrateEngine` |
| `src/background.ts` | the rpc engine port and the sync port, each memoised; `announceStoreChange` → the scheduler; `onInstalled` opens the onboarding (best effort; Safari may not) |
| `src/reading/studied-languages-view.ts` | the studied languages' choice; `offerFor(native)` hides the block when fewer than two languages are offered; mounted in Réglages (« Langues étudiées ») and onboarding |
| `src/onboarding/onboarding.ts` | languages section, level rows, account offer; `src/popup/popup.ts` first-run level call to action (`chooseLevelPrompt`), Safari popover |
| `src/reading/settings-view.ts` | tabs « Langue », « Apparence », « Pages & livres », « Données »; `settingBlock("…")` titles read by `lint-settings-hosts` |
| `src/sync/sync.ts` `widen` | clears the cursors when the accepted languages grow |

## Goals / Non-Goals

**Goals:**
- One choice, the native language, from three places, hidden while one native ships.
- Choosing rebuilds every engine for the native and keeps the reader's state.
- A studied language that becomes the native is no longer studied; nothing else is lost.

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

### D2 — The background sets the native language, and rebuilds

A new port method, `setNativeLanguage(native)`, served by the background (the rpc host dispatches
port methods by name):
1. refuse when `pairsOf(native)` is empty;
2. read the stored backup; compute the studied languages: the current ones without `native`, or
   `defaultPair(native)`'s studied language when nothing is left;
3. build a fresh `WasmAnalyzerPort` resolving to `native`, `hydrateEngine` it from the backup
   (the restore ignores the profile's native when the engine's differs: the profile is then set
   through `setProfile(native, studied)`, which the fresh engine accepts because its packs are
   glossed in `native`);
4. save the backup, replace the background's rpc and sync ports with the fresh one (the old
   engines are dropped), write the interface-language key (change 13), announce the store change.
Everything the background answers afterwards is in the new native.

Alternative: `setProfile` on the existing engine. It refuses a native its packs are not glossed
in, by design (change 4): an engine holds one native's packs.

### D3 — A port follows the backup's native language

`WasmAnalyzerPort.restore(backup)`: when the backup's profile names a native other than the
port's engine's, the port drops its engine and its added packs, resolves to that native and
builds again before restoring. Every surface that `watchBackup`s (the side panel, the review
page, the reading session, the popup) therefore follows the background's change; the content
script's `onExternalChange` already restores and re-reads the languages. `storedNativeLanguage`
is the resolver everywhere, so a port built after the change starts on the new native.

### D4 — Three places, one view

`src/reading/native-language-view.ts` mounts the choice: the native languages of
`shippedNatives()` named each in its own language (« Français », "English", « Español » — the
same in every interface language, an entry of the catalogue), the current one selected, a change
calling `port.setNativeLanguage`. Mounted in Réglages « Langue » above « Langues étudiées »
(`settingBlock("Langue maternelle")`, a title the lint reads), in the onboarding before the
languages section — preset from `navigator.language` (`fr`, `en` or `es` when it is one of the
shipped natives, English otherwise, M13) and applied on the reader's confirmation — and in the
popup's first run, as a call to action shown before the level's when the native was never chosen
on this device (`cymbra-lingua-native-chosen` in `chrome.storage.local`, set by the three places
and by an install whose choice is hidden).

### D5 — What follows a change

The sync's accepted languages change: `widen` clears the cursors when they grow, and a pull brings
the cards of the new pairs; the cards already held in a now-unstudied language stay in the backup
and are not reviewed (the review covers the studied languages). The translation controller
reconciles on its next status (change 8 D5), downloading nothing until asked. The review opens
in the first studied language. A full reset after the change keeps the native: `reset` builds
the fresh state on the engine's native, which is the chosen one (M3).

### D6 — Copy

The French strings of the three places (« Langue maternelle », « Je lis en… », the call to
action) are written into the catalogue (change 13) with their English and Spanish, and read
through it; the surfaces that host them stay on the lint's baseline until 14, 15 and 17.

## Risks / Trade-offs

- **An engine rebuilt under a page being read** → the content script restores on
  `onExternalChange` as it does for a synced backup today; the HUD and card re-read.
- **A reader who studies only the language they choose as native** → the studied list is set to
  the native's first shipped pair's studied language, and said so in the choice.
- **Safari never opening the onboarding** → the popup's first-run call to action.
- **A device whose browser is German** → preset English (M13), the reader changes it in the same
  step.

## Migration Plan

One release, silent: the choice is hidden until change 21 ships a second native. Nothing
stored changes until a reader chooses.
