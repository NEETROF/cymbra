## MODIFIED Requirements

### Requirement: Cymbra visual identity
The UI surfaces the extension owns (icon popup, extension pages) SHALL apply the Cymbra
visual identity — the "Sonic Luminescence" palette from
`apps/music/lib/theme/cymbra_theme.dart`, already mirrored into CSS variables by the back
office (`apps/back-office/src/styles.css`) — through a single token sheet embedded in the
extension. Surfaces injected into third-party pages (the word popup) SHALL consume the same
tokens, with legibility on both light and dark pages taking precedence over fidelity to the
dark theme. The default highlight tints SHALL derive from the palette's amber (`handLeft` —
"learning") and coral (`error` — "unknown"); the reader MAY replace them (see "The reader
chooses the highlight colours"). The two highlight statuses SHALL also be distinguishable
without colour in the default and in every preset, by two different underline styles, so that
a monochrome screen tells them apart. No color SHALL be hard-coded outside the token sheet: the
presets' colours are tokens, and a colour the reader picks is their data, not the code's.

#### Scenario: Extension surfaces stay consistent
- **WHEN** the user opens the icon popup and then an extension page
- **THEN** both surfaces render with the Cymbra tokens (Midnight Navy backgrounds, violet primary, the identity's radii) and no hex outside the token sheet exists in the styles (checked by lint)

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
Known words SHALL stay unmarked.

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
