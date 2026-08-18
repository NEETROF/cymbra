## MODIFIED Requirements

### Requirement: Enforce a per-user rolling upload quota

The backend SHALL limit how many scores a user may contribute within a rolling
time window: an upload MUST be rejected once the caller already has the configured
maximum number of contributed scores created within the configured window. The
maximum and the window length in days SHALL be **resolved per request from runtime
configuration keyed by the caller's effective plan**
(`plans.scores.upload_quota.free` = today's default 5 / 7 days,
`plans.scores.upload_quota.premium` for a plan whose unlock set includes
`scores.extended_quotas`), so free keeps its current value and premium raises it
without a release. Uploading and proposing to the catalog SHALL remain available to
every plan. The quota MUST be enforced server-side before any storage or database
write, scoped to the caller's own contributions; the typed refusal SHALL tell the
app that a higher plan raises the limit so the surface can upsell.

#### Scenario: Upload allowed under the quota

- **WHEN** the caller has fewer than the configured maximum of contributed scores
  within the current window
- **THEN** the upload is allowed to proceed to validation and storage

#### Scenario: Upload rejected at the quota

- **WHEN** the caller has already reached the configured maximum of contributed
  scores created within the current window
- **THEN** the upload is rejected with a typed quota error and nothing is stored

#### Scenario: Quota window is rolling

- **WHEN** a caller's earlier uploads fall outside the configured window
- **THEN** those uploads no longer count against the quota and new uploads are
  allowed up to the maximum again

#### Scenario: Premium quota applies to a plan holder

- **WHEN** a caller whose plan grants `scores.extended_quotas` uploads beyond the free
  maximum but within the premium maximum for the window
- **THEN** the upload is allowed

#### Scenario: Free plan keeps uploading and proposing

- **WHEN** a free user within quota uploads a score and proposes it to the catalog
- **THEN** both succeed exactly as before

## ADDED Requirements

### Requirement: Per-plan private score library cap

The backend SHALL bound the number of scores a user keeps in their private library by a
maximum resolved per request from runtime configuration keyed by the caller's effective plan
(`plans.scores.library_max.free`, `.premium`), independent of the rolling upload quota. An
upload that would exceed the cap MUST be refused with a typed error before any storage write;
deleting a score frees a slot. Scores already accepted into the public catalog SHALL NOT count
against the cap.

#### Scenario: Cap reached on the free plan

- **WHEN** a free user at the library cap uploads another score
- **THEN** the upload is refused with a typed error naming the cap and the plan that raises it, and nothing is stored

#### Scenario: Accepted scores do not count

- **WHEN** a user's proposed score is accepted into the catalog
- **THEN** it no longer counts against their private library cap
