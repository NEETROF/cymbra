# Design — localise-lingua-apple-host-italian

## Context

Measured on main at `e2564d9a6`. `swift test` in `LinguaSignIn` (73 tests) and
`tool/test_app_localizations.sh` (31 cases) pass. Paths are under `apps/lingua-apple/` unless
prefixed `ext:` (`apps/lingua-extension/`).

**What the app shows:**

| Surface | Copy | Language chosen by |
|---|---|---|
| Activation page (iOS, macOS) | `Shared (App)/Resources/copy.js`, one table per language: 15 keys (12 nodes, 3 pre-macOS-13 variants), 159 / 158 / 166 words in fr / en / es; the French also in `Base.lproj/Main.html`, held equal by `ext:test/apple-activation-page.spec.ts` | `ViewController.swift` injects `window.linguaLanguage`, from `SignInLanguage.shown(InterfaceLanguageStore.current(), in: .main)`: the language the extension stored in the App Group, else the device's preferred localisation among `CFBundleLocalizations`, else English once offered, else French |
| Extension's sign-in sheet | `SignInCopy` in `LinguaSignIn/Sources/LinguaSignIn/SignInFlow.swift`, 11 entries, 97 / 100 / 109 words | `SignInLanguage.shown(link, in: .main)` (`SceneDelegate.swift`, macOS `AppDelegate.swift`): the link's `lang` (`SignInLink.language(from:)`) when offered, then the same chain as the page |
| App's own sign-in sheet (from the review page) | the same `SignInCopy`, 7 entries | the App Group language, as the page |
| In-app review page | the extension's `app` build, `ext:src/i18n/<lang>/` | change 33's, through the extension's interface language, with `window.linguaLanguage` as its fallback (`ext:src/app/app.ts`) |
| macOS menu bar | `macOS (App)/Base.lproj/Main.storyboard`, 7 English titles | nothing: English for every reader |
| System-drawn | Sign in with Apple's button, the web-authentication consent, StoreKit's rating prompt | the system, from the bundle's localisations |

**Where `it` is dropped today:**
- `SignInLanguage.swift` declares `fr`, `en` and `es`, and everything goes through
  `init(rawValue:)`. So `offered(by:)` drops `it` from `CFBundleLocalizations`, `preferred` maps an
  Italian device to the development region, and `InterfaceLanguageStore.current()` reads a stored
  `it` as nothing.
- `SignInFlow.swift`: `SignInLink.language(from:)` reads `lang=it` as no language. `SignInCopy` has
  an exhaustive switch and three tables.
- `NativeMessage.swift` answers `interface.language` = `it` with `unknownLanguage`.
- `copy.js` has three tables. An unknown language leaves `Main.html`'s French; every table holds every key, which the page test checks (a missing key would render « undefined »).
- `tool/app_localizations.sh` already accepts any native: with `en-it.lingua` it writes
  `["fr","en","es","it"]` and region `en`. It also warns "it-native packs ship, but the app has no copy
  in it", and its order list and a comment name fr, en and es.

**What the open changes do here:**
- Changes 20 and 25 rewrite `copy.js`'s `fr` and `en` `lede` and `step3` to name Italian.
- Change 32 splits `InterfaceLanguage` from `NativeLanguage`; its D4 needs a host-app release
  carrying it before change 45.
- Change 33 writes `ext:src/i18n/it` and restores the alias; the Apple host is its non-goal.
- Change 39 writes `_locales/it`.
- Change 45 lists en-it, so `CFBundleLocalizations` gains `it` with no further code, and the
  extension's interface languages gain `it`.
- Until change 45, the extension never sends `it`: a stored `it` reads as French, so the link says
  `lang=fr`, and Safari's language message never names Italian.

## Goals / Non-Goals

**Goals:**
- The build that lists en-it shows the activation page and both sign-in sheets in Italian to an Italian
  reader, with no other code to write.
- Nothing visible moves before then.

**Non-Goals:**
- The App Store listing (change 46; I9: shipped with change 45's Apple submission).
- The macOS menu bar (D8).
- Display names and system-drawn text.

## Decisions

### D1 — `SignInLanguage.it`, appended

`SignInLanguage` gains `it` after `es`, as every enumeration of languages appends. `offered(by:)`,
`preferred` and `InterfaceLanguageStore.current()` then keep `it` like the others. The fallback rule
(the stored language if offered, then the device's, then English once offered, then French) is
unchanged. The doc comments that name `fr|en|es` gain it.

What a build offers is read from `CFBundleLocalizations` alone, which the "Declare Lingua languages"
build phase writes from the packs in `dist-safari/assets/packs`. So no build offers Italian until a pair
glossed in Italian ships, and the Italian tables of D2 and D3 stay unreachable until change 45. The pack
split (change 4) keeps each pair's layer at the pair's path, so the derivation still holds.

### D2 — The sign-in sheet's Italian

- `SignInCopy` gains its `.it` case and table: the 11 entries of the others, about 105 words.
- `SignInLink.language(from:)` reads `lang=it`.
- `NativeMessage` accepts `interface.language` = `it` and stores it in the App Group.
- The sheet's `Locale(identifier:)` follows unchanged.

A failure names its provider and never reads as a password error (the existing test loops over
`allCases`, so `.it` is covered). Apple's button keeps the label the system gives it.

### D3 — The activation page's Italian

`copy.js` gains an `it` table of its 15 keys, about 145 words, no longer than the Spanish page's 166:
the iOS page does not scroll. It names English only, the language
en-it studies, as the English and Spanish pages first named one language, and as `_locales/it` does
(change 39). The language names are lowercase with their articles (« l'inglese », « il tuo livello di
inglese »), and the page never names « italiano ».

Changes 50 and 54 rewrite its `lede` and `step3` to name French and Spanish; their programme rows say
so. `Main.html` keeps the French, and the header comment of `copy.js`, which names each native's pairs,
gains Italian.

### D4 — The build script

`app_localizations.sh` names Italian in its warning list and its order list (`fr, en, es, it`). The
region rule (English while English ships) is unchanged. `test_app_localizations.sh` gains a case:
en-it among the packs gives `["fr","en","es","it"]`, region `en`, and no warning.

### D5 — Tests

- **Swift**:
  - `SignInLanguageTests`: Italian offered and preferred; a stored `it`; `NativeMessage`'s Italian
    case (its tests live there);
  - an **inert** test: a bundle declaring `[fr, en, es]` (the file's bundle helper) never shows `.it`,
    even when `.it` is requested;
  - `SignInFlowTests`: `lang=it` parsed; the Italian draft is not the French; Italian addresses the
    reader with « tu »;
  - `AppSessionStoreTests`, which lists `[.fr, .en, .es]` for `appLede` and `emailButton`: Italian
    among them.
- **`ext:test/apple-activation-page.spec.ts`**:
  - `LANGUAGES` gains `it` (the test of the languages the table holds fails as soon as `COPY` does);
  - the "no copy of the French" check and the fill loop cover Italian;
  - a « tu » check like Spanish's, refusing « aprite », « attivate », « scegliete », « Apra », « Attivi »,
    « Scelga »;
  - the naming checks run against the shipped pairs plus en-it
    (`pairsOf("it", [...new Set([...SHIPPED_PAIRS, "en-it"])])`), so Italian's page is pinned at this
    change's merge and the test still fails changes 50 and 54 until they rewrite it;
  - the check that step 3 says where a second language is ticked (`drawer.settings`,
    `settings.tabLanguage`) applies only to natives with two listed pairs or more. A native with one
    pair is held to naming that language's level and no Settings tab, as the one-language pages of the
    precedent were. `named1()` is not called for a native with no pair.
- **`ext:test/native-signin.spec.ts`**: the teller does not send `it` while `INTERFACE_LANGUAGES` lacks
  it, which keeps this change inert. Today `NativeMessage` refuses `it`; after this change it stores it,
  so the guarantee moves to the extension, and the test pins it.

### D6 — Reviewed by the Italian reviewer agent

The two tables are reviewed in the implementation pull request by change 33's agent (I2). The owner is
not asked; change 44 rereads them. Its points:
- « tu », imperatives (« Apri », « Attiva », « Scegli », « Accedi », « Riprova », « Annulla »), and no
  gendered participle (« connesso/a »);
- Italian typography (« » without inner spaces, no space before « : »);
- product terms from Music's `app_it.arb` (« Accedi », « Annulla », « Riprova », « Continua con
  Google/Apple », « Continua con l'email »), and the review vocabulary from change 33's catalogue;
- the iOS and macOS paths (« Impostazioni → App → Safari → Estensioni », « Impostazioni di Safari »
  from macOS 13, « Preferenze » before), cited from a source or marked unverified, as change 33's D1
  does for menu paths.

### D7 — Released before change 45

Change 32's D4 requires a host-app release carrying change 32 to be live before change 45. This
change's release (its implementation commit, which touches `apps/lingua-apple`, bumps the host app) is that
release at the latest, and carries this change's tables with it. It declares `[fr, en, es]`, as the
build's natives check guarantees, so nothing visible changes and Apple validates the same shape it
accepts today. The release is the owner's (I12).

release-please assigns a commit to a component by the paths it touches, not by its scope
(`release-please-config.json` has no plugins or excluded paths). This change's implementation touches
`apps/lingua-apple` and `apps/lingua-extension/test`, so its commit bumps both the host app and the
extension, which change 32's D4 needs anyway. Only change 32 needs a host-app release before change 45;
this change's code ships in the same bundle as en-it. Once it lands, change 45 may touch nothing under
`apps/lingua-apple`, and then release-please would cut no host-app release for en-it; row 45 records
that change 45 touches `apps/lingua-apple` or carries `Release-As`.

### D8 — Not here

- **The macOS menu bar** stays English for every reader, as it is today for French, English and
  Spanish readers. Translating it (a Swift table applied at launch, about 0.25–0.5 day for four
  languages) would change what French readers see, which is the owner's call (*Open Questions*); it
  belongs to a change of its own if wanted.
- **Whether `preferredLocalizations` honours `CFBundleLocalizations` without `.lproj` folders, and
  which `_locales` Safari picks** (the precedent's task 4.5, still open): checked for Italian in change
  45's dogfood, with a device in Italian.
- **The Sign in with Apple button in another language than the sheet** (an Italian device with an
  English App Group value): a risk the precedent carried.

## Risks / Trade-offs

- **Italian unreachable until change 45, untested on a device** → the Swift and page tests exercise
  the tables; change 45's dogfood reads them on a device.
- **A page naming only English after fr-it ships** → rows 50 and 54 rewrite it; the page's test names
  every studied language a native's listed pairs study, so it fails them until they do.
- **Italian written without a native reader** → the reviewer agent, change 44, and change 55 for
  dogfood corrections.

## Migration Plan

Nothing deploys. The host-app release (D7) is the owner's. Rollback is a revert.

## Open Questions

For the owner, not blocking this change:
1. **The macOS menu bar**: English for every reader, as today, or its seven titles translated into the
   four languages? The precedent deferred it to matrix change 33's review, which recorded nothing.

## Effort

1.6–2.75 ideal days, against the programme's 1.5–3:

| Part | Ideal days |
|---|---|
| Swift: `.it`, the `SignInCopy` table, comments | 0.25 |
| `copy.js`'s Italian table and header | 0.25–0.5 |
| The build script and its test | 0.1–0.25 |
| The Swift and page tests | 0.5–0.75 |
| The reviewer agent's pass | 0.25–0.5 |
| README, spec, programme | 0.25–0.5 |
