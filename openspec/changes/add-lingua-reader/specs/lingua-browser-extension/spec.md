## MODIFIED Requirements

### Requirement: Cymbra visual identity
The UI surfaces the extension owns (icon popup, extension pages) SHALL apply the Cymbra
visual identity — the "Sonic Luminescence" palette from
`apps/music/lib/theme/cymbra_theme.dart`, already mirrored into CSS variables by the back
office (`apps/back-office/src/styles.css`) — through a single token sheet embedded in the
extension. Surfaces injected into third-party pages (the word popup) SHALL consume the same
tokens, with legibility on both light and dark pages taking precedence over fidelity to the
dark theme. Highlight tints SHALL derive from the palette's amber (`handLeft` —
"learning") and coral (`error` — "unknown"). The two highlight statuses SHALL also be
distinguishable without colour, by two different underline styles, so that a monochrome
screen tells them apart. No color SHALL be hard-coded outside the token sheet.

#### Scenario: Extension surfaces stay consistent
- **WHEN** the user opens the icon popup and then an extension page
- **THEN** both surfaces render with the Cymbra tokens (Midnight Navy backgrounds, violet primary, the identity's radii) and no hex outside the token sheet exists in the styles (checked by lint)

#### Scenario: Highlighting legible on a light page
- **WHEN** a light-background page is highlighted
- **THEN** the amber and coral highlight tints leave the page text in its original color and stay distinguishable from each other

#### Scenario: Highlighting legible without colour
- **WHEN** a highlighted page is rendered in greyscale
- **THEN** a learning word and an unknown word still differ, by their underline style

### Requirement: The reader never fights the platform's text selection
The reader SHALL NOT clear, suppress or pre-empt the host platform's native text selection while it is being made, the one exception being Safari's finished phrase below.
It SHALL NOT cancel `selectstart`, SHALL NOT suppress the platform's selection callout or
context menu, and SHALL NOT remove the document's ranges on its own. Where the platform's
selection gesture and a reader gesture are the same physical gesture, the platform's selection
SHALL win, and the reader SHALL derive its behaviour from the resulting selection. A
platform-drawn selection menu appearing alongside the panel is accepted; the panel SHALL be
positioned so it does not sit under that menu.

On Safari, once the user's finger lifts from a selection of several words, the reader SHALL
remove that selection after capturing it, so the platform's callout does not cover the
expression card; the card keeps what was captured. A single-word selection SHALL be kept, with
its handles, so it can still be extended into a phrase. A mouse lift SHALL keep the selection.
Safari reports the lift after a handle drag, which is what makes the end of the gesture known;
Firefox for Android does not, and there the selection SHALL stay as the platform left it.

#### Scenario: Press-and-hold on a phone
- **WHEN** the reader presses and holds a word on a touch device
- **THEN** the platform selects that word and its own menu appears, and the panel opens for that word with the actions matching its status

#### Scenario: Reclassifying a non-highlighted word on a phone
- **WHEN** the reader presses and holds a word marked "known" or "ignored"
- **THEN** the popup opens offering to reclassify it, reached through the platform selection rather than a competing long-press gesture

#### Scenario: A phrase finished on Safari
- **WHEN** the user drags a selection handle over several words on Safari and lifts the finger
- **THEN** the expression card opens for those words, and the page's selection is removed, taking the platform's callout with it

#### Scenario: A word selected on Safari stays extensible
- **WHEN** the user presses and holds a word on Safari and lifts the finger
- **THEN** the word stays selected with its handles, and dragging a handle extends it

#### Scenario: Firefox for Android keeps the selection
- **WHEN** the user selects several words on Firefox for Android
- **THEN** the selection and the platform's menu stay in place
