# localise-lingua-apple-host-italian — the Safari host app ready to speak Italian

## Why

Change 40 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3: Italian
speakers. On iPhone, iPad and Mac, Cymbra Lingua's Safari extension lives in a small host app
(`apps/lingua-apple`) that shows two screens of its own text. The first is the activation page, a web
view filled from `copy.js`. The second is the sign-in sheet, whose copy is `SignInCopy`. Both speak
French, English and Spanish (`localise-lingua-apple-host`). The app offers exactly the languages the
shipped packs are glossed in: its build writes them into `CFBundleLocalizations`.

The day en-it ships (change 45), that build declares Italian, and four places drop it:
- `SignInLanguage` knows only fr, en and es, so an Italian device or a stored `it` falls back to
  English;
- `SignInCopy` and `copy.js` have no Italian table;
- the native message that records the extension's language refuses `it`.

The extension and the host app ship as one bundle, so the host must accept Italian in the same build
that lists en-it. This change makes it ready ahead, inert until then.

## What Changes

- **The app's languages gain Italian** (design D1): `SignInLanguage.it` appended, so the device's
  language, the language the extension stores and the sign-in link's `lang` can be Italian. No build
  offers it before `CFBundleLocalizations` names it, which only a pair glossed in Italian does.
- **The sign-in sheet in Italian** (D2): an Italian `SignInCopy` table (11 entries, about 105 words);
  `SignInLink` reads `lang=it`; the native message accepts `it`.
- **The activation page in Italian** (D3): an `it` table in `copy.js` (15 keys, about 145 words) naming
  English, the language en-it studies. Changes 50 and 54 rewrite it to name French and Spanish.
- **The build script and its test** (D4): `app_localizations.sh` names Italian in its lists. A test
  checks that en-it gives `[fr, en, es, it]`, English as the development region, and no warning.
- **Tests** (D5): the Swift tests for Italian, and the activation page's tests, which assume every
  native has a listed pair.
- **Reviewed by the Italian reviewer agent** (D6, I2), not by the owner.
- **Released before change 45** (D7): this change's host-app release is the one change 32 requires
  live before Italian is shown. It declares `[fr, en, es]`, as today.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-apple-app`: ADDED *The host app speaks Italian once a pair glossed in Italian ships*.
  *Guided activation* and *The sign-in sheet speaks the extension's language* (held by open changes)
  already name "the languages of the shipped natives", so they are not modified.

## Impact

- **Products.** Cymbra Lingua's Apple host:
  - `apps/lingua-apple`:
    - `LinguaSignIn` (`SignInLanguage.swift`, `SignInFlow.swift`, `NativeMessage.swift`, their tests);
    - `Shared (App)/Resources/copy.js`;
    - `tool/app_localizations.sh` and its test;
    - `README.md`.
  - `apps/lingua-extension`: `test/apple-activation-page.spec.ts` and `test/native-signin.spec.ts`.

  No `pbxproj`, Info.plist or `Main.html` change. ID, Music, Live, the back office and the site are
  untouched.
- **Nothing visible changes** until change 45: every build declares French, English and Spanish, as
  today.
- **Prerequisites**: changes 32 and 33 implemented. Change 33 provides the Italian catalogue whose
  names and labels the page quotes, and the reviewer agent.
- **Order**: before change 45, released in the host app ahead of it.
- **Effort, against 1.5–3 ideal days**: 1.75–2.75 (design, *Effort*).
