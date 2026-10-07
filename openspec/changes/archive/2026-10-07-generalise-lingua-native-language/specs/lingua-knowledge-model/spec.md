## MODIFIED Requirements

### Requirement: L1/L2 profile
The user profile SHALL keep `native_language` (the language of comfort: glosses, future translations) distinct from the studied languages, and SHALL never name the native language among the studied ones. A native language SHALL be French, English or Spanish, named by its ISO 639-1 tag. Glosses and packs SHALL be keyed by pair, named `<studied>-<native>` (`en-fr`), whereas knowledge state SHALL be keyed by studied language alone (*Knowledge keyed by language-lemma pair*), so it does not depend on the native language. Adding a pair between languages the core knows SHALL NOT require a code change.

#### Scenario: Glosses in the native language
- **WHEN** a user whose native language is `fr` opens the popup for an English word
- **THEN** the gloss shown comes from the en-fr pack

#### Scenario: The native language is never studied
- **WHEN** a reader whose native language is English is given English among their studied languages
- **THEN** the choice is refused and the profile is unchanged

#### Scenario: Knowledge does not follow the native language
- **WHEN** a reader's native language changes from French to English
- **THEN** their statuses, declared levels and cards in Spanish are kept as they were
