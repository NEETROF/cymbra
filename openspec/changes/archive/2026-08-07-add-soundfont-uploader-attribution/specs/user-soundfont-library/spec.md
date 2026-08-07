## MODIFIED Requirements

### Requirement: Opt-in proposal of a private font to the public catalog

A user SHALL be able to explicitly propose one of their private fonts to the public
catalog. Proposing SHALL require a **licence declaration** and an explicit
**right-to-distribute attestation** captured at proposal time; the general CGU authorship
attestation SHALL NOT by itself satisfy this, because a soundfont is third-party sample
data. A proposed font SHALL enter the public catalog as `pending` (subject to the
soundfont-moderation lifecycle), recorded with the proposer as `uploaded_by`. A proposal
whose content is byte-identical to a non-`rejected` catalog font MUST be refused as a
duplicate. When the proposer's content matches a **`rejected`** catalog font, the proposal
is a **re-proposal**: it MUST carry a non-empty **justification** (an explanation
motivating reconsideration) and reopens that font per the soundfont-moderation "Re-proposing
a rejected soundfont reopens its row and requires a justification" requirement; a first
proposal (no prior rejected entry) requires no justification.

#### Scenario: Proposal requires licence and attestation

- **WHEN** a user proposes a private font without a licence declaration and right-to-distribute attestation
- **THEN** the proposal is refused and the font does not enter the public catalog

#### Scenario: Valid proposal enters the catalog as pending

- **WHEN** a user proposes a private font with a licence declaration and attestation
- **THEN** a public catalog entry is created `pending`, attributed to the proposer, awaiting review

#### Scenario: Proposal of already-cataloged content is refused

- **WHEN** a user proposes a font whose bytes match a non-`rejected` catalog entry
- **THEN** the proposal is refused as a duplicate and identifies the existing font

#### Scenario: Re-proposal of a rejected font requires a justification

- **WHEN** a user re-proposes a font whose catalog entry is `rejected`, with a justification
- **THEN** that entry reopens to `pending`, re-attributed to the proposer, and no second entry is created

#### Scenario: Re-proposal without a justification is refused

- **WHEN** a user re-proposes a `rejected` font without a justification
- **THEN** the proposal is refused and the entry stays `rejected`
