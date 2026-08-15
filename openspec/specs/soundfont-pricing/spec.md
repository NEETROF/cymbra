# soundfont-pricing Specification

## Purpose
TBD - created by archiving change add-soundfont-reward-pricing. Update Purpose after archive.
## Requirements
### Requirement: Admin sets a SoundFont's reward price

The server SHALL let a **music-scope admin** set a catalog SoundFont's `point_cost`
(curation points, `>= 0`) and `redeemable` flag via a dedicated pricing action
(`SetSoundFontPricing`). The action SHALL update those fields on the font and return
success only for a known font. It SHALL NOT change any other field (label, licence,
attribution, moderation status, object bytes, preview).

A negative `point_cost` SHALL be rejected as invalid. Setting `point_cost = 0` reverts the
font to **free**; `point_cost > 0` makes it a **costed** reward whose raw `.sf2` bytes are
then entitlement-gated (existing behavior). `redeemable = false` marks it "coming later"
(listed in the shop but not redeemable); `redeemable = true` makes it redeemable when
costed. Pricing is independent of whether the font has a preview clip.

The authorization SHALL be **admin**, strictly stronger than the moderator-or-admin gate on
metadata editing/moderation: a music-scope moderator SHALL NOT be able to price a font.

#### Scenario: An admin prices a free font as a reward
- **WHEN** a music-scope admin sets a font's `point_cost` to a positive value with `redeemable = true`
- **THEN** the font is stored with that cost and redeemability, so its download becomes entitlement-gated and it appears in the reward shop

#### Scenario: Pricing back to free
- **WHEN** an admin sets a costed font's `point_cost` to 0
- **THEN** the font is free again and its bytes are served to any authenticated caller

#### Scenario: A moderator cannot price a font
- **WHEN** a music-scope moderator (not an admin) invokes the pricing action
- **THEN** the request is refused (permission denied) and the font's price is unchanged

#### Scenario: A negative cost is rejected
- **WHEN** an admin sets a `point_cost` below 0
- **THEN** the request is refused as invalid and the font's price is unchanged

#### Scenario: Pricing an unknown font is not-found
- **WHEN** an admin prices a font id that does not exist
- **THEN** the request reports not-found and nothing is written

#### Scenario: Pricing leaves other fields untouched
- **WHEN** an admin changes a font's price
- **THEN** its label, licence, attribution, moderation status, stored bytes and preview clip are unchanged

### Requirement: The admin listing shows the current price

The privileged (music-scope moderator/admin) SoundFont admin listing SHALL report each
font's `point_cost` and `redeemable`, so the back office can display the current price and
initialize the pricing control. The public catalog listing (`ListSoundFonts` / the app)
SHALL NOT expose the raw `point_cost`/`redeemable`; the app continues to learn a font's
locked/preview state through the reward shop and `has_preview` as today.

#### Scenario: The admin listing carries the price
- **WHEN** a moderator/admin lists the catalog
- **THEN** each row includes its `point_cost` and `redeemable`

#### Scenario: The public listing omits the raw price
- **WHEN** the app lists the public catalog
- **THEN** no raw `point_cost`/`redeemable` is included on the public entries

