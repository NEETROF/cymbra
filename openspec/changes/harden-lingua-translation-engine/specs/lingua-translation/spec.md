## ADDED Requirements

### Requirement: The engine survives a trap
When the engine traps — a WebAssembly trap or the glue's abort, raised while a model is built or a sentence is translated — the worker SHALL report the trap as such and SHALL close itself, and its owner SHALL put it down, start a fresh worker, load the route again and ask the request once more. A request SHALL be asked again once; a second trap SHALL answer that the translation is unavailable. The requests in flight on the worker that trapped SHALL be asked again on the fresh one, once each. An ordinary refusal — no model, a route too long, the engine not loaded — SHALL be passed on as before, with the worker kept.

#### Scenario: A trap in a translation
- **WHEN** the engine traps while translating a selection's sentence
- **THEN** the worker is put down, a fresh one loads the route, the sentence is translated by it, and the reader's card shows the translation

#### Scenario: A trap while a model is built
- **WHEN** the engine traps while a route's model is being built
- **THEN** the worker is put down, a fresh one builds the route and answers the request

#### Scenario: A second trap
- **WHEN** the same request traps the fresh worker too
- **THEN** the answer is unavailable, the card behaves as without an engine, and the log names the pair and the markup's length, never its text

#### Scenario: The selection's two requests
- **WHEN** a marked selection's sentence traps while its fragment alone is in flight on the same worker
- **THEN** both are asked again on the fresh worker, and the card shows the sentence with its mark reconciled

#### Scenario: A refusal is no trap
- **WHEN** the worker answers that a model is not on the device, or a route is too long
- **THEN** the answer is passed on, the worker is kept, and nothing is asked again

### Requirement: The engine holds at most two models
The engine's worker SHALL hold at most two translation models. Before a route is loaded whose models would make a third, the worker SHALL delete the models no loaded route needs, least recently used first, until the models held and the route's make two at most, and SHALL drop the routes that used them. A route whose models are held SHALL cost nothing. The decision SHALL be a pure function, tested apart from the engine.

#### Scenario: A reader of French with English and Spanish
- **WHEN** a reader whose native language is French translates on an English page, then on a Spanish page
- **THEN** the worker holds en-fr and es-en, and nothing is deleted

#### Scenario: A third model
- **WHEN** the worker holds two models and a route is loaded that needs a model it does not hold
- **THEN** the model no loaded route needs, or the least recently used, is deleted before the new one is built, and the worker holds two

#### Scenario: Back to a deleted model's language
- **WHEN** the reader returns to a page whose route's model was deleted
- **THEN** the route is loaded again, and the sentence is translated

### Requirement: A route can be soaked by hand
A tool SHALL run the real engine through a pair's route over the committed corpus of the pair's studied language, outside CI, and SHALL report the inputs that trapped by their corpus id, the count translated, the time per sentence and the memory high-water mark; it SHALL be able to run each input apart, so that a trap does not end the run. The real engine SHALL NOT run in CI.

#### Scenario: en-es before it ships
- **WHEN** the en-es model is pinned in the catalogue and the tool runs for en-es over the English corpus
- **THEN** it reports which selections trap, so that change 35 is decided on measured inputs

#### Scenario: The shipped pairs today
- **WHEN** the tool runs for en-fr and for es-fr
- **THEN** it reports no trap on the committed corpus, and a memory high-water mark of about two models' worth for es-fr
