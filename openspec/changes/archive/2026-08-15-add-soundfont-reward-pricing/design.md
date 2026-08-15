## Context

`music.soundfonts` already has `point_cost INT NOT NULL DEFAULT 0` and `redeemable BOOL
NOT NULL DEFAULT TRUE` (migration `0016_curation_rewards.sql`). Everything that *reads*
them exists:
- the reward shop (`pg_curation_rewards::shop_items`, `WHERE point_cost > 0 OR redeemable =
  FALSE`) and `RedeemReward` → `curation_grants`;
- the entitlement gate on `GET /soundfonts/{id}` (`soundfont_access::entitlement`, keyed on
  `point_cost == 0` / grant / own-import / music-scope moderator-admin);
- the app's locked-font audition via the preview clip.

What is missing is a **write**. `ScoreGrpc::update_sound_font` (`UpdateSoundFontRequest`)
only sets `label/license/attribution`; `SoundFontRepo::update_meta` never touches
`point_cost`/`redeemable`. The admin listing (`AdminSoundFont`) also omits them, so the
back office can't even show the current price. Pricing a font is therefore impossible
without raw SQL.

Existing guards: `cymbra_platform::guard::require_moderator_or_admin` gates the metadata
edit / moderation; `guard::require_admin` gates admin-only actions (already used for the
auto-accept-on-upload branch). Music-scope resolution is the same `roles_by_scope` path the
other soundfont RPCs use.

## Goals / Non-Goals

**Goals:**
- A music-scope **admin** can set a catalog font's `point_cost` (`>= 0`) and `redeemable`
  from the back office, turning it into (or back from) a redeemable reward.
- The change is **write-only plumbing** over the already-shipped read paths: no new
  entitlement, shop, grant, or app behavior — pricing just populates the fields they read.
- The current price is **visible** on the back-office admin listing.
- The decision is host-testable (non-negative cost, admin-scope guard) without a DB/HTTP.

**Non-Goals:**
- Real-money purchases, DRM, watermarking.
- The points-earning economy (accrual, levels) — owned by `add-curation-rewards`.
- Changing redemption / grants / the shop listing rule — unchanged.
- Bulk pricing / scheduling / promotions — a single per-font set is enough for v1.

## Decisions

### 1. A dedicated `SetSoundFontPricing` RPC (not folded into `UpdateSoundFont`)
Pricing is a **product/economy** decision with a **stronger** authorization than metadata
editing (a moderator may fix a typo; only an admin may decide what a font costs). Folding
`point_cost`/`redeemable` into `UpdateSoundFontRequest` would either weaken pricing to
moderator-level or complicate one handler with two authorization tiers. A separate
admin-gated RPC keeps the split clean and mirrors how `SetSoundFontModerationStatus` is its
own verb.

```
SetSoundFontPricing(id, point_cost, redeemable)  // music-scope admin only
```

The pure decision — `require_admin` in scope, then `point_cost >= 0` — is a small
host-tested guard (`InvalidArgument` on a negative cost, `PermissionDenied` for a
non-admin, `NotFound` for an unknown id). The handler then calls a new repo write.

### 2. `SoundFontRepo::set_pricing(id, point_cost, redeemable) -> bool`
A focused write (`UPDATE music.soundfonts SET point_cost = $2, redeemable = $3 WHERE id =
$1`), returning whether a row matched (→ `NotFound` when it didn't). Kept separate from
`update_meta` so metadata and pricing stay independently authorized and testable; the
`FakeSoundFontRepo` gets the same method for handler tests.

### 3. Surface the price on the admin listing
`AdminSoundFont` gains `point_cost` + `redeemable` (the app's public `SoundFont` does **not**
— the app never needs the raw cost; the shop already exposes it through
`RewardShopItem`). The admin `list_admin_page` already yields these columns via the row; the
handler just copies them into the response so the back office renders the current price and
the pricing control's initial state.

### 4. Back office: priced in the edit drawer, displayed in the listing
The listing gains a read-only **Price** column; the control that changes it lives in the
font's **edit drawer**, next to its label/licence/attribution — pricing is one of the
font's settings, not a row action, and the actions column was already carrying
play/generate, accept, reject, edit and delete. The drawer calls `SetSoundFontPricing`
through the injectable client seam, its state the store's existing `Async<T>` union.

Two gates fall out of putting it there: the section shows only in **edit** mode (pricing
needs an existing row, so a font is never born priced) and only to an **admin**, while the
metadata around it stays moderator-or-admin. Metadata and pricing remain **two writes**
behind one Save, and the pricing call is sent **only when the price actually changed** — so
a moderator saving a label never attempts the admin-only RPC. No new screen.

### 5. Pricing before acceptance, but the shop gates on it
Pricing and moderation stay **independent**: an operator prices a font while it is still in
review so it is ready the moment it is accepted, and the pricing RPC therefore does not
check moderation status. That independence exposes a leak, because `shop_items` /
`shop_item` read `music.soundfonts` **directly** — they are the only app-facing read path
that does not pass through the moderation-visibility gate (the public listing uses
`list_accepted`; the bytes route gates before entitlement). Before this change nothing
could set a price from the UI, so a pending font never met the shop's
`point_cost > 0 OR redeemable = FALSE` predicate in practice; pricing makes that reachable.

So the gate lands in the shop, not in the pricing RPC: both queries add
`moderation_status = 'accepted'`. The **single-item** lookup matters as much as the
listing — it is what `redeem` resolves, so without it a crafted redemption could grant an
unvalidated font by key. Net rule: **the back office may price anything; the app sees
accepted only.**

### 6. Preview independence
A font may be priced whether or not it has a preview: pricing writes the economy fields;
the app **greys** a locked font's play until a preview exists (already implemented) and the
**accept-requires-a-preview** rule is unchanged. So the operational flow is: upload →
generate sample → accept → **price** — but pricing does not itself depend on the preview.

## Risks / Trade-offs

- **A costed font with no preview** is locked but not auditionable → the app greys its play
  (graceful), and the admin is expected to "Generate sample" first. Mitigation: the
  back-office control can hint when pricing a font that has no preview yet (non-blocking).
- **Pricing an already-owned font** (a user holds a grant) — the grant still entitles them
  (the entitlement gate checks grants regardless of current cost), so re-pricing never
  revokes access someone already redeemed. No migration of existing grants is needed.
- **Scope confusion** (moderator vs admin) — mitigated by the dedicated admin-only verb and
  a host-tested guard, so a moderator attempting to price gets a clean `PermissionDenied`.
- **New proto field on `AdminSoundFont`** — additive; regenerate the back-office + (unused
  but consistent) app stubs, as done for `has_preview`.
