## Context

See proposal.md for why. What exists today, and what this design leans on:

- **The card already renders a marked translation.** `translate/port.ts` defines
  `TranslationResult` and `MarkedTranslation { sentence, marks }` (`translate/markup.ts`); the card
  renders it as text with `<mark>` elements (`reading/wordpopup.ts`, `renderTranslation`). The
  on-device engine reaches it through `reading/selection-card.ts` (`openExpression`,
  `translateBounded`, `show`), which already guards against writing over a card the reader left.
- **Page code never calls the network.** The page sends `chrome.runtime` messages; the background
  owns the engine, the session and the gRPC-web clients (`net/transport.ts`, `net/api.ts`, bearer
  token, audience `lingua`, single-flight refresh).
- **Settings have one builder** (`reading/settings-view.ts`, `mountSettings`), hosted by the popup,
  the side panel and the drawer. The remote-voices toggle is the closest precedent: off by default,
  per device, sends text to a third party, note under the toggle.
- **The Lingua backend** (`backend/lingua`) has a module/core/grpc split, protos in
  `backend/lingua/proto/`, services mounted in `backend/server/src/main.rs` behind the strict
  `AuthInterceptor`. A Redis-windowed limiter exists (`platform/src/ratelimit.rs`). Runtime flags
  are declared in `feature-flags/src/registry.rs` and read through `server/src/flags.rs` on every
  call. Outbound HTTPS follows the port + thin reqwest adapter pattern (`discord/src/webhook.rs`,
  `plans/src/billing/rc_client.rs`).
- **`backend/lingua/tests/privacy_allow_list.rs`** checks that no proto field names a browsing
  signal. Its list of protos is written by hand.

Measured on 2026-10-11 with the prompt and answer shape of D3 and D4: 22 sentences covering the
six pairs of the language matrix (en→fr, en→es, es→fr, es→en, fr→en, fr→es), each selection an
idiom, a phrasal verb or a precise verb:

| Provider | Median latency | Selection marked | Cost / sentence |
|---|---|---|---|
| Claude Haiku 5.5 | 1.8 s | 22/22 | ≈ $0.00011 |
| Gemini 3.1 Flash-Lite | 0.8 s | 22/22, including the split « remettait … à plus tard » | ≈ $0.00014 |
| Bergamot (on device, today) | — | — | 0 |

Grammar slips seen on the corpus, every run: Haiku « Bien qu'il pleuvait » (subjunctive) and « le
immense » (elision); none from Gemini.

## Goals / Non-Goals

**Goals:**
- One more answer on a card, on demand, with the same rendering and the same "never written over"
  guard as the engine's.
- A relay that holds the keys, enforces the limit, tries two providers, and keeps nothing.
- A disclosure that is true before the first build that can send.

**Non-Goals:**
- Replacing the on-device engine, or making it optional for this feature.
- Translating single words, whole pages, or anything the reader did not press the button for.
- Storing, caching or reusing online translations, server-side or on a card.
- Restricting the feature by plan. Access is "signed in"; a later change may narrow it.
- Running a model on Cymbra hardware (`add-lingua-remote-translation`, withdrawn).

## Decisions

### D1. A Cymbra relay, not a call from the extension to the provider
The extension calls a new Lingua RPC; the server calls the provider.
- **Why:** the provider keys never ship in a public extension; the per-account limit and the kill
  switch live where they cannot be bypassed; providers can be swapped without a store review.
- **Alternative — direct calls with keys in the build:** keys would be extractable from any copy of
  the extension. Refused.
- **Alternative — the reader brings their own key:** a cost moved to the reader for a feature that
  is meant to be one press. Refused.

### D2. A hosted language model, with a second provider as fallback
Primary Claude Haiku 5.5 (Anthropic), secondary Gemini 3.1 Flash-Lite (Google), behind one port
trait with one adapter each.
- **Why:** both translate idioms Bergamot misses; two providers mean one outage does not take the
  button down. Both marked every selection on the corpus. Haiku is primary because its cost is
  covered by API credits Cymbra already has; Gemini is faster and made none of Haiku's grammar
  slips. Which one is primary is configuration (D2's last point), so the order can be swapped by a
  deploy if quality or cost says so.
- **Alternative — a dedicated machine-translation API (Google Translate, DeepL, Azure):** priced
  per character at roughly 100× the cost measured here, and none returns which words render the
  selection.
- **Alternative — an open model self-hosted (Hy-MT2 7B, measured at 0.6 s on an M2 Max):** good
  marks, weaker idioms, and a machine to keep running. Not now.
- The model identifiers and which provider is primary are configuration, not code, so a newer
  model or a swapped order is a deploy, not a change.

### D3. The provider names the words; the server places the marks
The provider is asked for structured output: `{ translation: string, highlighted: string[] }`,
where each entry is a substring of `translation`, in order. The server finds each entry in the
translation after the previous one and turns it into a mark; if any entry is missing, the answer
carries no marks.
- **Why:** asking a model to keep `<b>` tags lost or misplaced the mark in 1–2 sentences out of
  12; naming the words separately, as a list, marked 22/22 for both providers across the six
  pairs. A list, not a string, because a rendering is often split ("remettait … à plus tard"):
  with a single string, Gemini missed exactly those.
- **Why server-side:** the wire answer is the `MarkedTranslation` the card already renders. The
  extension needs no new rendering code and never parses model output.
- **Alternative — reuse `reconcileMarks` with a second request for the fragment alone:** doubles
  cost and latency for a problem the structured answer already solves.

### D4. The prompt carries the selection, not tags
The request holds the sentence and the selection's offsets. The server builds the prompt with the
selection marked, three short examples, and the target language; the examples are fixed so the
prompt prefix stays stable. With Haiku, thinking is left adaptive and structured output is
required: disabling thinking made the model write its own corrections into the answer ("Wait, I
need to apply the tags…"), which structured output prevents.

### D5. The pair is the document's language and the reader's language
The request names both, as the engine's request does (`translate/host/relay.ts` forms the pair
from the document's language and the native language). The service accepts any two different
languages among Lingua's (en, fr, es today) — the six pairs of the matrix — and refuses the rest.
There is no pivot: where the engine goes es→en→fr, the provider translates es→fr directly. The
button does not depend on `MARKED_PAIRS` either: the provider marks its own answer, so a pair the
engine translates without a mark still gets one online.

### D6. Limit and switch are runtime flags
- `lingua.online_translation.enabled` — default **false**. Off: every request is answered
  unavailable without asking a provider.
- `lingua.online_translation.daily_limit` — default 100 answered requests per account per rolling
  day, in the platform cache under `rl:lingua_online_translation:{account}`.
- A request is counted when a provider answered, not when it was refused or failed, so an outage
  does not spend the reader's day. `ratelimit::check` increments on every call, so this needs a
  read-then-increment variant: read the count before asking a provider, increment after an
  answer. Two concurrent presses may both pass at the limit; one extra sentence is acceptable.
- The sentence bound (1 000 characters) is a constant shared by the client check and the server
  check.

### D7. The RPC
A new `online_translation.proto` in `cymbra.lingua.v1`:
`OnlineTranslationService.Translate(TranslateRequest) returns (TranslateResponse)`.
- Request: `sentence`, `selection_start`, `selection_end` (UTF-16 offsets, as the extension counts
  them), `source_language`, `target_language`.
- Response: a `oneof` — `translated { translation, repeated Mark marks }` / `unavailable {}` /
  `limit_reached { resets_at }`. The provider is not in the response: the extension has no use for
  it, and the interface must not name it.
- Refused without a token (`UNAUTHENTICATED`) by the strict interceptor; an over-long sentence is
  `INVALID_ARGUMENT`.
- Field names pass `privacy_allow_list.rs`, which gains this proto. `backend/lingua/src/lib.rs`'s
  doc comment, which says no page text crosses the wire, is amended to name this one RPC.

### D8. The extension: a background client, a card button, a setting
- Storage key `cymbra-lingua-online-translation` (boolean, `chrome.storage.local`, per device).
- The background owns the client call (as for every RPC) and answers a new runtime message; the
  card never touches the network.
- The card shows the button when the setting is ticked and `account:state` says signed in. On
  press: a "translating online" line, then `show()` replaces the translation through the same
  generation guard the engine uses. The button is disabled once pressed on that card.
- Strings in `i18n/{fr,en,es}/card.ts` and `settings.ts`, in the reader's interface language
  (« Traduction en ligne », "Online translation", « Traducción en línea »). The setting's note
  links to the annex in the same language (`/confidentialite/`, `/en/privacy/`,
  `/es/privacidad/`).

### D9. Keeping nothing
The relay logs per request: account id, pair, provider, outcome, latency, token counts. A test
captures the tracing output of a full request, including a provider error path, and asserts that
the sentence and the translation appear nowhere. Provider errors are mapped without their body,
as `rc_client.rs` does. Both providers are used on paid terms that exclude training on requests
(Anthropic API; Gemini API with billing enabled — the free tier is excluded by its terms for
apps serving EEA, Swiss or UK users).

## Risks / Trade-offs

- **[A broken promise is not taken back]** Once published, the annex and listings say text can
  leave the device. → Text leaves only on a press, after an unticked-by-default setting, and the
  disclosure ships before the first build that can send (tasks group 1).
- **[Store reviews]** Firefox requires declared data collection, and AMO refused a release once
  over it (1.2.0). `websiteContent` is already declared optional in `tool/manifests.mjs` (for the
  deck's source sentence), so no new category is needed; its comment and `REVIEWERS.md` ("Where
  the add-on reaches the network") must name the new request, and reviewers will read it.
- **[Cost runs away]** A script with one account could press the button all day. → Daily limit per
  account, global kill switch, and a spend cap on each provider account.
- **[Provider output is wrong or partial]** → Structured output validated server-side; an
  unreadable answer counts as a failure and moves to the next provider; marks are dropped rather
  than misplaced.
- **[Grammar slips]** Haiku missed the subjunctive twice on the corpus. → Labelled as a machine
  translation, like the engine's; measured again before switching the flag on (task 5.3).
- **[Taking the feature back later is felt as a loss]** It is open to every signed-in reader. → A
  later change that narrows access must carry over the readers who already use it.

## Migration Plan

1. Ship the disclosure (annex fr/en, listings) — group 1.
2. Deploy the backend with the flag **off**; set the provider keys in the server environment.
3. Release the extension: the setting appears, the button answers "unavailable" while the flag is
   off.
4. Turn the flag on.

Rollback: turn `lingua.online_translation.enabled` off. Every press then answers "unavailable" and
nothing is sent to a provider; the RPC stays, so no installed client breaks.
