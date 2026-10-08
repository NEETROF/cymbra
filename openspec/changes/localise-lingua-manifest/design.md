# Design — localise-lingua-manifest

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `apps/lingua-extension/manifest.json` | `name` and `action.default_title` « Cymbra Lingua »; `description` (French, 109 characters); `commands.{lingua-capture-selection, lingua-toggle-drawer, lingua-side-panel}.description` (French); no `default_locale`, no `_locales`, no `__MSG_` anywhere |
| `build.mjs` | reads the base manifest; `firefoxManifest` and `safariManifest` derive the variants (Safari keeps name, description, title and commands); stamps the version; writes `dist-<target>/manifest.json`; copies `staticCopies` and `icons/` — nothing else; `PAIRS` from `packs.json` (`en-fr`, `es-fr`) |
| `tool/check_version.mjs` | `MAX_DESCRIPTION = 112` (Apple validates the Safari extension's manifest at upload; Chrome allows 132); reads `manifest.description` as a literal |
| `tool/check_variants.mjs` | per-variant manifest checks (service worker, side panel, gecko id, data collection, Safari shape, icons) — none of name, description, commands, locales |
| Stores | `STORE-LISTING.md`: the summary is not editable in the dashboards, both stores take the manifest's description; the CWS upload (API V2) and AMO's signing send no listing |
| Safari host | the "Copy Lingua extension" phase rsyncs `dist-safari` into each extension's bundle, a `_locales/` folder included |
| Réglages | describes the same three commands in the interface language (change 15) and opens the browser's shortcuts page, where the manifest's descriptions show |

## Goals / Non-Goals

**Goals:**
- The browser shows the extension's description and commands in its own language when a pair is
  glossed in that language.
- Nothing moves while every shipped pair is French-native.

**Non-Goals:**
- Following the reader's chosen language in the browser's own pages: `_locales` cannot.
- The store listings' long descriptions and screenshots (36, 37); the host app (28).
- Localising the brand name.

## Decisions

### D1 — Three committed message files, French byte for byte

`_locales/fr/messages.json` holds `extensionDescription`, `commandCaptureSelection`,
`commandToggleDrawer` and `commandSidePanel` with today's French text, each with a `description`
for translators; `en` and `es` hold the drafts (M9, M10). Keys name meaning, in English, as the
catalogue's do. The brand stays a literal in `name` and `action.default_title`.

### D2 — The build writes the localised manifest only when a non-French native ships

`shippedNatives(PAIRS)` (the natives of `packs.json`'s pairs, as `pairs.ts` derives them): when it
is `["fr"]`, the built manifests keep today's literal French and nothing is copied — byte for byte
today's packages. Otherwise the build replaces the four strings with `__MSG_<key>__`, copies
`_locales/<native>/` for each shipped native, and sets `default_locale` to `en` when `en` is among
them, else `fr`. A language with a folder but no shipped pair is never copied, so a browser in
that language reads the default.

Alternative: ship `_locales/fr` and `default_locale: "fr"` now. Nothing visible would move, but
every package's manifest would, and the localised path would ship before any reader needs it.

### D3 — The checks

`check_version.mjs` reads the base manifest's literal description and each committed
`_locales/*/messages.json`'s `extensionDescription`, and refuses any over 112 characters, naming
the language. `check_variants.mjs`: a manifest with a `__MSG_` reference carries `default_locale`
and `_locales/<default>/`; every reference resolves in every shipped folder; a manifest without
references carries neither; the shipped folders are exactly the shipped natives. A unit test runs
the manifest step with a pair list that holds es-en and asserts the localised variant for each
target.

### D4 — What the owner verifies

The first package that carries `_locales` is change 34's: the owner opens it in Safari on a device
in English (the description and the commands' names in Safari's settings) and uploads it; Apple's
check of the description at upload is the one that binds. Firefox and Chromium resolve `__MSG_` in
the manifest as documented; the extension tests assert the built files.

## Risks / Trade-offs

- **Safari or Apple's upload reads `__MSG_` differently** → found at change 34's first upload; the
  build can then fall back to literal text per variant, a one-line change.
- **An English summary before English readers can use the product** → D2 ships nothing until an
  English-glossed pair does.
- **Commands described in two languages** (the browser's page in the browser's language, Réglages
  in the reader's) → accepted: the browser's page follows the browser.

## Migration Plan

No release changes: every package built while only French-native pairs ship is byte for byte
today's. The first localised package is change 34's.
