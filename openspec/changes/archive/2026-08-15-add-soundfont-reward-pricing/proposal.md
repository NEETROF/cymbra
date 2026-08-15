## Why

`add-curation-rewards` gave every catalog SoundFont a `point_cost`/`redeemable` and a
reward shop, and `add-soundfont-entitlement-previews` gated the raw `.sf2` bytes behind
entitlement, made a locked font auditionable via a server-rendered preview, and let it be
redeemed. But **nothing sets a font's price**: `UpdateSoundFont` only edits
label/licence/attribution, and no other write touches `point_cost`/`redeemable`. So no
costed font can exist without hand-editing the database — the whole
entitlement→preview→redeem loop is dormant in production. This change adds the missing
control: an admin prices a SoundFont as a redeemable reward from the back office.

## What Changes

- **Admin can price a catalog SoundFont** — set its **cost in curation points**
  (`point_cost >= 0`) and its **redeemability** (`redeemable`). Setting `point_cost > 0`
  turns a free font into a shop reward: its `.sf2` download becomes entitlement-gated (own
  import / grant / music-scope moderator-admin exempt), it is auditioned via its preview
  clip, and it appears in the reward shop for redemption — all already implemented by
  `add-soundfont-entitlement-previews` and `add-curation-rewards`. Setting `point_cost = 0`
  reverts it to free. `redeemable = false` lists it as "coming later" (shown in the shop
  but not redeemable), matching the existing shop contract.
- **Pricing is an admin-scope decision** — stronger than the moderator metadata edit:
  changing what a font *costs* (a product/economy decision) requires a music-scope
  **admin**, not merely a moderator. The existing `UpdateSoundFont` (metadata) stays
  moderator-or-admin; pricing is separate and admin-only.
- **Backend write path** — a dedicated admin RPC (e.g. `SetSoundFontPricing`) that updates
  `point_cost`/`redeemable` on `music.soundfonts` and is surfaced on the admin listing so
  the back office can display the current price.
- **Back office** — the Sound fonts admin screen shows each font's cost + redeemability and
  lets an admin change them (the repo's `Async<T>` union pattern; results via the existing
  toasts).
- **App** — no change: it already locks a costed unowned font, auditions it via the preview
  clip, greys play when no preview exists, and redeems through the shop.

Out of scope: real-money purchases / DRM; the **points-earning** side (how users accrue
points — owned by `add-curation-rewards`); any new grant/shop mechanic (redemption already
exists). This change only lets an operator *set the price*.

## Capabilities

### New Capabilities
- `soundfont-pricing`: the admin control that sets a catalog SoundFont's reward price
  (`point_cost`) and redeemability (`redeemable`) — the write path that turns a free font
  into (or back from) a redeemable shop reward, admin-scope only, surfaced on the
  back-office admin listing.

### Modified Capabilities
- `reward-unlocks`: the shop now offers **accepted** fonts only. Pricing is deliberately
  allowed before acceptance (an operator prices a font while it is in review so it is
  ready on acceptance), which makes the shop the one read path onto `music.soundfonts`
  that does not go through the moderation-visibility gate — so it gains the gate itself,
  on both the listing and the redemption lookup.

<!-- The entitlement gate (soundfont-entitlement, from add-soundfont-entitlement-previews)
     is unchanged: pricing just writes the fields it already reads. -->

## Impact

- **Depends on** `add-curation-rewards` (`point_cost`/`redeemable`, shop, grants) and
  `add-soundfont-entitlement-previews` (entitlement gate, preview clip, redeem in the app).
- **Backend** (`cymbra-music`): a `set_pricing(id, point_cost, redeemable)` write on
  `SoundFontRepo`; a new admin-gated `SetSoundFontPricing` RPC; `point_cost`/`redeemable`
  added to the admin listing row (`AdminSoundFont`) so the back office can show them.
- **Back office**: a pricing control on the Sound fonts admin screen + a store action behind
  the injectable client seam.
- **App**: none.
- **Coverage**: the pure decision/validation (non-negative cost; admin-scope guard) is
  host-tested; the repo write + RPC glue follow the usual coverage exclusions.
