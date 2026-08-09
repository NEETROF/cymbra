## ADDED Requirements

### Requirement: Announcements are published through an outbound port

The backend SHALL publish Discord announcements through a single outbound port
(`DiscordSender`) whose production implementation posts to per-channel Discord webhooks. The
port SHALL be the only place that speaks the Discord protocol, so message selection and
rendering stay host-testable without network access. When the Discord configuration is absent
(no webhook URL), announcing SHALL be a **no-op** that leaves the surrounding operation
successful — an unconfigured deployment MUST behave exactly like a deployment with the feature
disabled.

#### Scenario: Configured deployment publishes the rendered message

- **WHEN** an announceable event occurs and a webhook is configured
- **THEN** the rendered message is posted to the channel configured for that event category

#### Scenario: Unconfigured deployment stays silent and healthy

- **WHEN** an announceable event occurs and no Discord configuration is present
- **THEN** nothing is posted, no error is surfaced, and the originating operation still succeeds

#### Scenario: Discord failure never fails the originating operation

- **WHEN** Discord rejects or times out the request
- **THEN** the originating domain operation remains committed and the failure is retried by the job engine, not propagated to the user

### Requirement: Announcements are routed to the channel of the product they concern

Every announceable event SHALL carry the **product** it belongs to, and SHALL be routed to a
channel belonging to that product's section — including statistics, which belong in their own
product's section rather than in a shared statistics channel. Routing SHALL be keyed by
`(product, category)`. A pair with no configured channel SHALL be a **no-op**; the system MUST
NOT fall back to a default channel, because that would publish one product's data in another
product's section.

#### Scenario: A product's statistics go to that product's channel

- **WHEN** the daily report for one product is published
- **THEN** it is posted in that product's own statistics channel and in no other product's channel

#### Scenario: An unmapped product/category publishes nothing

- **WHEN** an event's `(product, category)` pair has no configured channel
- **THEN** nothing is published and no default channel is used

#### Scenario: A new product cannot leak into an existing one's channels

- **WHEN** a product is added without its channels being configured
- **THEN** its events stay unpublished rather than appearing in another product's section

### Requirement: Announcements are enqueued transactionally, never sent inline

An announcement SHALL be produced by enqueuing a job in the **same database transaction** as
the domain write that justifies it. The request-handling path MUST NOT perform any Discord
HTTP call. Consequently, a domain write that is rolled back MUST NOT produce an announcement,
and a committed domain write MUST eventually produce its announcement even if Discord is
unreachable at commit time.

#### Scenario: Rolled-back write produces no announcement

- **WHEN** the transaction that would justify an announcement is rolled back
- **THEN** no announcement is ever published for that event

#### Scenario: Committed write survives a Discord outage

- **WHEN** the domain write commits while Discord is unreachable
- **THEN** the enqueued job is retried until it succeeds or is dead-lettered, and the announcement is not lost silently

#### Scenario: Request path performs no Discord call

- **WHEN** a client operation triggers an announceable event
- **THEN** the client response time does not depend on any Discord request

### Requirement: Delivery is idempotent under at-least-once execution

Because jobs are delivered at least once, each announcement SHALL carry a stable
**deduplication key** derived from the event identity (not from the delivery attempt), and a
re-delivered job MUST NOT publish a second message for the same event.

#### Scenario: Re-delivered job does not duplicate the message

- **WHEN** the same announcement job is executed twice
- **THEN** the channel shows exactly one message for that event

### Requirement: A named announcement requires the cumulative consent gate

An announcement that identifies a player SHALL be published only when **both** conditions
hold: the player has given the dedicated Discord-visibility consent, **and** the player is
publicly listable (public profile and age-eligible). The check SHALL be evaluated
**server-side at publication time** and SHALL be **fail-closed**: an unknown, private,
not-yet-age-eligible, or non-consenting player MUST NOT be named. A player who fails the gate
SHALL either be omitted from the message or represented only inside an anonymous aggregate.

#### Scenario: Consenting, listable player may be named

- **WHEN** an announceable event concerns a player who consented to Discord visibility and is publicly listable
- **THEN** the message may include that player's public display name

#### Scenario: Consent without public listability is not enough

- **WHEN** the player consented to Discord visibility but their profile is private or they are not age-eligible
- **THEN** the player is not named in any Discord message

#### Scenario: Public listability without Discord consent is not enough

- **WHEN** the player is publicly listable but has not consented to Discord visibility
- **THEN** the player is not named in any Discord message

#### Scenario: Gate is re-evaluated at publication time

- **WHEN** a player revokes consent or makes their profile private between the event and the job execution
- **THEN** the announcement is published without naming them, or suppressed if naming was its only content

### Requirement: Aggregate announcements never identify a player

An aggregate announcement SHALL contain only counts and non-identifying figures. It MUST NOT
contain display names, handles, account identifiers, email addresses, or any value from which a
single player could be re-identified — including an aggregate whose count is small enough to
name one person implicitly, which MUST be suppressed below a configured minimum.

#### Scenario: Digest reports counts only

- **WHEN** the daily digest is published
- **THEN** it contains counts and non-identifying figures, and no name, handle, identifier, or email

#### Scenario: Too-small aggregate is suppressed

- **WHEN** an aggregate figure covers fewer players than the configured minimum
- **THEN** that figure is omitted rather than published

### Requirement: Only allow-listed event categories are announceable

The system SHALL announce only events from an explicit allow-list, split into an **immediate**
tier (rare, high-value: a score or soundfont **accepted** into the public catalog, a season
record beaten) and a **digest** tier (high-frequency activity, aggregated). Software releases
are outside this capability: they are announced by the release pipeline, which owns the version
and its notes, and the backend SHALL NOT attempt to detect or announce them.
The following SHALL NEVER be announced on Discord: authentication events (sign-in, sign-up,
sign-out), any moderation state other than *accepted* (in particular `pending` and `rejected`
items and their reasons), email addresses, raw account identifiers, and any field excluded from
the public profile field set.

#### Scenario: Accepted catalog item is announced

- **WHEN** a proposed score or soundfont is accepted into the public catalog
- **THEN** an immediate announcement is published for it

#### Scenario: Pending and rejected items are never announced

- **WHEN** an item is proposed, is awaiting review, or is rejected
- **THEN** no Discord message mentions it, so moderation state is never disclosed

#### Scenario: Authentication events are never announced

- **WHEN** a user signs up, signs in, or signs out
- **THEN** no Discord message is produced

### Requirement: High-frequency activity is digested, not streamed

High-frequency activity (play sessions, arrivals of new players) SHALL be published as a
**scheduled digest**, not one message per event. Named announcements SHALL additionally be
**throttled per player** over a configured window, so a single player's repeated activity
cannot flood a channel. The system SHALL stay within Discord's documented rate limits, and
MUST NOT drop an announcement silently: a suppressed or throttled event SHALL be observable
(logged or counted).

#### Scenario: Session activity produces one digest, not one message per session

- **WHEN** many play sessions occur during a day
- **THEN** they are reported as a single aggregated digest message

#### Scenario: Repeated activity by one player is throttled

- **WHEN** the same player triggers several named announcements inside the throttle window
- **THEN** at most one is published for that window

#### Scenario: Suppression is observable

- **WHEN** an event is suppressed by the throttle or by an aggregate minimum
- **THEN** the suppression is recorded, so silent truncation cannot be mistaken for absence of activity

### Requirement: Announcements are governed by runtime flags

Discord announcements SHALL be governed by a **global kill-switch** plus a **per-category
flag**, both readable at runtime from the feature-flag service so they can be changed from the
back office **without a redeploy**. When the kill-switch is off, no announcement is published
regardless of category. Flag state SHALL be evaluated at publication time, so a message
already enqueued is still suppressed if the flag was turned off in the meantime.

#### Scenario: Kill-switch suppresses every category

- **WHEN** the global kill-switch is disabled
- **THEN** no Discord announcement is published, whatever its category

#### Scenario: A single category can be muted

- **WHEN** one category's flag is disabled while the kill-switch stays enabled
- **THEN** that category is silent and the other categories keep publishing

#### Scenario: One product can be muted without affecting another

- **WHEN** the flags of one product's categories are disabled
- **THEN** that product's channels go silent and the other products keep publishing

#### Scenario: Flags are honored after enqueue

- **WHEN** a flag is disabled after a job was enqueued but before it runs
- **THEN** the job completes without publishing anything
