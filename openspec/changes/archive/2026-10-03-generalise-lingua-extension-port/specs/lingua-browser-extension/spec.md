## ADDED Requirements

### Requirement: Every language-bound call names its studied language
The extension SHALL name the studied language on every request to the engine whose answer depends on it — page analysis, glosses, phrase glosses, word grammar, calibration, statuses, declared level, levels and ladder, vocabulary estimate, exposures and promotion, deck capture, attributions — on every variant and through every transport (the engine in the page, or the engine in the background reached by messaging). Requests about the whole reader (backup, restore, resets, counts, the review session, sync exports and applies) SHALL name none. Until the reader can choose a studied language, the extension SHALL ask in English, and what it shows SHALL be unchanged.

#### Scenario: A page analysis names its language
- **WHEN** a reading session analyses a page
- **THEN** the request names English, and the analysis shown is the one shown before this change

#### Scenario: The language crosses the messaging transport
- **WHEN** a surface on Firefox or Safari, or on a Chromium page whose policy blocks the in-page engine, asks for a gloss
- **THEN** the request that reaches the background engine carries the language, and the background engine answers in it

#### Scenario: Whole-reader requests name no language
- **WHEN** the extension takes a backup or starts a review session
- **THEN** the request carries no language, and covers every language the reader's state holds

#### Scenario: Naming English changes nothing
- **WHEN** the engine is asked every probe of the English invariance baseline with the language named `en`
- **THEN** each answer is byte-for-byte the one given when no language is named
