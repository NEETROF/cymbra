# Design — generalise-lingua-translation-model-state

## Context

See proposal.md (Why). After `generalise-lingua-translation-catalogue`:

| Where | What it does |
|---|---|
| `model-db.ts` | Files by sha256; one `complete` record `{ version, files }`; `erase()` deletes the database. |
| `model-download.ts`, `model-worker.ts`, `downloads.ts` | The worker downloads `loadTranslationModel(DEFAULT_LANGUAGE)`; progress over its files. |
| `model-controller.ts`, `setting.ts` | One state: `absent`, `downloading`, `ready`, `failed`, `interrupted`, `removed`. `ready()` reads the recorded state only. |
| `engine-worker.ts`, `channel.ts`, `offscreen-engine.ts`, `relay.ts` | One model loaded; `translate(markup)` and `warm()` name no language. |
| `port.ts`, `wire.ts`, `messaging-port.ts`, `answer-memory.ts`, `selection-card.ts` | `TranslationRequest = { sentence, selection }`. |
| `background.ts` | Answers a translation when `ready()`. |

## Goals / Non-Goals

**Goals:**
- The device keeps the models of its reader's languages' routes, and no other.
- One state for the setting, which knows which models are complete.
- A translation goes to its document's language's route, or is answered as without a model.
- English unchanged: one route, one model, the same download and the same answers.

**Non-Goals:**
- Translating through English (`add-lingua-spanish-translation-pivot`, with `translateViaPivoting`).
- A setting per language: « Traduction étendue » stays one checkbox per device.
- Downloading a model for a language the reader added without asking. The setting offers it, with
  its cost.

## Decisions

### D1 — Completeness per model, the old record read as its model's

The database keeps one record per model, `complete/<id>` = `{ files }`, beside the files by sha256:
- **`complete(model)`** reads the model's record, or the legacy `complete` record when its `version`
  is that model's id. A device updated with the English model stored is therefore complete without
  a rewrite.
- **`markComplete(model)`** writes the model's record.
- **`prune(keep)`** deletes:
  - every record of a model not in `keep`, the legacy one included once its model is not kept;
  - every file that no kept model names.

  A file two models share is kept, and so is a model two routes share.
- **`erase()`** still deletes the whole database when the setting is turned off.

### D2 — The models a device needs

`modelsFor(catalogue, languages)` is the union of the routes of `languages`, in their order, without
duplicates. A language without a route needs nothing. The controller reads the languages through a
dependency, `languages()`. The background serves it from the stored backup's profile,
`profile.studied_languages`, reduced to the shipped pairs as `acceptedLanguages` does.

It reads the profile without an engine. A Chromium service worker that wakes for a translation
would otherwise instantiate the reading engine and its pack just to learn the reader's languages. The
engine writes the languages under its enum's names (`"English"`, `"Spanish"`). A test in
`lingua-core` pins that layout, naming the extension's reader. A backup without a profile is
English.

The controller follows the reader's « Langues étudiées » the next time it reconciles:
- when a settings view asks for the status;
- when the background starts;
- before it downloads.

*Rejected — the background's reading engine (`acceptedLanguages(port)`).* It is correct by
construction. But on Chromium it instantiates the engine and its pack in the service worker, and it
holds the profile as of its last hydration: a change made in the side panel's own engine reaches it
only through a restore that would race its other clients.

### D3 — One state over the needed models

The state stays one value per device, and the phases that concern models name the complete ones:
- **`ready { models, languages }`**: every needed model is complete. `models` lists them, and
  `languages` lists the reader's languages whose whole route they make. Surfaces and the background
  read readiness per language from it, without the catalogue.
- **`missing { models, languages, total }`** (new): the setting is on, a needed model was never
  downloaded, and `total` bytes would fetch what is missing. « Télécharger » (the `resume` command)
  downloads it.
- **`removed`**: a model recorded complete is gone. The browser removed it, and nothing is fetched
  unasked, as before.
- **`downloading` and `interrupted`**: progress over every needed model's files. Complete files count
  as received at once, as a resumed download counts them today.
- **`failed`**: unchanged.

The two cases are told apart on reconciling. A missing model that the state recorded as complete
means `removed`. A missing model the state never had means `missing`.

A stored `ready` without `languages`, written by the release before this one, is read as `ready` for
English. That release recorded `ready` only once the English model was complete. The next reconcile
names its models.

The setting is offered only while one of the reader's languages has a route. A reader of a language
whose translation has not shipped yet sees no « Traduction étendue ».

When the setting is on, reconciling also prunes the models no longer needed (D1). Removing Spanish
from « Langues étudiées » frees es→en at the next reconcile, and keeps en→fr.

### D4 — The download takes a list of models

The download worker is told which models to fetch (`{ op: "download", models: [id…] }`). It reads
them from the catalogue and fetches each missing file, verified as today. It marks each model
complete as soon as its files are in, and reports one progress over all of them. The controller
sends the needed models; the offscreen bridge and the event page relay the list.

### D5 — A translation names its language

- **The request.** `TranslationRequest` gains `language`, the document's studied language. The
  selection card takes it from the session (`language: () => this.language`), as it takes the
  calibration. The answer memory keys on it too: the same sentence in another language is another
  question.
- **The background** answers when `ready(language)` holds:
  - the setting is on;
  - the state is `ready` or `missing`;
  - the language is among those it names translatable.

  Surfaces read the same rule to decide whether to offer a translator at all.

  A Spanish document is not translated while English is ready, and English stays translated while
  Spanish's model is `missing`.
- **The engine** (`translate(markup, language)`, `warm(language)`) loads the language's route in its
  worker. It keeps one engine instance and one translation model per id:
  - a route of one model translates with it;
  - a route of several answers « not supported », which the relay turns into « unavailable », until
    the pivot change;
  - a language without a route answers « no model ».

*Rejected — one setting per language.* Two checkboxes for one engine and one memory budget would
let a reader enable the expensive pivot without seeing that it adds to English.

## Risks / Trade-offs

- **A language added while the setting is on** costs a download the reader must ask for. The
  setting shows the cost and the action; until then, that language's sentences are answered as
  without a model.
- **The first status after the update** rewrites `ready` with its models. A translation asked before
  that rewrite is answered as without a model, once.
