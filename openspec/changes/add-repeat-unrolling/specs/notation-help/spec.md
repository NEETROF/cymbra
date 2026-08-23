# notation-help — delta for add-repeat-unrolling

## ADDED Requirements

### Requirement: Help for repeat notation symbols

The long-press notation help SHALL resolve and explain the repeat symbols the
renderers engrave: repeat barlines (start and end of a repeated section,
including what "play it again" means), volta brackets (which pass plays which
ending), the measure-repeat `%` sign (play the previous measure again), and
the segno, coda and D.C./D.S./Fine markers (where the music jumps). Both the
scrolling staff and the Partition painter SHALL record these symbols' regions
in the hit index, the help glossary SHALL gain one entry per symbol family,
and the copy SHALL land in **all** app locales in the same change (per the
no-translation-drift rule), plain-language and accessible like the existing
help entries.

#### Scenario: Long-press on a repeat barline

- **WHEN** the player long-presses the dotted repeat barline at the end of a
  repeated section
- **THEN** the help explains that the section between the repeat signs is
  played again

#### Scenario: Long-press on a volta bracket

- **WHEN** the player long-presses the "2." volta bracket
- **THEN** the help explains that this measure is played on the second pass,
  instead of the first ending

#### Scenario: Long-press on a measure-repeat sign

- **WHEN** the player long-presses a `%` sign
- **THEN** the help explains that the previous measure is played again

#### Scenario: Repeat help is present in every locale

- **WHEN** the app runs in any supported locale
- **THEN** every repeat-symbol help entry and glossary entry is translated (no
  English fallback)
