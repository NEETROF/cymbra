## ADDED Requirements

### Requirement: The engine is built from a pinned source
The translation engine SHALL be compiled by our own CI from a pinned upstream commit, and SHALL NOT be taken from a prebuilt binary or a moving branch.
The build SHALL publish the engine as an artefact rather than committing it, and SHALL record
the upstream commit it came from. A build that no longer succeeds SHALL NOT be worked around by
moving the pin silently; the pin is what makes every measurement taken on the engine mean
something.

#### Scenario: The engine is produced from the pinned commit
- **WHEN** CI builds the engine
- **THEN** it checks out the pinned upstream commit and publishes the resulting wasm and JS as an artefact

#### Scenario: The pin is visible
- **WHEN** a reader of the repository asks which engine a build contains
- **THEN** the pinned commit is stated in the build definition, not inferred from a lockfile or a date

### Requirement: The engine never occupies a thread that paints
The engine SHALL run in a worker of its own, and SHALL NOT be instantiated in a content script or in any extension page.
On a host whose background context cannot construct a worker, a document created solely to own
that worker SHALL do so, and SHALL perform no other work. The seam the reading code uses SHALL
make the forbidden placements unreachable rather than merely discouraged: no code path SHALL
offer an engine that runs in the caller's own context.

#### Scenario: A translation leaves the page alone
- **WHEN** the engine translates a sentence while the reader scrolls the page
- **THEN** the page's frame pacing is indistinguishable from the same page with no translation running

#### Scenario: A host that cannot construct a worker
- **WHEN** the extension's background context cannot construct a worker
- **THEN** a document is created for the sole purpose of owning it, and the engine runs there

#### Scenario: The reading code cannot place the engine badly
- **WHEN** a content script or an extension page asks for a translation
- **THEN** it obtains one through the seam, and no implementation is available to it that would run the engine in its own context

### Requirement: A translation blocks nothing else
While the engine translates, the rest of the extension SHALL keep answering.
A translation SHALL NOT delay analysis, gloss lookup, deck operations or any other request the
reader's surfaces make, on any target.

#### Scenario: The extension stays responsive mid-translation
- **WHEN** a request is made to the extension's background context while a translation is under way
- **THEN** it is answered without waiting for that translation to finish

### Requirement: The answer is the reader's sentence with their selection marked
A translation SHALL answer with the whole sentence the selection sits in, and SHALL mark, within that translated sentence, the span corresponding to what the reader selected.
The selection SHALL be translated in its sentence rather than on its own, so that its form
carries the grammar the context imposes. The marked span SHALL be identifiable in the answer
without the caller re-reading the source text.

#### Scenario: A fragment carries the sentence's grammar
- **WHEN** the reader selects a verb phrase inside a sentence whose subject and tense determine its form
- **THEN** the marked span holds that form, not the form the fragment would take alone

#### Scenario: A fragment whose translation is shorter than itself
- **WHEN** the reader selects several words that correspond to a single word in the target language
- **THEN** the marked span covers that single word

#### Scenario: The whole sentence is available too
- **WHEN** a translation is returned
- **THEN** the caller can show the translated sentence as well as the marked span

### Requirement: The mark is checked against the selection translated alone
Where the selection is marked SHALL be checked against a translation of the selection on its own, which SHALL only ever move, split or trim the marks and SHALL never be shown to the reader.
The engine places the mark by its own alignment, which can land on a neighbouring word, and a
single tag can only mark one run of words where the reader's words may be separated in the
translation. The check SHALL NOT invent a mark where the engine placed none, and when the
selection alone cannot be translated, the engine's mark SHALL stand as it is.

#### Scenario: The engine marks the wrong word
- **WHEN** the reader selects a word and the engine marks its neighbour in the translated sentence, while the selection's own translation stands exactly once elsewhere in it
- **THEN** the mark is on the selection's own translation, and the neighbour is not marked

#### Scenario: The reader's words are separated in the translation
- **WHEN** the translated sentence puts a word the reader did not select between words they did
- **THEN** the reader's words are marked as separate spans and the word between them is not marked

#### Scenario: Words the sentence's grammar imposes stay marked
- **WHEN** the marked span holds short words — an auxiliary, an article, a pronoun — that the selection translated alone does not contain
- **THEN** they stay marked

#### Scenario: The selection alone gets no translation
- **WHEN** translating the selection on its own fails or times out
- **THEN** the translation is still answered, with the engine's mark as it placed it

### Requirement: Page text is escaped before it is marked
Text taken from the page SHALL be escaped before the selection's markup is placed in it.
The engine is asked to preserve markup, so the sentence it receives is markup; page content that
happens to contain markup characters SHALL NOT be able to change the request's structure.

#### Scenario: A sentence containing markup characters
- **WHEN** the sentence around the selection contains characters that would otherwise be read as markup
- **THEN** the translation is requested and answered normally, and those characters appear as text in both the source and the answer

### Requirement: An engine that does not start is a failure, not a wait
Starting the engine SHALL fail within a bounded time rather than remain pending.
The caller SHALL be told that no translation is available, and SHALL fall back to the answer it
would give without an engine. An engine that failed to start SHALL NOT leave a surface waiting
indefinitely.

#### Scenario: The engine cannot start
- **WHEN** the engine fails to initialise
- **THEN** the request is answered as unavailable within the bound, and the surface shows what it shows when there is no engine

### Requirement: A machine translation is never stored
A translation produced by the engine SHALL be computed for display and discarded, and SHALL NOT be stored on a card, used as a card's gloss, or sent anywhere.
Only dictionary data SHALL be stored as a card's answer. This holds whatever surface asked for
the translation.

#### Scenario: A card made from a translated selection
- **WHEN** the reader creates a card from a selection the engine translated
- **THEN** the card holds its source sentence and its dictionary data, and no machine translation

### Requirement: Without a model the extension behaves as it does today
When no model is present, every surface SHALL answer exactly as it does without the engine.
The engine SHALL NOT be a condition for any existing behaviour, and its absence SHALL NOT be
reported to the reader as an error.

#### Scenario: No model available
- **WHEN** the reader selects a phrase and no model is present
- **THEN** the card answers from the pack exactly as it did before the engine existed, with no mention of a missing engine

### Requirement: A slow engine never costs the reader the pack's answer
A card that asks the engine SHALL bound its wait for it below the card's own answer timeout, and past that bound SHALL answer exactly as it would with no engine.
The pack and the engine SHALL be asked together, and the failure of either SHALL NOT prevent the
other's answer from being shown. A card SHALL NOT show word-by-word rows while it still waits for
the engine.

#### Scenario: The engine is still cold
- **WHEN** the reader selects a phrase and the engine has not answered within its bound
- **THEN** the card shows the pack's answer, as it would with no engine

#### Scenario: The pack fails but the engine answers
- **WHEN** the pack cannot answer a selection that the engine translates
- **THEN** the card shows the translation

#### Scenario: Waiting for the engine
- **WHEN** the pack has answered and the engine has not yet
- **THEN** the card still shows that it is waiting, not word-by-word rows the translation would replace

### Requirement: The translation is shown as a machine translation, in the reader's sentence
A card SHALL show a translation as the reader's sentence, labelled as a machine translation, with the selection's place in it marked, and SHALL render every part of it as text.
Beside a translation the card SHALL show no word-by-word rows and no note that the pack has no
translation; an expression's dictionary gloss SHALL remain. Only a selection of several words
SHALL be translated: a single word keeps its dictionary card.

#### Scenario: A translated phrase
- **WHEN** the engine translates a selection of several words
- **THEN** the card shows the sentence under a label saying it is a machine translation, with the selection's place marked

#### Scenario: Markup from the page
- **WHEN** the translated sentence contains characters that would read as markup
- **THEN** the card shows them as text, and no element is created from them

#### Scenario: A single word
- **WHEN** the reader selects a single word
- **THEN** the engine is not asked, and the card is the word's dictionary card
