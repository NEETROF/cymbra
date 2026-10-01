# add-lingua-colour-settings — the reader chooses the colours of the words and of the page

## Why

On a colour e-ink reader (Boox Go Color 7 Gen 2, Kaleido 3), Guillaume found the unknown-word
highlight almost invisible (2026-09-30). The fill is a pale coral at 30 % opacity and the
underline is the browser's default hairline. A colour e-ink filter washes pastels out and draws
colour at half the panel's resolution, so the fill disappears and a 1 px line is left. The
colours are fixed in the token sheet. The reader cannot fix this for their screen, and one
default cannot suit a laptop, a phone at night and a Kaleido panel all at once.

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

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-browser-extension`:
  - *Cymbra visual identity* (as rewritten by `add-lingua-reader`) — the palette's amber and
    coral become the **default** highlight tints rather than the only ones.
  - A requirement is added: the reader chooses the highlight colours.
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
  - `src/styles/tokens.css` — the presets' colours, as tokens;
  - `src/reading/` — a colour sheet generated from the preference, adopted next to the token
    sheet, and a Couleurs block in the settings builder;
  - `src/reader/` — the page colours through the same tokens.
- **Storage**: one new `chrome.storage.local` key. No IndexedDB schema change, no sync, no
  `.proto`, no backend, no new permission.
