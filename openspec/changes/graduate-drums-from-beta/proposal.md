## Why

The drums were a beta: `drums.enabled` defaults **off** and was rolled out as
`beta:midi-drums`, so only staff and members of the `midi-drums` campaign could see
percussion scores. In production the feature is now open to every player through a
back-office override, and the maintainer wants the beta gone (2026-09-28).

That leaves the code and the specs saying the opposite of production:

- the registry default is **off**, so the feature is only open while a database
  override says so — clearing the override, or a flag store that cannot be read,
  silently takes the drums away from every player;
- `music-drums-visibility` still requires the audience to be *staff plus the beta
  campaign, and no one else*, and requires every uncertainty about the caller to
  resolve to *not eligible* — a rule that protected a closed beta and now only
  protects players from a released feature;
- the app falls back to **hidden** before its first flag fetch;
- the runbook still presents `midi-drums` as the live beta example.

## What Changes

- **`drums.enabled` defaults on and stays as a kill-switch.** Every player sees the
  drums without any override; turning the flag off in the back office hides them
  for everyone, with no release. The flag is not removed: the backend keeps
  enforcing it on every path that can disclose or accept a percussion score, so the
  kill-switch is real and not only cosmetic.
- **The drum audience is every player**, no longer staff plus a campaign.
  Membership of `midi-drums` no longer matters; the campaign is closed afterwards,
  which then changes nothing for anyone.
- **Uncertainty about the flag store no longer hides the feature**: an unreadable
  store serves the code default, which is now on. Uncertainty about a *score* is
  unchanged (percussion rows stay classified from their bytes; `unknown` stays
  reachable).
- **The app's fallback before its first flag fetch becomes visible**, matching the
  server default (defence in depth only — the backend decides).
- **Docs**: the flag's English and French descriptions, the registry comment, and
  the deployment runbook's feature-beta section (the live example becomes a
  completed one).

Out of scope: removing the drum gate from the code (it is the kill-switch), and
the other specs that use `midi-drums` as an *example* of a feature campaign
(`runtime-feature-flags`, `feature-flags-admin`, `admin-plan-console`,
`music-plan-entitlements`, `music-premium-paywall`) — their scenarios describe how
any feature campaign behaves and stay true.

## Capabilities

### Modified Capabilities

- `music-drums-visibility`: the audience requirement (renamed — it is no longer
  "staff plus the beta campaign") and the fail-closed requirement's caller half.

## Impact

- **Music (backend)**: `backend/feature-flags/src/registry.rs` default and doc;
  tests that assume the default is off. No change to the gate's enforcement paths.
- **Music (app)**: `apps/music/lib/main.dart` fallback of the `drums.enabled` read;
  its tests. Takes effect with the next app release; until then the server value
  already opens the feature.
- **Back office**: the French description of `drums.enabled`.
- **Ops**: after deploying, delete (or keep) the global override — both now mean
  "on" — then close the `midi-drums` campaign. Order no longer matters once the
  default is on, but the runbook still states it.
- **Ordering with `add-drum-audio-channel`**: that open change MODIFIES the same
  audience requirement under its old name. It SHALL be archived **before** this
  one; archiving it after would look for a requirement this change renamed, and
  `openspec archive` aborts silently in that case.
