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
catalogue's do. The brand stays a literal in `name` and `action.default_title`. Each language's
description names the languages a reader of that native language can study in the shipped pairs —
Spanish in `en` (es-en), English in `es` (en-es) — and is not a translation of the French; it
changes when a pair of that native is added (fr-en, fr-es).

### D2 — The build writes the localised manifest only when a non-French native ships

The manifest step moves to `tool/manifests.mjs` — pure: base manifest, target, pairs and messages
in, the manifest and the folders to copy out — imported by `build.mjs`; `shippedNatives` gets a
`tool/packs.mjs` twin of change 20's `pairs.ts` helper, held equal by `test/pairs.spec.ts`.
`shippedNatives(PAIRS)`: when it is `["fr"]`, the built manifests keep today's literal French and nothing is copied — byte for byte
today's packages. Otherwise the build replaces the four strings with `__MSG_<key>__`, copies
`_locales/<native>/` for each shipped native, and sets `default_locale` to `en` when `en` is among
them, else `fr`. A language with a folder but no shipped pair is never copied, so a browser in
that language reads the default.

Alternative: ship `_locales/fr` and `default_locale: "fr"` now. Nothing visible would move, but
every package's manifest would, and the localised path would ship before any reader needs it.

### D3 — The checks

`check_version.mjs` reads the base manifest's literal description and each committed
`_locales/*/messages.json`'s `extensionDescription`, and refuses any over 112 characters, naming
the language; it also refuses a `_locales/fr/messages.json` whose `extensionDescription` or command
messages differ from `manifest.json`'s literal `description` and `commands.*.description`, naming
the key — the French has two homes, held equal. `check_variants.mjs`: a manifest with a `__MSG_`
reference carries `default_locale` and `_locales/<default>/`; every reference resolves in every
shipped folder; a manifest without references carries neither; when the manifest is localised, the
shipped folders are exactly the shipped natives. Chrome's manifest reference and MDN require
`default_locale` exactly when `_locales` exists; Safari documents nothing, and the check holds it to
the same rule. A unit test runs `tool/manifests.mjs` with a pair list that holds es-en and asserts
the localised variant for each target.

### D4 — What is verified, and by whom

Before change 34, with es-en added to a local, uncommitted `packs.json`, the three packages are
loaded with the browser in English, German and French: chrome://extensions and its shortcuts page,
about:addons and its shortcut manager, Safari's Extensions settings on macOS and iOS. Each shows the
expected description and no `__MSG_` text; Safari shows no command descriptions, so that half is not
checked there. The owner validates a localised Safari archive (Transporter or `xcrun altool
--validate-app`, nothing delivered) before change 34 merges — Apple's check at upload is the one
that binds, and a failure there must not wait for a release.

## Risks / Trade-offs

- **Safari or Apple's upload reads `__MSG_` differently** → found by D4's validation before change
  34 merges; the build can then fall back to literal text per variant.
- **`default_locale: en` re-keys the store listings** → the Chrome Web Store takes the listing's
  default language from `default_locale` and a summary per `_locales` folder; addons.mozilla.org
  reads `__MSG_` per locale at upload, and whether it rewrites an existing listing's summary and
  default locale is undocumented. Change 36 re-enters the French listing under `fr` and fills `en`;
  the owner checks both dashboards after change 34's first submission and records the result in
  `STORE-LISTING.md`.
- **A stale `_locales` inside the Safari app extension** → the host app's copy phase `rsync -a`
  keeps files; it removes `_locales/` from the extension before copying `dist-safari`.
- **An English summary before English readers can use the product** → D2 ships nothing until an
  English-glossed pair does.
- **Commands described in two languages** (the browser's page in the browser's language, Réglages
  in the reader's) → accepted: the browser's page follows the browser; the browser's pages are not
  the interface M22 keeps French, so a French reader whose browser is in English sees the English
  description once es-en ships.

## Migration Plan

No release changes: every package built while only French-native pairs ship is byte for byte
today's. The first localised package is change 34's, submitted only together with change 36's
English listing (M13); no store submission is made between their merges (M18).
