## ADDED Requirements

### Requirement: Store-listing URL fields

The repository SHALL contain the `cymbra.app` URLs declared in each store listing, per platform and locale, so the console values are reviewable and reproducible rather than remembered.

The values, and what each field answers:

| Field | Value |
|---|---|
| Marketing URL (ASC) / Website (Play) | `https://cymbra.app/music/` for `fr`; `https://cymbra.app/en/music/` for `en`, `it`, `es` |
| Support URL | `https://cymbra.app/support/` for `fr`; `https://cymbra.app/en/support/` for `en`, `it`, `es` |
| Privacy policy URL | `https://cymbra.app/confidentialite/` for `fr`; `https://cymbra.app/en/privacy/` for `en`, `it`, `es` |
| Data deletion URL (Play) | `https://cymbra.app/en/delete-account/` |

These are console-side values. Changing one is a listing edit, not a build.

#### Scenario: The marketing URL points at the product page

- **WHEN** a user follows the Website or Marketing URL from the Music listing
- **THEN** they SHALL land on the Cymbra Music product page in the listing's locale, not on the multi-product hub at the site root

#### Scenario: The recorded values match the consoles

- **WHEN** an operator compares the checked-in listing URLs against App Store Connect and the Play Console
- **THEN** every field SHALL match for every locale the listing is published in

### Requirement: The support URL is a support page

The support URL SHALL resolve to a page whose content is user assistance. It SHALL NOT be the site root, the marketing page, or any page whose purpose is to present the product.

App Review enforces this: the macOS 1.32.0 submission was rejected under guideline 1.5 for declaring the home page as the support URL. The requirement exists so that a site restructure cannot quietly reintroduce the rejection by making the marketing page the most convenient thing to point at.

#### Scenario: Support and marketing are distinct

- **WHEN** the listing's support URL and marketing URL are compared
- **THEN** they SHALL resolve to different pages, and the support URL SHALL be `/support/` or `/en/support/`

#### Scenario: A restructure retargets the support URL

- **WHEN** a change proposes pointing the support URL at a landing or hub page
- **THEN** the change SHALL be rejected, on the rejection precedent recorded above
