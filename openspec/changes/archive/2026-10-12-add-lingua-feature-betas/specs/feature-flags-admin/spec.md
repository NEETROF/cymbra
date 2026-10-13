## ADDED Requirements

### Requirement: A flag's beta options are the open campaigns of its app
The flags panel SHALL offer, as `beta:<campaign>` options for a flag, only the open campaigns of the flag's app, and every open campaign for a flag declared for every app.
Each option SHALL show the campaign's product. A stored scope naming a campaign of another product
SHALL be presented distinctly from a valid one, naming the cause.

#### Scenario: Scoping a Lingua flag
- **WHEN** an admin opens the rollout selector of `lingua.online_translation.enabled`
- **THEN** the beta options are the open Lingua campaigns, and no Music campaign is offered

#### Scenario: Scoping a Music flag
- **WHEN** an admin opens the rollout selector of `drums.enabled`
- **THEN** the beta options are the open Music campaigns
