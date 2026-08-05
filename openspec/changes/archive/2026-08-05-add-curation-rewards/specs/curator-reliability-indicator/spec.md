## ADDED Requirements

### Requirement: Per-user curator reliability indicator in the back office

The back office SHALL present, to `moderator`/`admin` identities, a **read-only** per-user
reliability indicator to inform manual promotion decisions. It SHALL show at least: the
user's **total number of ratings**, their **coverage contribution** (how much they rated
under-covered scores), and their **alignment rate** — the share of the user's **settled**
ratings that matched the ground truth. The indicator SHALL NOT trigger any automatic role
change; promotion remains a manual admin action. A non-`moderator`/non-`admin` caller MUST
NOT be able to read it.

#### Scenario: Moderator views a user's reliability

- **WHEN** a moderator/admin opens a user's reliability indicator in the back office
- **THEN** they see the user's rating count, coverage contribution, and alignment rate

#### Scenario: Alignment rate reflects settled ratings only

- **WHEN** a user has ratings that are not yet settled
- **THEN** those unsettled ratings are excluded from the alignment rate

#### Scenario: Indicator does not promote anyone

- **WHEN** a user's alignment rate is high
- **THEN** no role is granted automatically; an admin must grant it manually

#### Scenario: Unauthorized caller cannot read it

- **WHEN** a caller without `moderator`/`admin` requests a user's reliability indicator
- **THEN** the request is refused
