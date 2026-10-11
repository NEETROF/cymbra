# Design — add-lingua-interface-language

## Context

See proposal.md (Why). Measured on main (the inventory, 2026-10-07):

| What | Where |
|---|---|
| ≈ 514 literal sites, 450 unique texts, ≈ 2,514 words, 24 TS files | `src/account/{copy,flow,view}.ts`, `analyzer/language-labels.ts`, `onboarding/level-row.ts`, `popup/popup.ts`, `reader/{copy,library}.ts`, `reading/{account-setting,book-display-view,colour-settings-view,drawer,grammar-labels,hud,selection-card,settings-view,studied-languages-view,translation-setting,wordpopup}.ts`, `review/{review-page,view}.ts`, `stats/{ladder,view}.ts`, `sync/status.ts` |
| ≈ 48 text nodes in 6 HTML pages | `popup.html`, `onboarding.html`, `account.html`, `sidepanel.html`, `stats.html`, `reader.html` (all `lang="fr"`) |
| Copy objects that exist | `reader/copy.ts` `COPY` (strings and `(title) => string`), `translation-setting.ts` `COPY`/`FAILURE`/`costText`, `account-setting.ts` `ACCOUNT_COPY`, `account/copy.ts` `errorCopy`, `sync/status.ts` `lastSyncLabel`, `language-labels.ts` `WORDS` and 11 functions |
| Plurals | none through `Intl`; « carte(s) » (`review/view.ts:46`), `carte${n > 1 ? "s" : ""}` (`stats/view.ts:299`) |
| Sentences from fragments | ≈ 12: `colour-settings-view.ts:149`, `settings-view.ts:218,388`, `review-page.ts:166`, `stats/ladder.ts:37`, `translation-setting.ts:73`, `account/view.ts:248`, `popup.html:72,77`… |
| Formats | `toLocaleString("fr-FR")` ×3, `toLocaleDateString("fr-FR")` ×2, `Intl.DisplayNames(["fr"])`, `${pct}%` and `${n} %` |
| Typography | plain spaces before « : ; ? », ` ` escapes in `stats/ladder.ts` only, « » with plain inner spaces, apostrophes ASCII ×69 and U+2019 ×11 |
| The native language | the profile in the backup, read without an engine by `storedNativeLanguage` (async); no key in `chrome.storage.local`; `DEFAULT_NATIVE = "fr"` |
| Lints | 8 `test/lint-*.spec.ts`, each `readdirSync` + a regex per line; `lint-language-labels` forbids « anglais » outside `language-labels.ts`; `lint-settings-hosts` reads the `settingBlock("…")` titles out of `settings-view.ts` |
| Tests pinning French | ≈ 641 literals in 42 spec files, asserted inline; no snapshot |
| Bundles | one entry per surface, each inlining its imports (no splitting); reader 550 KB, background 380 KB, content 286 KB, popup 99 KB |
| Precedents | the site: `const en: typeof fr` (the compiler enforces alignment); the back office: `MessageSchema = typeof en` and a key-set test |

## Goals / Non-Goals

**Goals:**
- Every French text of the interface exists once, in the catalogue, byte for byte.
- English and Spanish drafts exist for every key, and cannot drift from the French keys.
- The interface language is readable by every surface before its first paint.
- Nothing on screen moves.

**Non-Goals:**
- Switching any surface to the catalogue (changes 14–17), or any HTML page.
- The choice of the native language (change 20); the manifest's `_locales` (27); the host app
  (28); the agent plugin (55).
- A typography pass on the French (M23: separate, if ever).
- Translating the card's tense names and form descriptions (change 18) or the languages' names
  (change 19): their French entries are extracted here; their English and Spanish entries are
  drafted here and settled there.

## Decisions

### D1 — One module per surface and language, the French the source, typed `typeof fr`

`src/i18n/fr/<surface>.ts` exports `export const popup = { … }` (no `as const`: the values must
be typed `string`, or the English text would have to equal the French literal); `src/i18n/en/
<surface>.ts` exports `export const popup: typeof fr.popup = { … }`; likewise `es`. A key missing
in English or Spanish does not compile, as on the site (`const en: typeof fr`). Plural forms are
objects `{one, many?, other}` and slot messages are functions, both typed by the French module.
`src/i18n/index.ts` exports the helpers (`plural`, `formatNumber`, `formatDate`) and the types;
it maps no surface, so importing it costs no copy. A surface imports `../i18n/{fr,en,es}/
<surface>.ts` itself (changes 14–17) and picks by the interface language. Keys are named by
meaning (`review.revealAnswer`, not `afficherLaReponse`), in English, so a French text can change
without renaming a key.

Why per surface: each entry bundles what it imports and nothing is split, so the popup carries
the copy of the popup and of what it hosts (Réglages' blocks), and not the reader's; the reader
(550 KB today) grows by its own copy in three languages, ≈ 10 KB.

Alternative: one file per language, or a map of every surface. Every entry would carry every
surface three times; the popup would grow by the whole catalogue.

### D2 — Slot messages and plurals

A sentence built from fragments becomes one function of its parts: `settings.knowCommonest(n)`
→ « Je connais les ${n} mots les plus courants » with the bold part marked by a slot the surface
renders (`{n}` wrapped as the surface decides), so that English can put the number elsewhere. A
count goes through `plural(language, n, forms)`, where `forms` is `{one, many?, other}` —
`Intl.PluralRules`' categories for these three languages: French and Spanish have `many` (for
exact millions, in current CLDR), English has not; a missing `many` falls back to `other`. Each
form is a function of the formatted number. In French every form of a message carries the same
string, the current one — « ${n} carte(s) à revoir » — and the number is the raw count the surface
writes today (no grouping: `Intl` would write « 1 234 » with a no-break space, and M23 forbids
it); English and Spanish forms receive `Intl.NumberFormat`'s number.

### D3 — The interface language is the native language, under its own key

`INTERFACE_LANGUAGE_KEY = "cymbra-lingua-interface-language"` (in `src/i18n/language.ts`, with
`InterfaceLanguage` and `setDocumentLanguage(document, language)`) in `chrome.storage.local`,
holding `fr`, `en` or `es`; `interfaceLanguage(area)` reads it, absent or unknown meaning `fr`
for a device that predates the key (M22: existing installs stay French; change 20 presets a new
install from the browser's language and writes the key). The store's owner — the background,
which alone writes the backup (`state/store.ts`) — alone writes the key, at two moments: at its
start, when the key is absent, from the stored backup (a device updated with a profile already
stored; a key already held is kept, with no read of the backup, since the background wakes often
and every surface's first read waits behind it); and after each write of the backup — a profile
change, a restore from a file, a full reset and the first hydration all go through that one
write — off the write path, once for a burst of writes, and only when the language differs from
the one last written, so the key cannot go stale behind the profile and a word's status change,
which rewrites the backup, costs no write of it. The key holds the native language as the
extension serves it — `nativeLanguageOf(backup, pairs)`'s gate, the one the reading engine
applies: French when no listed pair is glossed in the profile's language (M22), so the interface
speaks the language the glosses are written in.
The key mirrors the profile (M2: one choice) and a later override (M2's reservation) is a
different key, with no migration. The preferences every surface reads from `chrome.storage.local`
before it renders its copy (`loadEnabled`, the colours, the HUD state) are the model: a surface
awaits `interfaceLanguage` with them, and sets `document.documentElement.lang`. The static French
of the HTML pages paints before any script; it moves to the surfaces' modules with changes
14–17, which is when those pages stop saying `lang="fr"`.

Why a key of its own and not the backup: the backup lives in the background's IndexedDB and is
read through a message round-trip; the popup paints before it has an engine, and the account page
never has one.

### D4 — Formats follow the interface language; French keeps its strings

`formatNumber(language, n)` and `formatDate(language, d)` use `fr-FR`, `en-US`, `es-ES` for
English and Spanish. The French strings that `Intl` would change — « 25,8 Mo » with its plain
space where `Intl` would put a no-break one, « il y a 3 min. » with its full stop, a raw count —
stay as the surfaces write them today, in the catalogue. Percentages keep each surface's current
form in French.

### D5 — The lint, with a baseline that cannot go stale

`test/lint-copy.spec.ts` reads each `src/**/*.ts` (not `src/i18n/`, `gen/`, `pkg/`) with the
TypeScript compiler's parser (`typescript` is a dev dependency) and visits string and template
literals only — comments and regular-expression literals do not count, which is why `selection.ts`,
`reconcile.ts`, `speech.ts` and the French in `port.ts`'s and `sidepanel.ts`'s comments are not
hits — and the HTML pages' text nodes and attributes; a literal is French when it holds an
accented letter, « », ’ or one of the inventory's unaccented French words as a whole word
(« Annuler », « Fermer », « Réglages », « Mots inconnus »…). The file fails unless it is on the
baseline, a list in the test of the files that hold literals today; the baseline is checked the
other way too: a file on it that holds no literal fails, so each of changes 14–17 removes its
files. ASCII French with none of the listed words is beyond the lint — the baseline, shrinking
by whole files whose literals were moved by reading, is the real guard. `lint-language-labels`
admits `src/i18n/`.

### D6 — Parity and honesty tests

`test/i18n.spec.ts`: for every surface, `en` and `es` have the French key set (the compiler says
so; the test says it at runtime for the plural forms and the slot functions, which types describe
loosely); no English or Spanish value is empty; no value equals its French one unless the French
is in the list of texts that are the same in every language (« OK », « Cymbra Lingua », a model
size…); every slot a French message takes is taken by its English and Spanish messages (the
functions are called with sample arguments and the result must contain each argument). The
French catalogue is compared with the inventory's literals by a one-off script in this change's
pull request, not kept as a test: once the surfaces have moved (14–17), the 42 spec files that
assert French copy are the check.

### D7 — Drafts, terms and register

The English and Spanish drafts follow `src/i18n/README.md`: US English; Spanish with tú, neutral,
no vosotros; « forma en -ing »; RAE tense names; CEFR in English and MCER in Spanish (M19); RAE
numbers (20 000; 96 %); the French unchanged, tu/vous as it is today. The owner reviews both drafts
in the pull request (M9). Where a French message quotes a Réglages path or a button, the English
and Spanish quote their own catalogue's.

## Risks / Trade-offs

- **A French byte moves** → none is on screen yet; the one-off comparison of the catalogue with
  the inventory's literals runs in the pull request, and the 42 spec files guard the surfaces.
- **A key the inventory missed** → the lint's baseline names the file; change 14–17 find it when
  they take the file off the baseline.
- **Drafts that read like translations** → the owner's review (M9), and change 33's dogfood;
  goldens per pair bound the card's wording (18).
- **Bundle growth** → per-surface modules (D1): measured in the pull request per entry.
- **`lint-settings-hosts` reading titles out of `settings-view.ts`** → unchanged here (the titles
  stay literals until change 15, which moves the test to read the catalogue).

## Migration Plan

One release, silent. The key is written at the background's first start after the update, from
the stored profile, when the device holds none; a device that never writes it reads `fr`.
