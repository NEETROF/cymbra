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

`src/i18n/fr/<surface>.ts` exports `const popup = { … } as const` (and so on per surface);
`src/i18n/en/<surface>.ts` exports `const popup: typeof fr.popup = { … }`; likewise `es`. A key
missing in English or Spanish does not compile, as on the site. `src/i18n/index.ts` exports
`copyFor(surface, language)` and the `Surface` and `InterfaceLanguage` types. Keys are named by
meaning (`review.revealAnswer`, not `afficherLaReponse`), in English, so a French text can change
without renaming a key.

Why per surface: each entry bundles what it imports and nothing is split, so the popup carries
the popup's three languages (≈ 3 KB) and not the reader's; the reader (550 KB today) grows by
its own copy in three languages, ≈ 10 KB.

Alternative: one file per language. Every entry would carry every surface three times; the
popup would grow by the whole catalogue.

### D2 — Slot messages and plurals

A sentence built from fragments becomes one function of its parts: `settings.knowCommonest(n)`
→ « Je connais les ${n} mots les plus courants » with the bold part marked by a slot the surface
renders (`{n}` wrapped as the surface decides), so that English can put the number elsewhere. A
count goes through `plural(language, n, forms)`, where `forms` names `Intl.PluralRules`'
categories (`one`, `other`, and `many` where a language has it); the helper picks the category
for the language and formats the number with the language's locale. In French every category of
a message carries the same string, the current one — « ${n} carte(s) à revoir » — so no French
byte moves and `many` (French's category for exact millions) needs no form of its own.

### D3 — The interface language is the native language, under its own key

`INTERFACE_LANGUAGE_KEY = "cymbra-lingua-interface-language"` in `chrome.storage.local`, holding
`fr`, `en` or `es`; `interfaceLanguage(area)` reads it, absent or unknown meaning `fr` (M22:
existing installs stay French). The background writes it whenever the profile's native language
is read or set (`storedNativeLanguage`, the rpc engine's `setProfile`), so the key mirrors the
profile (M2: one choice) and a later override (M2's reservation) is a different key, with no
migration. The preferences every surface reads from `chrome.storage.local` before its first paint
(`loadEnabled`, the colours, the HUD state) are the model: a surface awaits `interfaceLanguage`
with them.

Why a key of its own and not the backup: the backup lives in the background's IndexedDB and is
read through a message round-trip; the popup paints before it has an engine, and the account page
never has one.

### D4 — Formats follow the interface language; French keeps its strings

`formatNumber(language, n)` and `formatDate(language, d)` use `fr-FR`, `en-US`, `es-ES`. The
French strings that `Intl` would change — « 25,8 Mo » with a no-break space, « il y a 3 min. »
with its full stop — stay as they are, written in the catalogue; English and Spanish use `Intl`.
Percentages keep each surface's current form in French.

### D5 — The lint, with a baseline that cannot go stale

`test/lint-copy.spec.ts` walks `src/**/*.ts` and the HTML pages (not `src/i18n/`, `gen/`, `pkg/`)
and fails on a French literal — a string holding an accented letter, « », ’ or one of the
inventory's unaccented words (« Annuler », « Fermer », « Réglages »…) — unless the file is on the
baseline, a list in the test of the files that hold literals today. The baseline is checked the
other way too: a file on it that holds no literal fails, so each of changes 14–17 removes its
files. Data that is not copy (a gloss in a test, a Wiktionary note) lives in tests or data, not in
`src/`, so the rule has no exception list beyond the baseline. `lint-language-labels` admits
`src/i18n/`.

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

One release, silent. The key is written at the background's first read of the profile; a device
that never writes it reads `fr`.
