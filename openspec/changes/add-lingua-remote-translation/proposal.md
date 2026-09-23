## Why

The engine runs on the device, and that costs a 25.6 MiB download and ~195 MiB resident while
loaded. Measured, local now works on the weakest hardware we have — a Galaxy Tab S6 Lite answers a
selection in 377 ms once the host is held — so this is **not** a rescue for a platform that cannot
run it. It is an offer to readers whose device or patience cannot give that download and that
memory, and it is the only shape that would serve them: the same translation, computed elsewhere.

It is written now because the question has an answer and the answer should be on the record. The
delivery change is about to fix the shape of the "Traduction étendue" setting, and whether a third
state is coming changes how that setting is written.

## What Changes

- **The setting gains a third state.** "Traduction étendue" becomes **none / local / remote**
  instead of off/on. Local is the engine on the device; remote computes the same answer on a
  Cymbra service. Still per device, still off by default.
- **A Cymbra translation service**, running **the same Bergamot model** as the local engine, behind
  the existing `TranslatorPort` seam: one more implementation, no new shape. Its answer is the same
  marked translation, so `translate/reconcile.ts` and every measurement taken for the local engine
  carry over unchanged.
- **BREAKING (promise).** Lingua's privacy annex says page text **stays on the device**,
  unconditionally. Remote sends the sentence the reader is reading. The disclosure becomes
  conditional — "unless the reader turns on remote translation" — in the policy, in three store
  listings and in the App Store privacy answers. This is the change's real cost, and it cannot be
  taken back once published.
- **Remote requires a signed-in account**, because a service that computes on request needs a
  subject to rate-limit. Local requires none and keeps requiring none.
- **The service never keeps what it translates**: no sentence in a log, no sentence in a store, no
  sentence in a metric.

## Capabilities

### New Capabilities

None. Remote is a second host for a capability that already exists.

### Modified Capabilities

- `lingua-translation`: where the engine may run. Today the answer is "on the device, off every
  thread that paints". A requirement is added for a remote host, for what it may and may not keep,
  and for the reader's choice between the two. **Depends on `add-lingua-translation-engine`
  (PR #532), which introduces this capability — this change archives after it.**
- `lingua-privacy`: **Lingua's privacy disclosures match what it collects** stops saying page text
  stays on the device unconditionally, and states the one case where it does not.

## Impact

**Products.** Cymbra Lingua (the extension and its three variants) and the backend, which gains a
service. The site's privacy page and the store listings change. Music, Live and the back office are
untouched.

**Not built yet, and deliberately.** This change is proposed, not started. It sits behind the
delivery change that introduces the setting and the local download: there is nothing to add a third
state to until that exists. The design records what would have to be true for it to be worth
building.

**Contract.** A new RPC in the Lingua backend. Nothing existing changes shape, so no client breaks.

**Cost.** Per-request CPU on our own hardware, for readers who opt in — the first Cymbra feature
whose running cost scales with use rather than with accounts.
