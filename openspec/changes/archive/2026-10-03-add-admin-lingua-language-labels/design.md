# Design — add-admin-lingua-language-labels

## Context

`LinguaView.vue` shows the usage report's per-language breakdown (`byLanguage`) and a filter drawn
from it (`studiedLanguageOptions`). Both print `l.language`, an ISO 639-1 code from the server. The
console is translated by vue-i18n (`src/i18n/locales/fr.json`, `en.json`); `appLabel`
(`src/i18n/app-label.ts`) already turns a code into a label in the same way.

## Goals / Non-Goals

**Goals:**
- A studied language named in the console's language wherever the Lingua screen shows one.
- A language without a name still visible, as its code.

**Non-Goals:**
- Naming languages in the usage report itself, or in any proto: the codes stay the contract.
- Other screens: none shows a Lingua language.

## Decisions

### D1 — Names in the locale files, a code where none is

`lingua.languages.<code>` holds a name per language in each locale:

| Code | `fr` | `en` |
|---|---|---|
| `en` | Anglais | English |
| `es` | Espagnol | Spanish |

`languageLabel(code, t, te)` returns the name when the locale has one (`te`), and the code
otherwise. A language that reaches the report before the console names it reads as its code, as
every language does today, so nothing ever shows a missing-key marker.

A test holds both locales to the same codes, so a language is never named in one and coded in the
other.

### D2 — Values stay codes

The filter's `<option>` keeps the code as its value and shows the name. `store.filters.language`,
the request, and the URL-free state are unchanged; the breakdown's row key stays the code.

## Risks / Trade-offs

- **A language named only in code for a while** → by design (D1): the code still identifies it, and
  naming it is one line per locale.
