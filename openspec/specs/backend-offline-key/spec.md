# backend-offline-key Specification

## Purpose
TBD - created by archiving change add-premium-subscription. Update Purpose after archive.
## Requirements
### Requirement: The offline cache secret rotates once when the plan lapses

The backend SHALL rotate a user's offline cache secret **once** when their effective plan drops
to `free` past the grace period (trial end, lapse, comp expiry, admin revocation), so cached
catalog scores on every device become unreadable at the next key derivation. Rotation MUST NOT
happen while a row is in grace or billing retry, MUST be idempotent for one lapse (sweep job and
on-demand evaluation may both observe it), and MUST NOT happen when a feature beta closes.
Free users keep a valid secret (own-upload cache).

#### Scenario: Rotation on trial end

- **WHEN** a trial ends with no other active row
- **THEN** the next secret request returns a new value, exactly once for that lapse

#### Scenario: No rotation in grace

- **WHEN** a subscription is in billing retry within the grace period
- **THEN** the secret is unchanged

#### Scenario: Idempotent lapse

- **WHEN** the lapse is observed by the daily sweep and by two device reconnects
- **THEN** only one rotation happens

