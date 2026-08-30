# score-upload — delta (add-private-score-catalog)

## MODIFIED Requirements

### Requirement: Mandatory rights attestation before upload

Before a validated file can be submitted, the user MUST declare the **basis** on
which they may contribute it — that they are its **author**, that it is in the
**public domain** (or under a free licence permitting its use), or that they
import it **for their strictly personal use** (`private_use`) — via a distinct
choice, AND MUST tick a **single confirmation checkbox** that attests the
declaration is accurate (e.g. "Je certifie que cette déclaration est exacte et que
je dispose des droits nécessaires pour mettre cette partition à disposition"),
localised per `app-localization`. The `private_use` option MUST make clear, in
its localised label or helper text, that such a score remains private forever
and can never be shared or proposed to the public catalog. The confirmation
MUST NOT restate the basis options (the basis choice already captures which
applies). The submit action MUST remain disabled until a basis is selected and
the checkbox is ticked, and both the declared basis and the confirmation MUST be
sent with the upload.

#### Scenario: Submit blocked without the attestation

- **WHEN** no rights basis is selected or the confirmation checkbox is not ticked
- **THEN** the upload cannot be submitted

#### Scenario: Attestation enables and accompanies submit

- **WHEN** the user selects a rights basis and ticks the confirmation checkbox
- **THEN** submit becomes available and both the declared basis and the
  confirmation are included in the upload request

#### Scenario: Personal-use basis is offered and explained

- **WHEN** the attestation step is shown
- **THEN** a third "strictly personal use" basis is available alongside author
  and public domain, with localised copy stating the score can never be shared
  or proposed to the public catalog

## ADDED Requirements

### Requirement: No public-catalog affordance for personal-use scores

The app SHALL omit, in any owner-facing listing of the user's own scores, every
affordance that initiates sharing or a proposal to the public catalog for a
score whose rights basis is `private_use`. Scores with the author or
public-domain basis SHALL keep the affordances they have today.

#### Scenario: Personal-use score shows no propose affordance

- **WHEN** the owner views a score imported with the `private_use` basis
- **THEN** no share/propose-to-catalog affordance is shown for that score

#### Scenario: Other bases unaffected

- **WHEN** the owner views a score with the author or public-domain basis
- **THEN** its existing affordances are unchanged
