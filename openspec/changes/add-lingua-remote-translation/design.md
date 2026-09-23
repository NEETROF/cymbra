## Context

`add-lingua-translation-engine` (PR #532) put the Bergamot engine on the device, behind
`TranslatorPort`: one call taking a sentence and the selection's offsets, answering the translated
sentence with the selection marked in its place. The seam was written so a caller never learns
where the engine runs — on Chromium an offscreen document owns its Worker, on Firefox the event
page does. A second host was always the seam's point.

What it costs on the device is now measured, not estimated: a 25.6 MiB engine and model to
download once, ~195 MiB resident while loaded, ~4.8 s for a first translation on a Galaxy Tab S6
Lite and 377 ms once the host is held loaded. That is workable on the weakest device we own, which
removes the argument that remote is needed to make the feature exist at all.

What has not changed is Lingua's central claim. `lingua-privacy` states, without condition, that
page text stays on the device; the same sentence appears in three store listings and in the privacy
annex. The reader's selected sentence is the most revealing thing Lingua touches — it is what they
are reading, on the page they are reading it on.

## Goals / Non-Goals

**Goals:**

- Record the decision: under what conditions a remote mode is worth building, and what it obliges
  us to.
- Keep one seam, one model and one set of measurements, so remote is a host and not a second
  product.
- State the promise change exactly, before anyone writes the code that forces it.

**Non-Goals:**

- Building it. This change is proposed and parked; see "Migration Plan".
- Replacing local. Remote is an alternative for readers who cannot take the download, never a
  default and never a silent fallback.
- A third-party translation API. See the decision below.
- Serving Safari. Safari's answer is Apple's on-device translation, decided in its own change.

## Decisions

### The same Bergamot model, on our own hardware — not a translation API

A hosted API (Google, DeepL, an LLM) would answer faster to build and better in quality. It is
refused for three reasons, in order of weight:

1. **A third party would receive the reader's sentences.** Moving page text off the device is
   already the hard part of this change; moving it to someone else's company is a different and
   larger one. Whatever we promise, we could only promise on their behalf.
2. **The same model means the same answer.** Remote returns the marked translation the local engine
   would have returned, so `translate/reconcile.ts` — the mark check, measured on 100 sentences —
   applies unchanged, and the card cannot behave differently depending on where the engine ran. A
   different engine would need its own hundred sentences and its own thresholds.
3. **Cost and dependency.** Bergamot is ours to run, CPU-only, already built by
   `lingua-engine-build`. A per-character bill is a tap someone else can turn off.

*Trade-off accepted:* our quality ceiling stays Bergamot's, which the local measurement already
puts at "right sentence, occasionally wrong sense" on idioms.

### Remote requires a signed-in account; local never will

A service that computes on request needs a subject to rate-limit, and an anonymous endpoint that
runs a neural model is an invitation. Local stays account-free, which matters: the reader who wants
no account keeps the whole feature, on the device.

*Rejected — anonymous with an install identifier.* It rate-limits nothing (an identifier is free to
mint) and adds an identifier to requests carrying page text, which is worse than the account.

### The service keeps nothing it translates

No sentence in an access log, in an error, in a metric or in a trace. Metrics count requests and
latency, never content. This is a requirement, not an intention, because it is the only thing that
makes the promise change survivable: *page text leaves the device, and is not kept anywhere.*

*Consequence:* a translation cannot be debugged from production data. Accepted — the local engine
is where translation is debugged, and it is the same model.

### The reader chooses, and the choice never changes itself

Three states — none, local, remote — set by the reader, per device. **No automatic fallback in
either direction.** A local engine that fails must not send the sentence to the network, and a
remote call that fails must not silently download 25.6 MiB. The card answers as it does today with
no engine: the pack's answer, nothing reported to the reader.

*Rejected — "remote until the model finishes downloading".* It is the friendliest behaviour and the
one that would make the promise unkeepable: a reader who chose local would be sending sentences.

### What would have to be true to build it

Written down so the decision is not re-argued from taste:

- A reader has asked for it, or a device that cannot take the local engine has been identified —
  not supposed. Every device measured so far runs it.
- The delivery change has shipped and the setting exists.
- The privacy annex, the three store listings and the App Store answers have new text drafted and
  reviewed, before any code.
- We accept a running cost that scales with use.

If the first is never true, this change is never built, and that is a good outcome: the local
engine serves everyone we can measure.

## Risks / Trade-offs

**The promise becomes conditional, and conditional promises are read as broken** → the disclosure
names the one case and the default stays "never leaves the device". The setting is off by default,
per device, and remote is never reached without an explicit choice and an account.

**A reader turns on remote and forgets** → the state is visible in the setting, not hidden behind a
one-time prompt, and it is per device rather than synced, so it is never turned on somewhere the
reader is not looking.

**Running cost scales with use** → the first Cymbra feature that does. It needs a per-account limit
from day one, not after a bill.

**Two hosts drift** → the same model file and the same `TranslatorPort` contract; a remote answer
that differs from the local one for the same input is a defect, and the local engine is the
reference.

**Store review asks why an extension that promised local processing now sends text** → because the
reader asked it to, off by default, with the policy saying so. Answering that requires the text to
exist before submission, which is why it is a precondition above.

## Migration Plan

1. Not built. This change is proposed and parked behind the delivery change.
2. When the conditions above hold: the promise text first, the service second, the setting's third
   state last.
3. Rollback: the setting loses its third state and the service is withdrawn. A reader on remote
   falls back to **none**, never silently to local — nothing is downloaded without being asked for.

## Open Questions

- **Which limit?** Requests per account per day, and what a reader sees when they reach it. Decided
  when the service is built; it does not change the shape here.
- **Does the delivery change reserve the third state, or add it later?** Reserving costs a wider
  setting shape for something that may never exist; adding it later is a small migration. Settled
  when the delivery change is written.
