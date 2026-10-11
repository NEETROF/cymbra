## ADDED Requirements

### Requirement: A campaign belongs to one product
Every beta campaign SHALL belong to exactly one product, chosen when it is created and never changed, and its memberships SHALL count only in that product's plan snapshot.
A campaign created without a product SHALL be a Music campaign, so every campaign that existed
before this requirement is one. A campaign key SHALL stay unique across products. A Lingua campaign
SHALL be of kind `feature`; a `premium_trial` campaign SHALL be refused for Lingua. Rules that hold
"per account" — at most one premium trial at a time — SHALL be checked within the campaign's
product.

#### Scenario: A Lingua feature campaign
- **WHEN** an admin of Lingua creates the feature campaign `online-translation` for Lingua and enrols an account
- **THEN** the account's Lingua snapshot lists `online-translation`, and its Music snapshot does not

#### Scenario: A trial for Lingua
- **WHEN** an admin tries to create a `premium_trial` campaign for Lingua
- **THEN** creation is refused and nothing is stored

#### Scenario: Existing campaigns
- **WHEN** the change is deployed
- **THEN** every existing campaign is a Music campaign and every Music snapshot is unchanged
