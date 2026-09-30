# lingua-data-packs — reduction rules shared across pairs

## ADDED Requirements

### Requirement: A pair's reduction rules include the rules it shares
The reduction rules of a pair SHALL be its own reducer together with every module of rules shared between pairs, and the record committed with a pair's tables SHALL name those files and carry one digest over all of them. A change to any of them SHALL fail the checks of every pair whose tables were reduced by the previous rules, until those tables are reduced again from their pinned sources; the version of a pack reduced again SHALL name that digest.

#### Scenario: A shared rule changes
- **WHEN** a pull request edits a rule module shared by the en-fr and es-fr reducers without reducing their tables again
- **THEN** the checks of both pairs fail, naming the rule files that changed

#### Scenario: Another pair's reducer changes
- **WHEN** a pull request edits only the es-fr reducer
- **THEN** the en-fr tables still pass their check

#### Scenario: Moving rules into a shared module
- **WHEN** rules are moved out of a pair's reducer into a shared module without changing what they do
- **THEN** the pair's tables reduced again from the same pinned sources are byte-identical, except for the pack version that names the new rule digest

### Requirement: A source is credited as its licence requires
The attribution notice of a pack SHALL credit each source in the form its licence makes a condition of use, including the author's name where the licence names how the author is to be credited.

#### Scenario: wordfreq
- **WHEN** a pack's frequencies come from wordfreq
- **THEN** its notice credits wordfreq to Robyn Speer, with the CC BY-SA 4.0 licence of its data
