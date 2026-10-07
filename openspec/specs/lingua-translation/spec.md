# lingua-translation Specification

## Purpose
TBD - created by archiving change add-lingua-translation-engine. Update Purpose after archive.
## Requirements
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
A translation SHALL answer with the whole sentence the selection sits in, and SHALL mark, within that translated sentence, the span corresponding to what the reader selected, for a pair whose marks have been measured: en-fr, and es-fr, whose marks through English `release-lingua-spanish-translation` measured on the programme's first tier. For a pair whose marks are not measured, the selection SHALL be sent untagged, in a single request, and the answer SHALL be the translated sentence without a mark.
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

#### Scenario: A Spanish sentence
- **WHEN** a reader of French selects a word in a Spanish sentence
- **THEN** the answer is the whole sentence in French, through English, the selection marked

#### Scenario: A language whose marks are not measured
- **WHEN** a translation is asked for a pair whose marks were not measured
- **THEN** the answer is the whole sentence in the reader's native language without a mark, from a single request to the engine

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

### Requirement: The engine is not torn down between a reader's selections
While a page is being read, the context hosting the engine SHALL be kept loaded, so that a selection made long after the previous one does not pay for loading the engine again.
Holding it SHALL cost nothing until the engine is first used: the engine SHALL NOT be loaded
before a translation is actually asked for. A shipped build, which carries no engine, SHALL hold
nothing.

#### Scenario: A selection made minutes after the last one
- **WHEN** the reader selects a phrase long after the previous selection on the same page, on a target whose background context is otherwise torn down when idle
- **THEN** the engine answers without reloading its model, and the card shows a translation rather than falling back

#### Scenario: A page where nothing is ever translated
- **WHEN** a reader opens a page and selects nothing
- **THEN** the engine is never loaded, and the memory it would hold is never taken

#### Scenario: The host goes away for its own reasons
- **WHEN** the context hosting the engine is unloaded despite being held
- **THEN** the reader's page holds it again, and translation keeps working without reloading the page

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
A card SHALL show what the pack knows as soon as the pack answers, whatever the engine is doing, and SHALL NOT wait for the engine before answering.
While a translation is still on its way, the card SHALL say so alongside the pack's answer, so
word-by-word rows are never read as the last word on the selection. When the translation lands it
SHALL replace them; when it is known that none is coming, the card SHALL simply stop saying one is.
A card the reader has since replaced, closed or acted on SHALL NOT be written over.

#### Scenario: A cold engine on a slow device
- **WHEN** the reader selects a phrase and the engine needs seconds to load its model
- **THEN** the pack's answer is shown at once, with a line saying a translation is still coming

#### Scenario: The translation arrives
- **WHEN** the translation lands while the card is still the one the reader opened
- **THEN** it replaces the word-by-word rows, and the card stops saying a translation is coming

#### Scenario: No translation is coming after all
- **WHEN** the engine answers nothing, or does not answer at all
- **THEN** the card keeps the pack's answer and stops saying a translation is coming

#### Scenario: The reader moved on
- **WHEN** a translation lands after the reader has closed the card or opened another
- **THEN** nothing is shown in its place

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

### Requirement: The setting's cost comes from the catalogue
The setting SHALL state the download it costs and the room the model takes on the device as the sums, over the files of the models it downloads, of their sizes as served and of their decompressed sizes, read from the catalogue. Without those sums, the setting SHALL state its cost without a size.

#### Scenario: Every reader today
- **WHEN** a reader opens « Traduction étendue » before ticking it
- **THEN** it says it downloads 25,8 Mo once, and a storage failure says the model takes 36,7 Mo

### Requirement: A translation is asked in its document's language
A translation request SHALL name the language of the document the sentence was read in. The background SHALL form the reader's pair from that language and the reader's native language, SHALL answer the request only when every model of that pair's route is on the device, and SHALL otherwise answer as without a model. The engine SHALL translate the sentence through that pair's route.

#### Scenario: An English page
- **WHEN** a reader of French with the en-fr model ready selects a phrase on an English page
- **THEN** the request names English, and the translation comes through en-fr's route

#### Scenario: A language whose models are not all there
- **WHEN** a reader selects a phrase on a page in a language whose pair's route has a model that is not on the device
- **THEN** the card answers as it does without a model

### Requirement: Spanish is translated through English
The catalogue SHALL carry Mozilla's es-en model, pinned like every model, and es-fr's route SHALL be es-en then en-fr. The engine SHALL load the models of a two-model route once, and SHALL translate a sentence through both in one request. For a reader of French whose accepted languages include Spanish, the setting SHALL state the download of both models.

#### Scenario: A reader of Spanish turns the setting on
- **WHEN** a reader of French who accepts Spanish opens « Traduction étendue » before ticking it
- **THEN** it says it downloads 52,0 Mo once, and ticking it stores both models

#### Scenario: A reader of English alone
- **WHEN** a reader of French who accepts English alone opens « Traduction étendue »
- **THEN** it says it downloads 25,8 Mo once, as before

#### Scenario: A Spanish selection
- **WHEN** a reader of French with both models selects a phrase on a Spanish page
- **THEN** the French sentence comes back from es-en then en-fr, in one request to the engine

#### Scenario: The host misses the es-en model
- **WHEN** the model host does not serve a file of the es-en model
- **THEN** the check run before a submission fails, naming that file

### Requirement: Translation models are listed in one catalogue, with a route per pair
The package SHALL carry one catalogue of the translation models it may download, each file pinned by its address, its size as served, its decompressed size and the sha256 of its decompressed bytes, and SHALL give for each pair the route of models that translates its studied language into its native language, in order. A route SHALL be keyed `<studied>-<native>`, SHALL name only models of the catalogue, SHALL start from the pair's studied language, SHALL chain each model's target to the next model's source, and SHALL end in the pair's native language; a catalogue that breaks any of these SHALL be refused, and nothing SHALL be fetched from it. The assembly of the model host, its mirror releases and the check run before a package is submitted SHALL cover every model of the catalogue.

#### Scenario: Every reader today
- **WHEN** the package is built
- **THEN** its catalogue holds the en-fr and es-en models, the en-fr route is the en-fr model alone, the es-fr route is es-en then en-fr, and the setting downloads, stores and loads them as before

#### Scenario: A route that does not reach French
- **WHEN** a catalogue's es-fr route ends in English
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A route that does not reach its native language
- **WHEN** a catalogue's en-es route ends in French
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A route that does not start from its studied language
- **WHEN** a catalogue's en-es route starts with a model that translates from Spanish
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A route whose key is no pair
- **WHEN** a catalogue keys a route `en` or `en-`
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A model stored before the catalogue
- **WHEN** the extension updates on a device that holds the en-fr model
- **THEN** the model is still complete, and nothing is downloaded again

#### Scenario: The host misses a model
- **WHEN** the model host does not serve a file of one of the catalogue's models
- **THEN** the check run before a submission fails, naming that model's file

### Requirement: The models follow the reader's pairs
While extended translation is on, the device SHALL keep the models that the routes of the reader's pairs need, and no other; the reader's pairs are the shipped pairs of their native language that study each of their accepted languages. Turning the setting on SHALL download every needed model. A model that the reader's pairs no longer need SHALL be deleted, and a model several routes share SHALL be kept while any of them needs it. A needed model that was never downloaded SHALL NOT be fetched until the reader asks, and the setting SHALL say what it costs. A model stored before this requirement SHALL stay complete.

#### Scenario: Every reader today
- **WHEN** a reader of English alone, whose native language is French, ticks « Traduction étendue »
- **THEN** the en-fr model is downloaded, as before, and nothing else

#### Scenario: A language added while the setting is on
- **WHEN** a reader with the setting on and the en-fr model ready adds a language whose pair's route needs a model not on the device
- **THEN** nothing is downloaded, and the setting offers to download the missing model with its size

#### Scenario: A language removed
- **WHEN** a reader of English and Spanish, whose native language is French, removes Spanish
- **THEN** the models only es-fr needed are deleted, and en-fr stays

#### Scenario: The native language changes
- **WHEN** a reader of English whose native language was French now has Spanish as their native language, and en-es ships with its route
- **THEN** the models en-es's route needs are the needed ones, the ones only en-fr needed are deleted, and nothing is fetched until the reader asks

#### Scenario: A model stored before the update
- **WHEN** the extension updates on a device that holds the en-fr model
- **THEN** it is still complete, and English is still translated

### Requirement: A pair's marks are measured before they are shown
A pair SHALL have its selection marked only once its marks, measured on the committed corpus of its studied language's selections with the extension's own marking through that pair's route, reach the programme's first tier: at least 90 % of the shown marks correct, and at most 25 % of the selections without a mark. The pairs that reached it SHALL be listed by name in the extension, and a pair outside the list SHALL be translated without a mark. The corpus, the rule that chose its selections, the harness and every judgment SHALL be committed, filed by pair, so that the measurement can be run again and checked.

#### Scenario: Measuring again
- **WHEN** the harness runs for a pair on the committed corpus of its studied language with the catalogue's models
- **THEN** it produces each selection's translated sentence and mark, as the extension would through that pair's route, and the judged results give that pair's share of correct and withheld marks

#### Scenario: The shipped pairs today
- **WHEN** a reader whose native language is French selects words on an English or a Spanish page
- **THEN** the selection is marked in the translated sentence, as en-fr's and es-fr's measurements allow

#### Scenario: Spanish below the first tier
- **WHEN** es-fr's measured marks fall short of 90 % correct or exceed 25 % withheld
- **THEN** its sentences stay translated without a mark

#### Scenario: Spanish on the first tier
- **WHEN** es-fr's measured marks reach 90 % correct with at most 25 % withheld
- **THEN** its selection is marked in the translated sentence, as en-fr's is

#### Scenario: A pair measured in another native language
- **WHEN** es-en is listed in the catalogue and its marks are not yet measured
- **THEN** es-en is translated without a mark, although es-fr's marks are measured

### Requirement: A translation goes through the reader's pair
The background SHALL form the pair of a translation from the language the page asked in — the document's — and the reader's native language, read from their stored profile, and SHALL ask the engine for that pair's route; a page SHALL keep asking in the document's language and SHALL NOT name a pair. A pair the catalogue lists no route for SHALL be unavailable, and the engine SHALL NOT be started for it. The ready pairs recorded on the device SHALL be read by pair where the background gates, and by studied language where a page gates: a page may ask in a language that a ready pair studies.

#### Scenario: Every reader today
- **WHEN** a reader whose native language is French selects words on an English page
- **THEN** the background asks for en-fr's route, and the sentence is translated as before

#### Scenario: The same page for another native language
- **WHEN** a reader whose native language is Spanish selects words on an English page, and en-es ships with its route
- **THEN** the background asks for en-es's route, and nothing of en-fr is loaded

#### Scenario: A pair without a route
- **WHEN** a reader whose native language is Spanish selects words on an English page, and no en-es ships
- **THEN** the answer is unavailable, and the engine is not started

#### Scenario: A state recorded before pairs
- **WHEN** the extension updates on a device whose stored state names the languages `en` and `es` as ready
- **THEN** en-fr and es-fr are ready, English and Spanish pages are translated as before, and the next reconciliation records the pairs

