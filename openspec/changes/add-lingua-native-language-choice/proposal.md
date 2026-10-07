# add-lingua-native-language-choice — the reader chooses their native language, once two are shipped

## Why

Change 20 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the last of stage 1, decisions M2, M3 and M22. The native language exists on the device since
change 4 — in the profile, one per engine, read by every port — and nobody can choose it: the
engine's `setProfile` has no caller, the profile's native is whatever the first pack is glossed
in, and every shipped pack is glossed in French. The interface language (change 13) mirrors it;
the translation pairs (change 8), the review language and the sync's accepted languages follow
it; the card's gloss language (change 11) is written from it.

This change gives the reader the choice, in three places: the first question of the onboarding,
preset from the browser's language; the popup's first-run call to action, because Safari does
not always open the onboarding tab; and Réglages, « Langue ». The choice is hidden while only
one native language has a shipped pair — today — so nothing a reader sees moves, and an
installed extension keeps its reader in French without asking (M22). It appears with the first
pair glossed in another language (change 21, es-en), without a release of its own.

Choosing a native language changes which packs an engine holds: a pack is glossed in one native
language, and an engine holds the packs of one. So the choice rebuilds the engines — the
background's, each surface's — for the new native, restoring the reader's state into them. It
also settles what happens to a studied language that is the native chosen: it is no longer
studied.

## What Changes

- **A background command, `setNativeLanguage(native)`**: refused when no shipped pair is glossed
  in it; otherwise the background builds a fresh engine for that native, restores the stored
  backup into it with the profile set to (native, the studied languages minus the native — or
  the native's first shipped pair's studied language when nothing is left), saves the backup,
  replaces its own engines (the rpc one, the sync one), writes the interface-language key (change
  13), and announces the store change.
- **Ports follow the backup's native language**: a port whose restored backup names another
  native than its engine's rebuilds its engine for it, as a restore already reloads the state;
  every surface's `watchBackup` therefore rebuilds, and the page reads in the new language.
- **The choice, in three places**, each shown only when `shippedNatives()` counts two or more:
  the onboarding's first question (« Je lis en… » — the native languages named each in its own
  language — preset from `navigator.language` when it is one of them, English otherwise, M13);
  the popup's first-run call to action, before the level's; Réglages, « Langue », a block above
  « Langues étudiées ».
- **What follows**: the sync's accepted languages (`widen` when they grow), the translation pairs
  (the controller reconciles), the review language (the first studied language), the level
  prompts; a full reset keeps the native language (M3: `reset` builds the fresh state on the
  engine's native).
- **A new device** asks at onboarding (M3); the profile is never synced.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *The reader chooses their native language*. *A reader is
  served the pairs of their native language* (held by no open change) stands: "an installed
  extension SHALL keep its reader in French without asking" is what the hidden choice does.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (`src/background.ts`, `src/analyzer/
  {engine,port,messaging-port,pairs}.ts`, `src/state/storage.ts`, `src/onboarding/**`,
  `src/popup/popup.ts`, `src/reading/settings-view.ts`, a `native-language-view.ts`, tests). The
  engine's `setProfile` and `reset` are already what this needs. ID, Music, Live, the back office
  and the site are untouched.
- **No byte moves**: the choice is hidden while one native ships; the French copy of the three
  places is in the catalogue (changes 14, 15, 17 move those surfaces; this change writes its new
  French strings into the catalogue directly, with their English and Spanish).
- **Order.** After changes 13 (the key), 14/15/17 for the surfaces it touches (or it writes its
  strings into the catalogue beside the surfaces' literals, on the lint's baseline), 19 (the
  languages' names). Change 21 ships the first second native.
- **Not here.** The studied languages' choice (exists); the native language on the server (12);
  what an open page's answer memory does across a change of native (the engines are rebuilt, the
  page's memory is per page and goes with it).
