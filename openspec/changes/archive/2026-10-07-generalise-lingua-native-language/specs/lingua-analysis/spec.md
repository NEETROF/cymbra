## ADDED Requirements

### Requirement: An engine serves one native language
An engine SHALL serve one native language, the one its first pack is glossed in, and SHALL refuse, with an explicit error naming both languages, a pack glossed in another, leaving the packs it holds as they were. It SHALL report that native language and, apart from it, the native language of the reader's profile. Setting the reader's profile SHALL be refused, with an explicit error and the profile unchanged, when no pack the engine holds is glossed in the profile's native language. A new engine's profile SHALL be its native language, studying its first pack's language alone.

#### Scenario: Every engine today
- **WHEN** an engine is built from the en-fr pack and the es-fr pack is added
- **THEN** it holds both, and reports French as its native language and as the reader's

#### Scenario: A pack of another native language
- **WHEN** a pack studying Spanish glossed in English is added to an engine built from the en-fr pack
- **THEN** it is refused with an error naming French and English, and the engine still holds the en-fr pack alone

#### Scenario: A native language no pack serves
- **WHEN** the reader's profile is set to Spanish studied with English native, on an engine holding the en-fr and es-fr packs
- **THEN** it is refused, and the reader's profile is unchanged

#### Scenario: English and Spanish output do not move
- **WHEN** the English and Spanish invariance baselines run after this change
- **THEN** every probe is byte for byte the one recorded before
