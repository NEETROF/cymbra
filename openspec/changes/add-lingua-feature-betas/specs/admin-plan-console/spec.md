## ADDED Requirements

### Requirement: Campaigns show and choose their product
The campaigns page SHALL show each campaign's product and SHALL ask for a product when a campaign is created, offering only the products the admin administers.
Choosing Lingua SHALL offer only the `feature` kind. A user's page SHALL let an admin enrol the
account in an open campaign of a product they administer, SHALL show each membership with its
campaign's product, and SHALL let them revoke a membership of a product they administer. The page
SHALL stay reachable to an admin of Music or of Lingua.

#### Scenario: Creating a Lingua campaign
- **WHEN** an admin of Lingua creates a campaign and picks Lingua
- **THEN** only the `feature` kind is offered, and the created campaign is listed with the product Lingua

#### Scenario: Enrolling a reader who asked on Discord
- **WHEN** an admin of Lingua opens a reader's page and enrols them in `online-translation`
- **THEN** the membership is listed with the product Lingua, and the reader's Lingua flags include it at their next fetch

#### Scenario: An admin of Music only
- **WHEN** an admin who administers Music but not Lingua opens the enrol dialog
- **THEN** no Lingua campaign is offered
