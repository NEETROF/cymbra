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
A translation produced by the engine or by the online translation service SHALL be computed for display and discarded, and SHALL NOT be stored on a card, used as a card's gloss, or sent anywhere.
Only dictionary data SHALL be stored as a card's answer. This holds whatever surface asked for
the translation. The one exception to "sent anywhere" is the sentence the reader sends to the
online translation service by pressing its button; nothing else SHALL be sent, and no translation
SHALL be sent back to Cymbra once shown.

#### Scenario: A card made from a translated selection
- **WHEN** the reader creates a card from a selection the engine translated
- **THEN** the card holds its source sentence and its dictionary data, and no machine translation

#### Scenario: A card made after an online translation
- **WHEN** the reader creates a card from a selection after pressing « Traduction en ligne »
- **THEN** the card holds its source sentence and its dictionary data, and not the online translation

### Requirement: Without a model the extension behaves as it does today
When no model is ready, every surface SHALL answer exactly as it does without the engine.
No model is ready when extended translation is off, when its model is downloading, when the
download failed, or when the model was removed. The engine SHALL NOT be a condition for any
existing behaviour. On a reading surface its absence SHALL NOT be reported to the reader, as an
error or as a translation on its way; only the setting SHALL say that the model is not ready, and
why.

#### Scenario: No model available
- **WHEN** the reader selects a phrase and no model is present
- **THEN** the card answers from the pack exactly as it did before the engine existed, with no mention of a missing engine

#### Scenario: The model still downloading
- **WHEN** the reader selects a phrase while the model is downloading
- **THEN** the card answers from the pack, with no line saying a translation is on its way

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
translation; an expression's dictionary gloss SHALL remain. A selection of several words SHALL be
translated. A single word SHALL be translated only when the pack has no gloss for it and it is not
a proper noun outside the lexicon, and then in its sentence with the word marked, never alone; a
single word the pack glosses keeps its dictionary card.

#### Scenario: A translated phrase
- **WHEN** the engine translates a selection of several words
- **THEN** the card shows the sentence under a label saying it is a machine translation, with the selection's place marked

#### Scenario: Markup from the page
- **WHEN** the translated sentence contains characters that would read as markup
- **THEN** the card shows them as text, and no element is created from them

#### Scenario: A single word
- **WHEN** the reader selects a single word the pack has a gloss for
- **THEN** the engine is not asked, and the card is the word's dictionary card

#### Scenario: A single word the pack does not know
- **WHEN** the reader selects "disambiguation", which the pack has no gloss for, with a model ready
- **THEN** the card shows the sentence translated with the word's place marked, labelled as a machine translation

#### Scenario: A proper noun outside the lexicon
- **WHEN** the reader selects a single word the analysis classes as a proper noun outside the lexicon
- **THEN** the engine is not asked

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

### Requirement: The review translates a card the pack cannot answer
When a card without a gloss comes up in a review — an expression, or a single word the pack does not gloss — and this device has a model ready for the card's language, the review SHALL ask the engine for the card's sentence translated with the word's place marked, as soon as the card comes up, and SHALL show that translation with the answer once revealed, labelled as a machine translation (French UI copy: « Dans votre phrase — traduction automatique ») and rendered as text. An expression whose sentence does not hold it SHALL be translated alone, under a label saying it is a machine translation of the expression (French UI copy: « Traduction automatique »); a single word SHALL never be translated alone. A card with a gloss SHALL show its gloss alone, and the engine SHALL NOT be asked for it. The translation SHALL NOT show before the reveal, SHALL NOT hold the reveal back, and SHALL be discarded with the card, never stored.

#### Scenario: An expression in review
- **WHEN** the card of « such as those made », which has no gloss, comes up and a model is ready for English
- **THEN** once the answer is revealed, it shows the card's sentence translated with the expression's place marked, under the machine-translation label, and nothing of it shows before the reveal

#### Scenario: Ready at the reveal
- **WHEN** the reader reveals the answer after the translation has arrived
- **THEN** the translation is part of the answer from that moment, with nothing added afterwards

#### Scenario: Still on its way
- **WHEN** the reader reveals the answer before the translation has arrived
- **THEN** the answer says that a translation is coming, the translation replaces that line when it lands, and if none comes the answer keeps saying the card has no translation

#### Scenario: A word the pack glosses
- **WHEN** a card with a gloss comes up
- **THEN** the engine is not asked, and the answer is the gloss alone

#### Scenario: No model
- **WHEN** extended translation is off on this device, or the models of the card's language are not on it
- **THEN** the review is as it was before the engine existed, with no line about a translation

#### Scenario: A sentence that does not hold the expression
- **WHEN** an expression's card holds a sentence the expression is not in
- **THEN** the expression is translated alone, under the label of a machine translation of the expression

#### Scenario: Another card
- **WHEN** a translation lands after the reader has answered its card
- **THEN** it is shown nowhere

#### Scenario: Never stored
- **WHEN** the reader backs the deck up, or the deck synchronises, after such a review
- **THEN** no machine translation is in the backup or in what is sent

### Requirement: Online translation is chosen in the settings, off by default
The settings SHALL offer « Traduction en ligne » only to a signed-in reader whose account has access to online translation, as a choice made on each device, unticked until the reader ticks it.
Access SHALL be granted to an account by an operator, one account at a time, and SHALL be read from
the server, never decided by the extension alone. Its note SHALL say that the selected sentence is
sent to an external translation service to be translated and is kept by no one, and SHALL link to
the privacy policy. It SHALL NOT name the service. A reader without access, signed in or not,
SHALL NOT be offered the setting, and a ticked setting SHALL have no effect while the reader has no
access. The setting SHALL be independent of « Traduction étendue »: either may be ticked without
the other.

#### Scenario: A reader who was given access
- **WHEN** a reader whose account was given access opens the settings for the first time on a device
- **THEN** « Traduction en ligne » is shown unticked, with a note saying the sentence leaves the device and is kept by no one

#### Scenario: A signed-in reader without access
- **WHEN** a signed-in reader whose account was not given access opens the settings
- **THEN** « Traduction en ligne » is not shown

#### Scenario: A signed-out reader
- **WHEN** a reader who is not signed in opens the settings
- **THEN** « Traduction en ligne » is not shown

#### Scenario: Another device
- **WHEN** the reader ticks the setting on one device and opens the settings on another
- **THEN** it is unticked on the other device

### Requirement: The online translation button appears only when chosen
A card that shows a selection of several words SHALL show a « Traduction en ligne » button if and only if the setting is ticked and the reader has access.
The button SHALL NOT depend on the on-device engine: it SHALL be shown whether or not a model is
present, and whether or not the engine has answered. A single word SHALL keep its dictionary card,
without the button. No text SHALL leave the device until the reader presses the button.

#### Scenario: Setting ticked, a phrase selected
- **WHEN** a reader with access who ticked the setting selects several words
- **THEN** the card shows the pack's answer, then the engine's translation if any, and a « Traduction en ligne » button

#### Scenario: Setting not ticked
- **WHEN** a reader who did not tick the setting selects several words
- **THEN** the card has no « Traduction en ligne » button, and no request leaves the device

#### Scenario: No model on the device
- **WHEN** a reader who ticked the setting but never downloaded the engine's model selects several words
- **THEN** the card shows the pack's answer and the button

#### Scenario: A single word
- **WHEN** the reader selects a single word
- **THEN** the card is the word's dictionary card, without the button

### Requirement: An online translation is shown in place of the engine's, as a machine translation
When the reader presses the button, the card SHALL ask once for the sentence holding the selection and SHALL show the answer, when it lands, in place of any translation it shows, under a label saying it is an online machine translation, with the selection's place marked.
While the answer is on its way the card SHALL say so and SHALL keep what it shows. The card SHALL
render every part of the answer as text. The label SHALL NOT name a provider. A card the reader has
since replaced, closed or acted on SHALL NOT be written over. Pressing the button again on the same
card SHALL NOT send the sentence again.

#### Scenario: The answer lands
- **WHEN** the reader presses the button and the answer arrives while the card is still open
- **THEN** the card shows the online translation under its label, with the selection marked, in place of the engine's translation

#### Scenario: The reader moved on
- **WHEN** the answer arrives after the reader closed the card or opened another
- **THEN** nothing is shown in its place

#### Scenario: Markup in the answer
- **WHEN** the online translation contains characters that would read as markup
- **THEN** the card shows them as text, and no element is created from them

### Requirement: The selection is marked in the online translation
The service SHALL return the translated sentence with marks on the words that render the reader's selection, and SHALL accept a rendering split into several segments.
The service SHALL place a mark only on text that occurs in the translated sentence, in order. When
the words cannot be found in the sentence, the answer SHALL carry the sentence without marks rather
than a misplaced mark.

#### Scenario: A phrasal verb split in the translation
- **WHEN** the reader selected "putting off" and the translation renders it as « remettait … à plus tard »
- **THEN** both « remettait » and « à plus tard » are marked

#### Scenario: Words not found
- **WHEN** the words the provider names do not occur in its translation
- **THEN** the card shows the translation without a mark

### Requirement: An unavailable online translation costs the reader nothing
When the service answers unavailable, refuses the request, or does not answer within its bound, the card SHALL keep what it shows and SHALL say, in the reader's language, that online translation is unavailable for now.
The message SHALL NOT expose a technical error. A refused request because the daily limit is
reached SHALL say so and SHALL say when it resets.

#### Scenario: No provider answers
- **WHEN** the reader presses the button and every provider fails
- **THEN** the card keeps the engine's translation or the pack's answer, and says online translation is unavailable for now

#### Scenario: The daily limit is reached
- **WHEN** the reader presses the button after using up the day's online translations
- **THEN** the card says the daily limit is reached and when it resets, and no provider is asked

### Requirement: Online translation is limited per account
The service SHALL serve only a signed-in reader whose account has access, identified by their access token, and SHALL count each answered request against a per-account limit over a rolling day.
The limit SHALL be tunable at runtime without a redeploy. An operator SHALL be able to grant access
to an account, withdraw it from an account, and withdraw it from every account at once, all without
a release; access is withdrawn from everyone until granted. A request from an account without
access SHALL be answered as unavailable without asking any provider. The service SHALL refuse a
sentence longer than a fixed bound, and the extension SHALL NOT send one. The daily limit SHALL NOT
apply to Lingua staff (admins and moderators of Lingua), whose requests are still recorded in the
usage figures.

#### Scenario: A request without an account
- **WHEN** a request reaches the service without a valid access token
- **THEN** it is refused as unauthenticated and no provider is asked

#### Scenario: Access withdrawn from everyone
- **WHEN** the operator withdraws access from every account and a reader who had it presses the button on a card already open
- **THEN** the card says online translation is unavailable for now, no provider is asked, and the setting is no longer shown once the extension next reads its access

#### Scenario: An admin past the limit
- **WHEN** an admin of Lingua presses the button after more answered requests than the daily limit
- **THEN** the translation is shown, and the request is counted in the usage figures

#### Scenario: Access granted on request
- **WHEN** the operator grants access to one reader's account
- **THEN** that reader is offered the setting once the extension next reads its access, and no other reader is

### Requirement: Online translation goes straight from the document's language to the reader's
The card SHALL ask for a translation from the language of the selection's document into the reader's native language, and the service SHALL translate directly between them, without a pivot language.
The service SHALL accept a pair whose two languages are among Lingua's languages and differ, and
SHALL refuse any other pair. The button SHALL be offered on every pair the reader reads, including
a pair the on-device engine translates through English or does not mark.

#### Scenario: A French reader of a Spanish book
- **WHEN** a reader whose native language is French presses the button on a Spanish sentence
- **THEN** the sentence is translated from Spanish into French directly, with the selection marked

#### Scenario: An English reader of a French book
- **WHEN** a reader whose native language is English presses the button on a French sentence
- **THEN** the sentence is translated from French into English, and the card's copy is in English

### Requirement: The online translation service tries a second provider before giving up
The service SHALL ask a primary provider and, when it fails, times out or answers something it cannot read, a secondary provider, and SHALL answer unavailable only when both fail.
The reader SHALL see the same card whichever provider answered.

#### Scenario: The primary provider is down
- **WHEN** the primary provider returns an error and the secondary answers
- **THEN** the reader sees the secondary's translation, under the same label

### Requirement: The online translation service keeps nothing it is given
The service SHALL NOT write a sentence it receives, or a translation it returns, to a log, a store, a metric, a trace or an error report.
It MAY log, per request, the account, the time, the pair, the provider that answered, the
latency, and the token counts. What it stores for the usage figures SHALL be daily totals with no
account and no text. A provider SHALL be used only under terms that exclude training on
the request.

#### Scenario: A provider error
- **WHEN** a provider fails and the service logs the failure
- **THEN** the log holds the provider, the status and the latency, and no sentence

### Requirement: Extended translation is chosen per device, off by default
The reader SHALL turn extended translation on or off for each device separately, it SHALL be off until they do, and nothing but the reader SHALL change it.
The choice SHALL be stored on the device and SHALL NOT be synchronised. The stored value SHALL name
where the engine runs — today nowhere or on the device — so that another host can be added without
migrating it. The setting SHALL state, before it is turned on, how much it downloads and roughly how
much memory the engine uses while translating.

#### Scenario: A fresh install
- **WHEN** a reader installs the extension and never opens the setting
- **THEN** extended translation is off, no model is downloaded, and no engine runs

#### Scenario: Two devices on one account
- **WHEN** a signed-in reader turns extended translation on on the Mac, and the tablet then synchronises
- **THEN** extended translation is still off on the tablet, and nothing is downloaded there

### Requirement: The model is downloaded only when the reader turns extended translation on
The model SHALL be downloaded only after the reader turns extended translation on, only from Cymbra's static host, and SHALL be used only once every file matches its pinned hash.
The download SHALL show its progress, SHALL be cancellable, and SHALL report a failure in the
reader's language without a raw technical message. A file that does not match its hash SHALL be
discarded and never loaded. A cancelled download SHALL keep nothing and SHALL leave the setting
off. A failed one SHALL leave the setting on, offering to try again, with every surface answering
as it does without a model.

#### Scenario: Turning it on
- **WHEN** the reader turns extended translation on
- **THEN** the model is downloaded with its progress shown, verified, and translation becomes available once it is

#### Scenario: A file that is not the model
- **WHEN** a downloaded file does not match its pinned hash
- **THEN** it is discarded, the engine is not started, and the setting says the download failed and offers to try again

#### Scenario: Cancelling
- **WHEN** the reader cancels a download in progress
- **THEN** no part of the model is kept and extended translation is off

### Requirement: Turning extended translation off deletes the model
Turning extended translation off SHALL stop the engine and delete every stored file of the model from the device.

#### Scenario: Turning it off
- **WHEN** the reader turns extended translation off after the model was downloaded
- **THEN** the engine stops, the model's files are gone from the device's storage, and turning it on again downloads them again

### Requirement: The engine is part of the extension, and only its model is fetched
The engine's code SHALL be packaged inside the extension, and the extension SHALL NOT download, load or execute code from anywhere else.
Only the model — data — SHALL be fetched. A package that offers extended translation SHALL carry
the engine built from the pinned source.

#### Scenario: Inspecting a shipped package
- **WHEN** a shipped Chromium or Firefox package is inspected
- **THEN** it contains the engine, and no request the extension makes fetches JavaScript or WebAssembly

### Requirement: A model the browser removed is not fetched again unasked
When extended translation is on and its model is no longer on the device, the extension SHALL NOT download it again without the reader asking, and the setting SHALL say that the model must be downloaded again.
Until it is, every surface SHALL answer as it does without a model.

#### Scenario: Storage cleared by the browser
- **WHEN** the browser has removed the extension's stored model while extended translation is on
- **THEN** nothing is downloaded, cards answer from the pack, and the setting offers to download the model again

### Requirement: The engine is loaded only while it is used
The engine SHALL be loaded when a translation is asked, when a reader begins a selection while a model is ready, or when a page that asked for a translation within the idle period becomes visible again; it SHALL stay loaded while reading tabs keep asking for translations, and SHALL be released after an idle period in which none is asked.
Beginning a selection and returning to such a page SHALL count as asking, for the idle period
only: they load the engine early and arm its release, and they translate nothing. Nothing SHALL be
loaded for them when no model is ready. Turning extended translation on SHALL NOT by itself load
the engine, and neither SHALL opening a page. A keep-warm signal from a reading tab SHALL NOT count
as a translation asked.

#### Scenario: Reading stops
- **WHEN** no translation has been asked for the idle period, even with reading tabs still open
- **THEN** the engine is released, and the next translation loads it again

#### Scenario: The setting just turned on
- **WHEN** the model has finished downloading and the reader has asked for nothing yet
- **THEN** the engine is not loaded

#### Scenario: A selection begins
- **WHEN** a model is ready and the reader starts selecting text, before the selection settles
- **THEN** the engine starts loading, and the translation asked when the selection settles uses it

#### Scenario: A selection that needs no translation
- **WHEN** the reader selects a single word the pack glosses, after the engine was loaded for that selection
- **THEN** nothing is translated, and the engine is released once the idle period passes with no translation asked

#### Scenario: Back to a page that was translating
- **WHEN** a page that asked for a translation less than the idle period ago becomes visible again and the engine is no longer loaded
- **THEN** the engine starts loading before the reader selects anything

#### Scenario: A page opened, nothing selected
- **WHEN** the reader opens or returns to a page that has asked for no translation within the idle period
- **THEN** the engine is not loaded

#### Scenario: No model
- **WHEN** extended translation is off, or its model is not ready, and the reader begins a selection
- **THEN** nothing is loaded and nothing is sent to the engine's host

### Requirement: Extended translation is offered where it has been measured
The setting SHALL be offered on Chromium, on Firefox desktop, on Firefox for Android and on Safari — iPhone, iPad and Mac.
Everywhere it is offered, the setting, its download, its deletion and its states SHALL be those of
Firefox desktop, and the engine SHALL run in the extension's own package: no variant downloads code,
only the model. A platform SHALL join this list on a measurement on its devices, recorded in the
change that adds it.

#### Scenario: Firefox for Android
- **WHEN** a reader opens the settings in Firefox for Android
- **THEN** the extended translation setting is there, off, stating the download size and the memory used, exactly as on Firefox desktop

#### Scenario: Safari on iPhone, iPad and Mac
- **WHEN** a reader opens the extension's settings in Safari on an iPhone, an iPad or a Mac
- **THEN** the extended translation setting is there, off, stating the download size and the memory used, exactly as on Firefox desktop

#### Scenario: The Safari package
- **WHEN** the Safari package is inspected
- **THEN** it carries the pinned engine and the model's manifest, and no model file

### Requirement: A selection already translated on the page is not translated again
A page SHALL keep the translations it has received, and SHALL answer a request identical to one already answered — the same sentence and the same selection within it — from them, without asking the engine.
An identical request made while the first is still being answered SHALL wait for that answer
instead of asking again. Only translations SHALL be kept, not the absence of one. The kept answers SHALL belong to the page:
they SHALL NOT be stored, SHALL NOT be shared with another tab, and SHALL go when the page goes. A
different selection in the same sentence SHALL be asked for, since the engine marks the selection
in its answer.

#### Scenario: Handles back on a span already answered
- **WHEN** the reader adjusts the selection handles and comes back to a selection translated a moment ago on the same page
- **THEN** the card shows that translation without the engine running again

#### Scenario: The same request twice while the engine loads
- **WHEN** the same sentence and selection are asked for a second time before the first answer arrives
- **THEN** the engine is asked once, and both get its answer

#### Scenario: A different span of the same sentence
- **WHEN** the reader selects other words of a sentence already translated on the page
- **THEN** the engine is asked, and the card marks the new selection

#### Scenario: No translation last time
- **WHEN** a request was answered without a translation (no model ready, the engine did not answer)
- **THEN** the same request asks again next time

