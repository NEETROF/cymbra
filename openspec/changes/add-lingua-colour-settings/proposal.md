# add-lingua-colour-settings — the reader chooses the colours of the words and of the page

## Why

On a colour e-ink reader (Boox Go Color 7 Gen 2, Kaleido 3), Guillaume found the unknown-word
highlight almost invisible (2026-09-30). The fill is a pale coral at 30 % opacity and the
underline is the browser's default hairline. A colour e-ink filter washes pastels out and draws
colour at half the panel's resolution, so the fill disappears and a 1 px line is left. The
colours are fixed in the token sheet. The reader cannot fix this for their screen, and one
default cannot suit a laptop, a phone at night and a Kaleido panel all at once.

Dogfooding the presets on a Boox Go 10.3 Lumi (10.3", 1860×2480, 300 ppi, monochrome) on
2026-10-03 found the rest of the reading surface fighting the same screen:
- **Every surface is sized for a desktop.** 121 font sizes in px, none relative: the word card,
  the drawer, the pill, the reader's toolbar and library read tiny on a large, dense panel.
- **The text size reaches the book only.** At 180 % the book is comfortable, while the word card,
  the drawer and the toolbar stay at a third of its size.
- **The surfaces keep the dark identity whatever the reader chose.** With an e-ink preset on a
  paper page, the card and the drawer are small light text on navy, which an e-ink panel shows
  worst.

## What Changes

- A **Couleurs** block in Réglages, rendered by the one settings builder, so it appears in the
  popup, the side panel, the reader's drawer and the Safari app alike. It holds:
  - **Presets**: *Cymbra* (today's colours, the default), *E-ink contrasté* (grey fill, thick
    black underlines — for a monochrome panel) and *E-ink couleur* (saturated ink colour on the
    word itself and a thick underline — for a Kaleido panel).
  - **Free settings**, one set per highlighted status, *mots inconnus* and *mots en cours*:
    - the fill: a colour and an intensity (none, light, strong);
    - the underline: a colour, a style (solid, dotted, dashed, wavy, double, none) and a
      thickness (thin, thick);
    - the colour of the word's text: the page's own colour, or a chosen one.
  - **The reader's page**: background and text colours of the *paper* page and of the *dark*
    page. On paper, the text keeps the book's own colour unless one is chosen.
  - A **preview** paragraph with an unknown and a learning word, painted with the current
    choice, and a **reset** to the Cymbra preset.
  - Changing any free setting starts from the preset on screen and turns the choice into
    *Personnalisé*.
- **Everywhere, at once**: the highlight colours apply to every page the extension reads and to
  the book reader. The page colours apply to the book reader. A change is applied to open tabs
  and open books without a reload.
- Known words stay unmarked. The point of the highlight is what is not known yet.
- The two statuses stay distinguishable without colour in the default and in every preset. In
  *Personnalisé* the reader may choose otherwise; the settings warn when the two styles become
  identical.

- **The surfaces follow the reader's colours and page** (product owner's choice, 2026-10-03):
  with an e-ink preset, every surface of the extension is black on white, without grey text,
  tinted fills or shadows. With the Cymbra or custom colours, the surfaces follow the page theme:
  a light theme of the identity on paper, today's Midnight Navy on dark. That covers the word
  card, the drawer and its views, the pill, the reader's toolbar, panels and library, the popup,
  the side panel and the extension's pages.
- **One text size for the book and every surface** (product owner's choice, 2026-10-03): the
  reader's text size (80–200 %) scales the extension's surfaces as well as the book, on web pages
  as in the book reader. The size and the theme move from the *Livres* block to a block of their
  own, *Affichage*, next to *Couleurs*; the reader's *Aa* panel keeps them. At the largest size,
  a surface stays within the screen's width.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-browser-extension`:
  - *Cymbra visual identity* (as rewritten by `add-lingua-reader`) — the palette's amber and
    coral become the **default** highlight tints rather than the only ones, and the surfaces get a
    light and an e-ink theme besides the dark one.
  - A requirement is added: the reader chooses the highlight colours, which an e-ink preset carries
    to every surface.
  - A requirement is added: one text size for the book and every surface.
- `lingua-reader`: a requirement is added — the reader chooses the colours of the paper and dark
  pages.

Both deltas depend on `add-lingua-reader`, still open. This change archives after it
(`archiveAfter` in `.openspec.yaml`).

## Impact

- **Products**: Cymbra Lingua only — the browser extension (`apps/lingua-extension`) on
  Chromium, Firefox and Safari, and therefore the Safari app (`apps/lingua-apple`) that ships its
  bundle. ID, Music, Live, back office and site are untouched. Nothing is consumed from `id-*`
  or `platform-*`; the choice is a device preference like the reader's display, not an account
  setting.
- **Code**:
  - `src/state/` — a colour preference and its validation;
  - `src/styles/tokens.css` — the presets' colours, as tokens, and the surfaces' light and e-ink
    themes;
  - `src/reading/` — a colour sheet generated from the preference, adopted next to the token
    sheet, a Couleurs block and an Affichage block in the settings builder, and `surface-look.ts`,
    which themes and scales every surface root;
  - `src/reader/` — the page colours through the same tokens;
  - every stylesheet — font sizes and the main widths multiply the text size.
- **Visible to every reader**: the default display is the paper page, so with the default colours
  the surfaces turn light. The dark identity stays one tap away (*Sombre*).
- **Storage**: one new `chrome.storage.local` key. No IndexedDB schema change, no sync, no
  `.proto`, no backend, no new permission.
