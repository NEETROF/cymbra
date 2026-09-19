# Store listings — Cymbra Lingua

Copy for the Chrome Web Store and addons.mozilla.org listings, kept here so the answers stay
the same in both dashboards and so a change to the extension can change its listing in the
same pull request.

Nothing here is submitted by CI: `lingua-extension-release` uploads a **new version of an
existing item**, and the listing itself is filled once by hand. What follows is what to paste.

Constants:

| Field             | Value                                                                                                |
| ----------------- | ---------------------------------------------------------------------------------------------------- |
| Name              | Cymbra Lingua                                                                                        |
| Homepage          | `https://cymbra.app/lingua/`                                                                         |
| Privacy policy    | `https://cymbra.app/confidentialite/` (EN: `https://cymbra.app/en/privacy/`) — Annex B covers Lingua |
| Support           | `https://cymbra.app/support/`                                                                        |
| Category          | Chrome: _Education_ · AMO: _Language support_                                                        |
| Listing language  | French — the interface is French, and the product teaches English to French speakers                 |
| Firefox add-on id | `lingua@cymbra.app`                                                                                  |

---

## Summary (short description)

**Not editable in the dashboards** — both stores take it from the package's `manifest.json`
`description`, where it is:

> Lisez l'anglais sur le web : les mots inconnus surlignés, et un pourcentage honnête. Hors ligne, privé, sans compte requis.

105 characters. **The limit is 112, Apple's** — checked when the signed archive is uploaded to
App Store Connect, which is where `lingua-apple-v1.1.0` died after a full build. Chrome allows
132, so calibrating on Chrome produces an archive Apple refuses. `yarn check:version` holds the
112 now.

Changing it means changing `manifest.json` and shipping a new package, so it is worth getting
right before a submission.

It is in **French** because the extension is: its whole interface is French ("Analyser cette
page", "Je connais", "Toujours surligner"), and it teaches English _to French speakers_. An
English summary would send English speakers to an interface they cannot read. The listing's
language field is French for the same reason.

## Description

**FR**

Cymbra Lingua surligne, sur la page que vous lisez, les mots d'anglais que vous ne connaissez
pas encore — sans rien changer à la mise en page. Un pourcentage vous dit quelle part du texte
vous est familière, calculée sur ce que vous avez réellement marqué, pas sur une estimation.

Cliquez un mot surligné : sa traduction, sa forme du dictionnaire et sa rareté en anglais courant. Puis décidez : « Je connais »,
« + Deck » pour le réviser plus tard, ou « Ignorer ». Sélectionnez plusieurs mots et appuyez
sur Alt+L pour capturer une expression entière avec sa phrase.

Les cartes que vous créez se révisent dans un panneau, à côté de votre lecture ou dans la barre
latérale, avec une répétition espacée qui décide toute seule du bon moment.

**L'analyse est locale.** Le dictionnaire et le moteur tournent dans votre navigateur : aucune
page que vous lisez n'est envoyée nulle part, et l'extension fonctionne hors ligne. Sans
compte, elle ne fait aucune requête réseau.

Créez un compte Cymbra si — et seulement si — vous voulez retrouver vos mots et vos cartes sur
vos autres appareils. C'est la seule chose qui quitte votre machine, et vous pouvez effacer ces
données depuis les Réglages sans supprimer votre compte.

Cymbra Lingua existe aussi sur Firefox et sur Safari (iPhone, iPad, Mac).

**EN**

Cymbra Lingua highlights, right on the page you are reading, the English words you do not know
yet — without changing the layout. A percentage tells you how much of the text is familiar,
counted from what you actually marked rather than guessed.

Click a highlighted word for its translation, its dictionary form and how rare it is in everyday English, then decide: "I know this",
"+ Deck" to review it later, or "Ignore". Select several words and press Alt+L to capture a
whole phrase with the sentence it came from.

The cards you build are reviewed in a panel, beside your reading or in the sidebar, with
spaced repetition that picks the moment for you.

**The analysis is local.** The dictionary and the engine run in your browser: no page you read
is ever sent anywhere, and the extension works offline. With no account, it makes no network
request at all.

Create a Cymbra account if — and only if — you want your words and cards on your other
devices. That is the only thing that leaves your machine, and you can erase it from Settings
without deleting your account.

Cymbra Lingua is also on Firefox and on Safari (iPhone, iPad, Mac).

---

## Graphics

| Asset            | Requirement                                                             | Where it is                  |
| ---------------- | ----------------------------------------------------------------------- | ---------------------------- |
| Store icon       | 128×128                                                                 | `icons/icon-128.png`         |
| Screenshots      | 1280×800 or 640×400, JPEG or 24-bit PNG **without alpha**, at least one | captured by hand — see below |
| Small promo tile | 440×280, no alpha                                                       | optional                     |
| Marquee          | 1400×560, no alpha                                                      | optional                     |

Screenshots cannot be generated from the repository: they need the extension loaded in a
browser, reading a real page. Load `dist-chromium` unpacked, open an English article, click
**Analyser cette page**, and capture the highlighted page, the word popup, the review panel and
the stats. Any capture can then be conformed to 1280×800 without alpha.

## Extra fields

| Field          | Value                                                                                                                                                                                              |
| -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Homepage URL   | `https://cymbra.app/lingua/` — **only once the site is deployed**; the live site predates that page, and an unknown path there serves the home page rather than a 404, so it looks fine and is not |
| Support URL    | `https://cymbra.app/support/`                                                                                                                                                                      |
| Mature content | no                                                                                                                                                                                                 |

## Test instructions (for the reviewer)

No account is needed to review this extension; sign-in only turns on sync between a reader's
own devices.

1. Install the extension and open any English-language web page (a news article works well).
2. Click the toolbar icon, then **Analyser cette page** ("Analyse this page"). The extension is
   activeTab-first on Chrome: it reads a page only when asked, or after the optional
   "Toujours surligner" permission is granted.
3. Words you have not marked are highlighted, and the pill shows the share of the page you
   already know.
4. Click a highlighted word: a popup gives its translation, its dictionary form and how common
   it is, with three actions — "Je connais", "+ Deck", "Ignorer".
5. Press Alt+L on a multi-word selection to capture a phrase; Alt+Shift+S opens the review
   panel.

The interface is in French: the extension teaches English to French speakers.

## Single purpose (Chrome Web Store)

Cymbra Lingua has one purpose: helping a reader understand and learn English vocabulary on the
page they are already reading. Every feature serves it — highlighting unknown words, showing a
word's translation on click, capturing words and phrases into a deck, and reviewing that deck.

## Permission justifications (Chrome Web Store)

Each answer names the user-visible feature and the code path, because that is what a reviewer
checks against the bundle.

**`activeTab`** — The reader is injected only into the tab the user is looking at, when they
click "Analyser cette page" in the toolbar popup. This is the extension's default mode on
Chrome: without it, the extension cannot read the page it was asked to analyse.
`src/popup/popup.ts` (`activeTabId`, then the injection below).

**`scripting`** — To inject that reader. Two paths: `chrome.scripting.executeScript` on the
popup's explicit request (`src/popup/popup.ts:309`), and
`chrome.scripting.registerContentScripts` once — and only once — the user has granted the
optional `<all_urls>` permission, so highlighting survives a page reload
(`src/background.ts:454`). Revoking the permission unregisters it.

**`storage`** — The reader's own data, kept on their machine: which words they marked as known,
learning or ignored, the cards they captured, their review history and their preferences. No
page content is stored. `src/state/`, and IndexedDB for the parts that grow.

**`sidePanel`** — The review panel, opened by the user from the popup or the Alt+Shift+S
shortcut, so they can review cards beside the page instead of over it.
`src/sidepanel/`, opened at `src/popup/popup.ts:289`.

**`identity`** — Only for the optional account. `chrome.identity.launchWebAuthFlow` runs the
Google or Apple sign-in flow and returns to `chrome.identity.getRedirectURL()`
(`src/background.ts:251`). The extension never sees a password. A reader who does not sign in
never reaches this code.

**`<all_urls>` (optional, not requested at install)** — Offered behind "Toujours surligner" for
readers who want highlighting on every page without clicking the toolbar each time. It is
requested by `chrome.permissions.request` from a user gesture (`src/popup/popup.ts:424`) and
can be revoked at any time. The extension is fully usable without it.

## Remote code

None. Everything the extension runs ships inside the package, including the WebAssembly
analysis engine and the language pack. No script is fetched at runtime; the content security
policy is `script-src 'self' 'wasm-unsafe-eval'`.

## Data usage disclosures (Chrome Web Store)

Answer **yes** to collecting _"Personally identifiable information"_ — but only in the narrow
sense below — and **no** to every other category (health, financial, authentication
information, personal communications, location, web history, user activity, website content).

What is collected, and only for a signed-in reader: the email address used to create the
account, and the reader's own vocabulary state (word statuses, cards, review history) sent to
`https://api.cymbra.app` so their devices agree. Reading activity, page content and browsing
history are **not** collected — the analysis never leaves the browser.

Tick the three certifications: the data is not sold to third parties, it is not used or
transferred for a purpose unrelated to the item's single purpose, and it is not used or
transferred to determine creditworthiness or for lending.

## Source code (addons.mozilla.org)

AMO requires the human-readable source because the package is built (esbuild bundle, and a
`.wasm` compiled from Rust). `lingua-extension-release` attaches it automatically; its build
instructions are in [REVIEWERS.md](REVIEWERS.md), which becomes the archive's `README.md`.

License to declare on AMO: **Apache-2.0** (the repository's, see `LICENSE`).
