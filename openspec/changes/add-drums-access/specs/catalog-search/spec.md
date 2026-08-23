## MODIFIED Requirements

### Requirement: Piano-only filter parameter

The search operation SHALL accept an optional **instrument** filter naming an
instrument family (`keyboard` or `percussion`); when set, only scores recorded with
that instrument SHALL be returned. When unset, scores are not constrained by
instrument. This parameter is independent of the caller (the app may always supply
it) and composes conjunctively with the other filters. It replaces the former
boolean piano filter, which rested on a staff-count proxy rather than on the
score's actual instrument.

Independently of the filter the caller supplies, results SHALL be constrained by
the caller's drum eligibility (see `music-drums-visibility`): a caller without the
drum feature never receives a percussion score, whether they filtered for one,
filtered for keyboard, or filtered for nothing at all. The filter narrows; it never
widens.

#### Scenario: Instrument filter narrows to that family

- **WHEN** an eligible caller searches with the instrument filter set to keyboard
- **THEN** only scores recorded as keyboard are returned

#### Scenario: No instrument filter is not constrained by instrument

- **WHEN** an eligible caller searches without the instrument filter
- **THEN** results are not constrained by instrument

#### Scenario: Filtering for percussion without the feature returns nothing

- **WHEN** an ineligible caller searches with the instrument filter set to
  percussion
- **THEN** no scores are returned, rather than an error revealing the constraint

#### Scenario: An unfiltered search still withholds percussion

- **WHEN** an ineligible caller searches without any instrument filter
- **THEN** percussion scores are absent from the results
