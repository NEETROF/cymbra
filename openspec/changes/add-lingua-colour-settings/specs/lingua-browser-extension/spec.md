## MODIFIED Requirements

### Requirement: Cymbra visual identity
The UI surfaces the extension owns (icon popup, extension pages) SHALL apply the Cymbra
visual identity — the "Sonic Luminescence" palette from
`apps/music/lib/theme/cymbra_theme.dart`, already mirrored into CSS variables by the back
office (`apps/back-office/src/styles.css`) — through a single token sheet embedded in the
extension. The identity's dark theme SHALL apply to every surface when the reader's page theme is
dark, and a light theme of the same identity (the reader's paper and ink, the identity's violet)
when it is paper; with an e-ink preset, every surface SHALL be black on white, without grey text,
tinted fills or shadows. Surfaces injected into third-party pages (the word popup, the drawer, the
pill) SHALL consume the same tokens, with legibility on both light and dark pages taking
precedence over fidelity to the dark theme. The default highlight tints SHALL derive from the palette's amber (`handLeft` —
"learning") and coral (`error` — "unknown"); the reader MAY replace them (see "The reader
chooses the highlight colours"). The two highlight statuses SHALL also be distinguishable
without colour in the default and in every preset, by two different underline styles, so that
a monochrome screen tells them apart. No color SHALL be hard-coded outside the token sheet: the
presets' colours are tokens, and a colour the reader picks is their data, not the code's.

#### Scenario: Extension surfaces stay consistent
- **WHEN** the user opens the icon popup and then an extension page
- **THEN** both surfaces render with the same theme of the Cymbra tokens (Midnight Navy on a dark page theme, paper and ink on a paper one, the identity's violet primary and radii) and no hex outside the token sheet exists in the styles (checked by lint)

#### Scenario: The surfaces follow the page theme
- **WHEN** the reader, with the Cymbra colours, switches the page theme from paper to dark
- **THEN** the word card, the drawer, the pill, the reader's toolbar and library, the popup and the side panel turn from the light theme to the dark one, without a reload

#### Scenario: Highlighting legible on a light page
- **WHEN** a light-background page is highlighted with the default colours
- **THEN** the amber and coral highlight tints leave the page text in its original color and stay distinguishable from each other

#### Scenario: Highlighting legible without colour
- **WHEN** a highlighted page is rendered in greyscale, with the default colours or any preset
- **THEN** a learning word and an unknown word still differ, by their underline style

## ADDED Requirements

### Requirement: The reader chooses the highlight colours
The extension SHALL let the reader choose how unknown words and learning words are marked,
from presets or setting by setting, in its settings. The presets SHALL include the Cymbra
default and two presets for e-ink screens, one for monochrome panels and one for colour panels.
For each of the two statuses, the reader SHALL be able to set the fill (a colour and an
intensity, including none), the underline (a colour, a style including none, and a thickness)
and the colour of the word's text (the page's own, or a chosen one). A setting that the browser
cannot paint in a highlight SHALL NOT be offered on that browser. Changing a setting SHALL start
from the preset on screen. The settings SHALL show a preview painted with the current choice,
SHALL warn when the two statuses become indistinguishable without colour, and SHALL offer a
return to the default. The choice SHALL be kept on the device, and SHALL apply to every page the
extension reads and to the book reader, including pages and books already open, without a reload.
Known words SHALL stay unmarked. With an e-ink preset, every surface of the extension SHALL be
black on white as well.

#### Scenario: An e-ink preset reaches the surfaces
- **WHEN** the reader picks the monochrome e-ink preset, on a paper or a dark page
- **THEN** the word card, the drawer, the pill and the reader's toolbar and library turn black on white at once

#### Scenario: Choosing a preset for an e-ink screen
- **WHEN** the reader picks the colour e-ink preset
- **THEN** the unknown and learning words of the open pages and books are repainted with it, and the preview shows it

#### Scenario: Setting one status by hand
- **WHEN** the reader, on the Cymbra preset, thickens the underline of unknown words
- **THEN** unknown words get a thick underline, learning words keep the Cymbra style, and the settings show the choice as customised

#### Scenario: Two statuses made identical
- **WHEN** the reader gives learning words the same underline style as unknown words
- **THEN** the settings warn that the two will look alike on a monochrome screen, and apply the choice

#### Scenario: Back to the default
- **WHEN** the reader resets the colours
- **THEN** the Cymbra highlight and page colours are restored everywhere

#### Scenario: A browser that cannot thicken a highlight's underline
- **WHEN** the settings open on a browser whose highlights ignore the underline's thickness
- **THEN** no thickness setting is offered, and the other settings work

### Requirement: One text size for the book and every surface
The reader's text size SHALL scale the text of the books and every surface of the extension: the word card, the drawer and its views, the pill, the reader's toolbar, panels and library, the icon popup, the side panel and the extension's pages, on web pages as in the book reader. It SHALL be set in one place of the settings, next to the theme, and from the book reader's own panel. A change SHALL apply to the open pages, books and surfaces without a reload. At the largest size, a surface SHALL stay within the screen's width.

#### Scenario: A larger text, everywhere
- **WHEN** the reader sets the text size to 180 %
- **THEN** the book's text, the word card, the drawer and the reader's toolbar are drawn at 180 % of their default size

#### Scenario: Set once
- **WHEN** the reader changes the text size in the book reader's panel
- **THEN** the settings show the same size, and the surfaces open in other tabs follow it

#### Scenario: A small screen
- **WHEN** the text size is 200 % on a phone
- **THEN** the word card and the drawer still fit the screen's width
