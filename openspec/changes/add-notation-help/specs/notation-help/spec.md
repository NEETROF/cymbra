## ADDED Requirements

### Requirement: Contextual help for a staff symbol on demand

The app SHALL let the user request help for any symbol rendered on the staff by **long-pressing**
it, in both the scrolling player staff and the static score view. On a long-press that lands on a
rendered symbol, the app SHALL show a **dismissible help bubble** that explains what that symbol
is, positioned to point at the symbol in place. The help SHALL be available **at any time** while a
score is displayed and MUST NOT alter, pause, or block playback, nor consume the gesture used to
play.

Help SHALL be available for **every symbol the staff renderers draw** — the user's long-press on any
rendered glyph resolves to an explanation, with no rendered symbol left as a dead press. This
includes at least: a **note** (with its pitch name and register), a **rest**, an **accidental**
(sharp, flat, natural, double-sharp, double-flat), a **clef** (treble/G, bass/F, C), the **key
signature** (armature), the **time signature**, an **augmentation dot**, and the structural/relational
marks the renderers produce (**stems, flags, beams, ties/slurs, ledger lines, bar/measure lines,
braces, tuplet numbers**). If the renderers gain a new glyph, it SHALL ship with matching help.

#### Scenario: Long-press a note shows its help

- **WHEN** the user long-presses a note on the staff
- **THEN** a dismissible bubble explains that it is a note and names its pitch, pointing at that note

#### Scenario: Long-press an accidental explains it

- **WHEN** the user long-presses a sharp, flat, or natural
- **THEN** a dismissible bubble explains what that accidental does to the note

#### Scenario: Long-press a clef or key signature explains it

- **WHEN** the user long-presses the clef or the key signature at the head of the staff
- **THEN** a dismissible bubble explains the clef, or the key signature, respectively

#### Scenario: Help never blocks playback

- **WHEN** a help bubble is shown while a piece is loaded or playing
- **THEN** playback continues unaffected and the user can dismiss the bubble at any time

#### Scenario: Long-press a structural mark explains it

- **WHEN** the user long-presses a rendered structural mark (for example a beam, a tie/slur, a bar line, or a ledger line)
- **THEN** a dismissible bubble explains what that mark is

#### Scenario: Every rendered symbol has help

- **WHEN** any glyph the staff renderers draw is long-pressed
- **THEN** it resolves to an explanation, with no rendered symbol left without help

#### Scenario: Long-press on empty staff area does nothing intrusive

- **WHEN** the user long-presses where no symbol is rendered
- **THEN** no help bubble is shown and the score is unaffected

### Requirement: Symbol resolution from the rendered staff

The staff renderers SHALL publish, for the currently displayed frame, the on-screen region and a
**symbol descriptor** for each glyph they draw, so a long-press location can be resolved to the
symbol under the finger. The descriptor SHALL carry the symbol **kind** and the specifics needed to
select the correct help (for a note: its pitch name and register; for an accidental: which one; for a
clef: which clef). Resolving a symbol MUST NOT change what the renderers draw.

#### Scenario: A press maps to the symbol under it

- **WHEN** a long-press falls within a rendered symbol's region
- **THEN** the app resolves it to that symbol's descriptor and shows the matching help

#### Scenario: Nearest symbol when regions are dense

- **WHEN** a long-press falls between two adjacent symbols' regions
- **THEN** the app resolves to the closest symbol rather than showing nothing

#### Scenario: Rendering is unchanged by the query seam

- **WHEN** the staff is drawn with symbol resolution available
- **THEN** the rendered output is identical to drawing it without symbol resolution

### Requirement: Discovery of the help gesture

Because the long-press gesture is not self-evident, the app SHALL inform the user, **once**, that
staff symbols can be long-pressed for help. This hint SHALL be delivered through the shared
`feature-discovery` coaching mechanism, SHALL be **dismissible**, SHALL have its "seen" state
**persisted** so it is not shown again, and MUST NOT block interaction with the score.

#### Scenario: First-time hint that help exists

- **WHEN** the user views a score for the first time after this feature ships
- **THEN** a dismissible hint tells them they can long-press symbols for help

#### Scenario: Hint shown only once

- **WHEN** the user has already seen and dismissed the help-gesture hint
- **THEN** it is not shown again on later scores

### Requirement: Browsable notation glossary in help/tips

The app SHALL expose the same symbol explanations as a **browsable glossary** reachable from the
`feature-discovery` help/tips surface, so a user can look up a symbol away from a score and re-read
an explanation that was previously shown as a one-time bubble. The glossary SHALL cover the same
symbol kinds as the on-staff help.

#### Scenario: Look up a symbol from help/tips

- **WHEN** the user opens the notation glossary from the help/tips surface
- **THEN** they can browse and read the explanations for the staff symbols

#### Scenario: Glossary and on-staff help stay consistent

- **WHEN** an explanation exists for a symbol kind
- **THEN** the same explanation is available both as an on-staff bubble and in the glossary

### Requirement: Notation help copy is localized and accessible

All notation help copy — bubbles, the discovery hint, and the glossary — SHALL be authored through
the app's localization system and available in every supported language, and SHALL be accessible
(dismissible without relying on a single gesture, adequate contrast, screen-reader friendly),
consistent with the app's responsive/landscape layout.

#### Scenario: Help follows the app language

- **WHEN** the app is used in a supported language
- **THEN** the notation help bubbles, hint, and glossary appear in that language

#### Scenario: Bubble is dismissible accessibly

- **WHEN** a user relies on assistive input
- **THEN** they can dismiss a help bubble without depending on a single specific gesture

#### Scenario: Bubble stays on-screen in any orientation

- **WHEN** a help bubble is shown in portrait or landscape
- **THEN** it is positioned to remain fully visible and to point at its symbol
