## Why

The on-device engine (Bergamot) answers a selection without anything leaving the device, and it
should stay the default. It is also a small model: measured on a twelve-sentence corpus, it misses
idioms a reader stops on precisely because they are idioms. A hosted language model renders them
well — "four-poster bed" becomes « lit à baldaquin », « prendre la poudre d'escampette » becomes
"took to his heels" and « salió pitando » — in each of the six pairs of the language matrix
(en-fr, es-fr, es-en, en-es, fr-en, fr-es), at a cost measured in hundredths of a cent per sentence.

The reader should be able to ask for that better answer, on purpose, one selection at a time, and
only after choosing in the settings that a sentence may leave the device to get it.

This change replaces `add-lingua-remote-translation`, parked since 2026-09-23. That change sent the
sentence to the **same** Bergamot model on Cymbra's hardware, so its only gain was sparing a weak
device the download; none of its four gates was ever met, and it explicitly refused a hosted model.
The question it answered is no longer the one being asked, so it is withdrawn rather than amended.

## What Changes

- **A setting, « Traduction en ligne », off by default**, per device, shown only to a reader
  whose account was given access. Its note says that the selected sentence is sent to an external translation service to
  be translated, and is kept by no one. It does not name the service; the privacy policy does.
- **A button, « Traduction en ligne », on a card that shows a translation of several words**,
  present only when the setting is ticked. Pressing it asks for the sentence once and, when the
  answer lands, shows it in place of the engine's, labelled as an online translation, with the
  selection marked. Single words keep their dictionary card, as today.
- **Nothing is sent without that press.** The local order is unchanged: a word is answered by the
  pack, a phrase or sentence by the pack then the on-device engine. The button needs neither the
  engine nor its model.
- **A translation service in the Lingua backend**, behind one new RPC, that relays the sentence to
  a hosted model, then to a second provider if the first fails, and otherwise answers
  "unavailable". It asks the model for the translation and for the words that render the
  selection, and returns the sentence with marks, in the shape the card already renders.
- **Access by grant, as a feature beta.** An operator gives access to one account at a time from the
  back office — for example to a reader who asked on Discord — and can withdraw it from one account
  or from everyone at once, without a release. Nobody has access until granted. This consumes
  Lingua feature betas, introduced by `add-lingua-feature-betas`, which must land first.
- **A per-account daily limit**, tunable at runtime (100 by default), that does not apply to Lingua
  staff.
- **Usage and estimated cost in the back office**: the Lingua console shows online translations
  answered, failed and refused, and their estimated cost, per day, provider and pair. Daily totals
  only, with no account and no text, like the rest of that console.
- **The service keeps nothing**: no sentence in a log, a store, a metric or a trace.
- **BREAKING (promise).** The Lingua privacy annex says page text **stays on the device**,
  unconditionally, and the store listings repeat it. Both become conditional: page text leaves the
  device only for a selection the reader sends with the button, after ticking the setting. The
  annex names the providers that translate it.
- **Withdrawn:** `openspec/changes/add-lingua-remote-translation/`, deleted in this change.

## Capabilities

### New Capabilities

None. Online translation is a second source for an existing capability, `lingua-translation`.

### Modified Capabilities

- `lingua-translation`: adds the setting, the button, the service's answer and its limits, and
  amends **A machine translation is never stored** so that a sentence may be sent once, when the
  reader presses the button.
- `admin-lingua-console`: adds the online translation usage and estimated cost.
- `lingua-privacy`: **Lingua's privacy disclosures match what it collects** stops saying page text
  stays on the device unconditionally, names the one case where it does not, and names who
  translates it.

## Impact

**Products.**
- **Cymbra Lingua** — new: the setting, the button, a client for the new RPC (the extension's three
  variants and the Safari host app ship the same code). Consumed: the account session and its
  bearer token, the existing card and its marked-translation rendering.
- **Backend (`backend/lingua`)** — new: a translation service and its RPC, two outbound provider
  adapters, a daily per-account limit. Consumed: the auth interceptor, the platform rate limiter,
  runtime feature flags, configuration from the environment.
- **Site (`apps/site`)** — the Lingua privacy annex changes in French, English and Spanish.
- **Store listings and App Store privacy answers** for Lingua are corrected.
- **Back office** — new: online translation usage and cost in the Lingua console. Consumed: the
  console's window, tiles and day-series chart.
- **ID, Music, Live** — untouched.

**Contract.** A new RPC in a new `.proto` under `backend/lingua/proto/`. Nothing existing changes
shape, so no installed client breaks. It is the first Lingua RPC that carries page text; the
privacy allow-list test is extended to cover it.

**Other changes.** Four in-flight changes named `add-lingua-remote-translation` as the holder of
*Lingua's privacy disclosures match what it collects* and added beside it rather than modify it;
they now name this change. The other mentions of the withdrawn change are history in their
designs and are left as written.

**Dependencies.** Two external translation providers, reached over HTTPS with keys held by the
server. No new client dependency.

**Cost.** Measured on the corpus: about $0.00014 per sentence with the primary provider and about
$0.00011 with the secondary. The daily limit bounds it per account; access by grant bounds it
globally.
