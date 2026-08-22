# soundfont-entitlement Specification

## Purpose
TBD - created by archiving change add-soundfont-entitlement-previews. Update Purpose after archive.
## Requirements
### Requirement: Entitled-only delivery of raw SoundFont bytes

The SoundFont delivery route (`GET /soundfonts/{id}`) SHALL serve the raw `.sf2`
bytes only to an **entitled** caller. A caller is entitled when **any** of the
following holds:

- the font is **free** (`point_cost = 0`); OR
- the caller **owns** the font (a `music.curation_grants` row for that caller and
  that font — i.e. a redeemed reward); OR
- the font is the caller's **own import** (the caller uploaded it); OR
- the caller's **effective plan grants the `soundfonts.library` unlock** (`premium`,
  whatever its source — store, web, trial or admin); OR
- the caller is a **music-scope moderator or admin** (exempt).

The existing moderation-visibility gate SHALL be applied **before** entitlement: a
caller who cannot view the font at all (not accepted and not a moderator) is
refused regardless of entitlement.

A refusal for lack of entitlement SHALL be **indistinguishable from a missing
font** (the same not-found response), so the endpoint does not reveal the
existence of costed fonts.

The entitlement decision SHALL be a pure, host-testable function (no DB/HTTP) so
each branch is unit-tested; the route supplies its inputs (font row incl.
`point_cost`/`uploaded_by`, whether a grant exists, whether the plan grants the
unlock, whether the caller is a music-scope moderator/admin). The plan input SHALL
be false while `plans.enabled` is off. The app's client-side lock mirror SHALL apply
the same rule so a plan-unlocked font is selectable in the picker.

#### Scenario: Free font is downloadable by any authenticated caller
- **WHEN** an authenticated caller requests a font with `point_cost = 0` that is accepted
- **THEN** the server streams the raw `.sf2` bytes

#### Scenario: Owned costed font is downloadable
- **WHEN** an authenticated caller requests a costed font for which a `curation_grants` row exists for that caller
- **THEN** the server streams the raw `.sf2` bytes

#### Scenario: Own import is always downloadable by its owner
- **WHEN** a caller requests a costed font that the caller uploaded
- **THEN** the server streams the raw `.sf2` bytes

#### Scenario: Plan-unlocked costed font is downloadable
- **WHEN** a caller whose effective plan grants `soundfonts.library` requests any accepted costed font
- **THEN** the server streams the raw `.sf2` bytes without any grant row

#### Scenario: Lapsed plan is refused like any locked font
- **WHEN** a caller whose plan lapsed requests a costed font they never redeemed
- **THEN** the server responds exactly as for a missing font

#### Scenario: Music-scope moderator/admin is exempt
- **WHEN** a music-scope moderator or admin requests any costed font
- **THEN** the server streams the raw `.sf2` bytes regardless of grants

#### Scenario: Locked costed font is refused as not-found
- **WHEN** an authenticated caller with no grant, no plan unlock, who is not the uploader and not a music-scope moderator/admin, requests a costed font (`point_cost > 0`)
- **THEN** the server responds exactly as it would for a missing font, revealing nothing about the font's existence
- **AND** no `.sf2` bytes are sent

#### Scenario: Instrument use path is gated identically
- **WHEN** the app attempts to load a locked costed font as the active instrument (via `GET /soundfonts/{id}`)
- **THEN** the request is refused server-side by the same entitlement gate, so a locked font can never be loaded even if the client-side guard is bypassed

