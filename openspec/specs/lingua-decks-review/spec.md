# lingua-decks-review Specification

## Purpose
TBD - created by archiving change add-lingua-decks-review. Update Purpose after archive.
## Requirements
### Requirement: Card schema with provenance
A card SHALL carry at minimum: the lemma, the encountered form, the originating context
sentence, the source of the encounter (URL or agent session identifier, timestamp), the
gloss, and an optional media slot (`media`, with `source: capture|stock|generated` and
`sync_policy`) — not populated in this change but present in the schema. Multi-word
expressions SHALL be cards in their own right.

#### Scenario: Card created from the popup
- **WHEN** the user adds `conundrum` to the deck from a web page
- **THEN** the card holds the lemma, the complete originating sentence and the page URL

#### Scenario: Expression card
- **WHEN** the user captures the selection "compelling starting point" with the keyboard shortcut
- **THEN** an expression card is created with the originating sentence

### Requirement: FSRS review scheduling
Review SHALL be scheduled by FSRS in the core: each card carries its own state
(stability, difficulty, due date), and the four answers (`again`, `hard`, `good`, `easy`)
SHALL update that state and the due date. The number of due cards SHALL be computable at
any moment.

#### Scenario: A "good" answer pushes the due date out
- **WHEN** a due card is graded `good`
- **THEN** its due date becomes strictly later than now and its FSRS state is updated

### Requirement: Moving to known from review
Marking a card "I know this" during review SHALL move the lemma to the `known` status
(provenance `srs`) and remove it from the due cards, without deleting the card or its
history.

#### Scenario: Word learned
- **WHEN** the user answers "I know this" on the `seldom` card
- **THEN** `seldom` moves to the `known` status (provenance `srs`) and the card leaves the review queue

### Requirement: Lossless backup and restore
The system SHALL export the complete state (cards with all their fields — lemma,
encountered form, sentence, gloss, source, status, due date, FSRS state — word statuses,
calibration, FSRS parameters) to a versioned backup file, and SHALL restore that file
identically. No populated field SHALL be omitted from the backup; a restore onto a clean
state SHALL reproduce the original state (a lossless round trip). The card schema SHALL
stay entirely serialisable field by field, so that an Anki-format export (deferred to a
later change) remains a plain serialiser.

#### Scenario: Backup round trip
- **WHEN** the user backs up a state of 20 cards and 300 statuses, then restores it onto a fresh install
- **THEN** the restored state is identical to the original — cards, statuses, calibration and review due dates included

### Requirement: Review present right next to the reading
Review SHALL be reachable without leaving the browser: the due-card count SHALL be visible in the icon popup and in the side panel, and a review session SHALL be launchable from the side panel or the injected panel. The front of a card SHALL show the sentence in which its word was met, with the word marked, when the card has one; a card without a sentence SHALL show its word alone. The card's answer — its gloss — SHALL stay hidden until an explicit reveal action.

#### Scenario: Micro-session from the side panel
- **WHEN** the user opens the side panel and starts a review with 3 cards due (French UI copy: « Réviser maintenant »)
- **THEN** the cards come one by one, each showing its sentence with the gloss hidden, then revealed, and the due count goes down

#### Scenario: A word met in a book
- **WHEN** a card captured in a book comes up
- **THEN** its front shows the book's sentence with the word marked, and its gloss appears only once the reader reveals the answer

#### Scenario: A card without a sentence
- **WHEN** a card seeded from a level, which has no sentence, comes up
- **THEN** its front shows the word alone, and its gloss appears only once the reader reveals the answer

### Requirement: Level-targeted deck seeding
The deck SHALL support seeding cards for a chosen set of lemmas — typically the lemmas of a
selected CEFR level (or frequency band) — without a real web encounter. Seeded cards SHALL
record the reserved `import` encounter source rather than fabricating a URL or an agent
session. Seeding SHALL be capped per operation, SHALL skip lemmas that already have a card
or an explicit status (idempotent), and SHALL let the caller choose the order in which
lemmas are taken from the level (commonest-first by default).

#### Scenario: Seed a level into the deck
- **WHEN** the user asks to add 20 words of level B2 with 5 of them already tracked
- **THEN** at most 15 new cards are created, each with encounter source `import`, and no already-tracked lemma is duplicated

#### Scenario: Cap respected
- **WHEN** the user requests more words than the per-operation cap allows
- **THEN** only up to the cap are seeded and the caller is told how many were added

### Requirement: The backup records the reader's language profile
The backup SHALL record the reader's language profile: their studied languages, an ordered list, non-empty and without duplicates, whose first language is the primary one, and their native language, which is never one of the studied languages. The default profile is English studied, French native. A backup whose profile is the default and whose every per-language record is English SHALL be written with schema version 1 and the exact content it had before the profile was stored. Any other backup SHALL be written with schema version 2, the profile included. A choice of studied languages that holds the native language SHALL be refused, and the native language SHALL be set only together with the studied languages, the whole choice being refused when it would study the native language; a refused choice SHALL leave the profile as it was. A full reset SHALL give the reader the engine's native language, studying the language of the engine's first pack alone; resetting statuses SHALL leave the profile unchanged. The profile SHALL never be sent to the server.

#### Scenario: An English reader's backup does not change
- **WHEN** an English reader with 20 cards and 300 statuses is backed up
- **THEN** the backup has schema version 1, carries no profile, and is byte for byte what the previous build wrote

#### Scenario: Another studied language
- **WHEN** the studied languages are set to Spanish then English, and the state is backed up and restored
- **THEN** the backup has schema version 2, and the restored studied languages are Spanish then English

#### Scenario: Another language in the records only
- **WHEN** the profile is the default but the deck holds a Spanish card
- **THEN** the backup has schema version 2

#### Scenario: Another native language
- **WHEN** the profile is set to Spanish studied with English native, and the state is backed up and restored
- **THEN** the backup has schema version 2, and the restored profile studies Spanish with English native

#### Scenario: A backup written before the native language could change
- **WHEN** a version 2 backup written by the previous build, whose profile studies Spanish then English, is restored
- **THEN** its native language is French and its studied languages are Spanish then English

#### Scenario: Resets
- **WHEN** a reader who studies English and Spanish erases their data, on an engine whose first pack is en-fr
- **THEN** the profile is the default again, whereas resetting statuses leaves it unchanged

#### Scenario: A full reset on an engine glossed in English
- **WHEN** a reader erases their data on an engine whose first pack studies Spanish glossed in English
- **THEN** their native language is English and they study Spanish alone

#### Scenario: An invalid choice of studied languages
- **WHEN** studied languages are set as an empty list, as a list naming a language twice, or as a list holding the native language
- **THEN** they are refused and the previous studied languages stay

#### Scenario: A native language the reader would study
- **WHEN** the profile is set to English native with English among the studied languages
- **THEN** it is refused and the previous profile stays

#### Scenario: Never synced
- **WHEN** a sync pushes and pulls
- **THEN** no request or response carries the profile

### Requirement: A restore reads the backup's schema version first
A restore SHALL read the backup's schema version before the rest of the file, SHALL read schema versions 1 and 2, and SHALL refuse any other version as unsupported, naming the version found. When a restore is refused, the extension SHALL not start the surface that asked for it and SHALL leave the stored backup unchanged. A build released before schema version 2 existed refuses a version 2 backup: as malformed when the backup holds another language's records, as unsupported otherwise.

#### Scenario: Both known versions
- **WHEN** a version 1 backup and a version 2 backup are restored
- **THEN** both are read

#### Scenario: A later version
- **WHEN** a backup of schema version 3 is restored
- **THEN** the restore fails as unsupported version 3, and the stored backup is unchanged

#### Scenario: A build released before version 2
- **WHEN** a build released before this change opens a store whose backup has schema version 2 and Spanish cards
- **THEN** its restore fails as malformed, its reader does not start, and the backup is still there for a later build

### Requirement: A review is in the language being read
A review session and its counts SHALL cover one studied language at a time, never several mixed. The review SHALL open in the language of the page or book it is opened beside, when the reader studies that language. Otherwise it SHALL open in the last language the reader chose in the review on this device, and otherwise in the reader's first studied language. A language the reader chooses in the review SHALL be remembered on the device as the last one chosen, and SHALL hold until the review is opened again or the page or book beside it changes; the review SHALL then open in that page's language again, and an open review SHALL follow the page or book beside it. The review's count of cards and of cards due SHALL be that language's, and the popup's count of cards to review SHALL follow the same language.

#### Scenario: Beside a Spanish page
- **WHEN** a reader of English and Spanish opens the review beside a page in Spanish
- **THEN** the review and its counts hold the Spanish cards only, and no choice of all languages is offered

#### Scenario: A book that opens on its cover
- **WHEN** the reader opens a Spanish book on its cover, or on front matter in English, beside the review
- **THEN** the review is in Spanish

#### Scenario: Away from a page
- **WHEN** the review opens beside no page in a studied language, and the reader last chose English in the review
- **THEN** it opens in English

#### Scenario: Choosing another language
- **WHEN** the reader chooses English in the review beside a Spanish page
- **THEN** the review and its counts switch to English, and English is remembered as the last language chosen

#### Scenario: Back from the statistics
- **WHEN** the reader chose English in the review beside a Spanish page, then shows the statistics and the review again
- **THEN** the review is still in English

#### Scenario: Another page or book
- **WHEN** the reader chose English in the review beside a Spanish page, then moves to another page or book in Spanish with the review open, or opens the review again
- **THEN** the review is in Spanish

#### Scenario: Every reader today
- **WHEN** a reader studies English alone
- **THEN** no choice of language is shown, and the review and its counts are as before

### Requirement: A card is keyed by its studied language
A card SHALL belong to exactly one studied language, and a lemma SHALL be able to have one card per studied language: the deck, its counts and its synchronisation SHALL treat a card of one language and a card of another as distinct even when their lemmas are spelled the same.

#### Scenario: One spelling, two cards
- **WHEN** a reader studying English and Spanish holds a card for `pie` in each language
- **THEN** the two cards are scheduled, reviewed and synchronised independently, each with its own gloss and sentence

### Requirement: One review queue across languages, with a language filter
A review session SHALL take the due cards of the one studied language the review is in, as *A review is in the language being read* chooses it, ordered as *A session is short and starts with the most fragile cards* orders them. When the reader studies several languages, the review page SHALL offer each of them, one at a time, and its counts SHALL follow the choice.

#### Scenario: A mixed queue
- **WHEN** a reader of English and Spanish starts a review in Spanish with cards due in both languages
- **THEN** the session holds only Spanish cards, the most fragile first, never the two languages mixed

#### Scenario: Spanish only
- **WHEN** the same reader chooses Spanish in the review page
- **THEN** the counts and the next session hold only the Spanish cards, ordered the same way

#### Scenario: Every reader today
- **WHEN** a reader studies English alone
- **THEN** no filter is shown, and the session follows the same order

### Requirement: A review card says its language
The review card SHALL carry its studied language, and the review page SHALL show it when the reader studies several languages. A review or a word learned from a card SHALL be counted in the card's language.

#### Scenario: A Spanish card in a mixed queue
- **WHEN** a Spanish card comes up in a mixed review
- **THEN** it is marked as Spanish, and grading it counts a Spanish review

### Requirement: A card says the language of its gloss
A card SHALL carry the language its gloss is written in, `fr` by default; a card created on an engine — added from a page, or seeded from the pack — SHALL take the engine's native language; the local export of card operations SHALL emit it when it is not `fr`, and the apply SHALL read it, an absent or empty value meaning `fr`, so that the exported operations of a French-glossed deck keep their bytes. The backup SHALL leave the label out when it is `fr`, so that a backup written before this requirement, and every backup whose glosses are French, keep their bytes; a card labelled other than `fr` is another language in the records, so a backup holding one SHALL be written with schema version 2 and a version 1 backup SHALL never carry a label; the newest schema version SHALL NOT change for it, and a build released before this requirement SHALL read a backup that carries it.

#### Scenario: Every card today
- **WHEN** a reader of French with 20 French-glossed cards is backed up
- **THEN** the backup is byte for byte what the previous build wrote, and every card restores with the gloss language `fr`

#### Scenario: A card created on an engine glossed in English
- **WHEN** a card is created on an engine whose packs are glossed in English
- **THEN** its gloss language is `en`, the backup carries it, and the previous build still reads the backup

#### Scenario: A card applied from a card operation
- **WHEN** a card operation with the gloss language `en` is applied, and another with none
- **THEN** the first card is labelled `en` and the second `fr`, and the export emits `en` on the first and no label on the second

### Requirement: Review shows a gloss the reader can read
When the gloss of the card under review is in a language other than the engine's native language, review SHALL show the gloss of the pack held for the card's studied language — a word's gloss, or an expression's from the pack's expression table — and SHALL show the card's own text when that pack has none or none is held. A card whose gloss is in the engine's native language SHALL be shown as before. The card's stored gloss and label SHALL NOT be rewritten by review, and the substitution SHALL NOT add or rename a key of the view.

#### Scenario: The English baseline
- **WHEN** the English baseline runs
- **THEN** every review view is byte for byte as pinned

#### Scenario: A word glossed in another native language
- **WHEN** a French-native engine reviews a Spanish word card whose gloss is in English
- **THEN** the view's gloss is es-fr's gloss for that lemma, and the card keeps its English text and label

#### Scenario: An expression glossed in another native language
- **WHEN** a French-native engine reviews a Spanish expression card whose gloss is in English
- **THEN** the view's gloss is es-fr's expression gloss for that key

#### Scenario: A gloss the pack has not
- **WHEN** the card's lemma is not glossed by the current pack
- **THEN** the view shows the card's own text, in its language

### Requirement: A session is short and starts with the most fragile cards
A review session SHALL hold at most 10 cards and SHALL start with the due cards whose predicted recall is lowest. Never-reviewed cards SHALL be interleaved, one after every three reviewed cards, within the day's allowance of new words; when no reviewed card is due, never-reviewed cards SHALL fill the session up to that allowance. Continuing (French UI copy: « Encore 10 ») SHALL start a new session over the cards still due.

#### Scenario: Forty cards due
- **WHEN** a reader with 40 reviewed cards due starts a review
- **THEN** the session holds 10 cards, the one whose predicted recall is lowest first

#### Scenario: Continuing
- **WHEN** the reader finishes those 10 cards and chooses to continue (French UI copy: « Encore 10 »)
- **THEN** a new session holds the next 10 cards still due, ordered the same way

#### Scenario: New words among reviews
- **WHEN** 12 reviewed cards and 5 never-reviewed cards are due, and the day's allowance is not spent
- **THEN** the session holds three reviewed cards, one new card, three reviewed cards, one new card, then two reviewed cards

### Requirement: New words enter review at a daily pace
The review SHALL bring at most a daily allowance of never-reviewed cards of each studied language into review per local day — 10 by default, 5 or 20 when the reader chooses so in Réglages (French UI copy: « Nouveaux mots par jour ») — and SHALL leave the others due for the following days. A never-reviewed card SHALL count against its own language's allowance, on the day on which it is first answered. Capturing a word SHALL stay unlimited and SHALL NOT be affected by the allowance.

#### Scenario: A heavy reading day
- **WHEN** a reader captures 40 words in one day and reviews several times that day
- **THEN** 10 of those words enter review that day and the other 30 wait

#### Scenario: The next day
- **WHEN** the same reader reviews the next day
- **THEN** up to 10 of the waiting words enter review

#### Scenario: A larger allowance
- **WHEN** the reader sets the daily allowance of new words to 20
- **THEN** up to 20 never-reviewed cards enter review per day

#### Scenario: Two languages
- **WHEN** a reader of English and Spanish lets 10 new English words into review in the morning, then reviews in Spanish that evening
- **THEN** up to 10 new Spanish words still enter review that day

### Requirement: A missed card comes back in the same session
A card answered as missed SHALL come back in the same session after three other cards, or at the end when fewer remain, until it is recalled or has been asked three times in that session; when no other card remains to be shown, a missed card SHALL NOT come back, since it would be shown again at once. Only a card's first answer in a session SHALL update its FSRS state and count as a review; a later answer in the same session SHALL only keep the card in the session or take it out. A card shown again in a session SHALL say so above its sentence, in place of the count of cards remaining — in a practice session too, in place of its practice note — with the time it is being asked (French UI copy: « À revoir · 2ᵉ passage », « À revoir · 3ᵉ passage »); its first showing SHALL say nothing of the kind.

#### Scenario: A word recovered
- **WHEN** the reader misses `seldom`, then recalls it when it comes back
- **THEN** `seldom` is graded `again` once, leaves the session, and counts as recovered

#### Scenario: Missed three times
- **WHEN** the reader misses the same card three times in one session
- **THEN** the card leaves the session and waits for its next due date

#### Scenario: One review counted
- **WHEN** a missed card comes back and is answered again
- **THEN** the day's count of reviews holds one review for that card, not two

#### Scenario: A card that comes back says why
- **WHEN** the reader misses `seldom` and it comes back three cards later
- **THEN** the card shows « À revoir · 2ᵉ passage » above its sentence instead of the count of cards remaining

#### Scenario: Its third showing
- **WHEN** the reader misses `seldom` again when it comes back
- **THEN** its next showing says « À revoir · 3ᵉ passage »

#### Scenario: A first showing
- **WHEN** a card is shown for the first time in a session
- **THEN** it shows the count of cards remaining, as before

#### Scenario: The last card missed
- **WHEN** the reader misses the last card of a session, no other card remaining
- **THEN** the session ends: the card is graded `again` once and is not shown again in a row

#### Scenario: In practice
- **WHEN** a practice card the reader missed comes back
- **THEN** it says « À revoir · 2ᵉ passage » in place of its practice note

#### Scenario: In every host
- **WHEN** a missed card comes back in the side panel, in the in-page drawer or in the Cymbra Lingua app
- **THEN** each says it the same way, in the reader's interface language

### Requirement: Two answers on the card
The review SHALL offer two answers on a revealed card — a miss (French UI copy: « Pas su »), graded `again`, and a recall (French UI copy: « Su »), graded `good` — together with marking the word known (French UI copy: « Je connais ») and hiding it from review (French UI copy: « Ne plus me le montrer »). The core SHALL keep accepting the four FSRS ratings from its other callers.

#### Scenario: Remembered
- **WHEN** the reader reveals a card and answers that they recalled it
- **THEN** the card is graded `good` and the next card comes

#### Scenario: Forgotten
- **WHEN** the reader reveals a card and answers that they missed it
- **THEN** the card is graded `again` and will come back in the session

#### Scenario: Another caller
- **WHEN** a caller other than the review page grades a card `hard` or `easy`
- **THEN** the core applies that rating as it always has

### Requirement: Hiding a word from review
Hiding a word from review (French UI copy: « Ne plus me le montrer ») SHALL mark the word `ignored`, stamped for synchronisation, and SHALL retire its card from review without deleting the card or its history. The word SHALL be put back to learn the way any ignored word is.

#### Scenario: A name the reader does not want to learn
- **WHEN** the reader hides `Netherfield` from review
- **THEN** `Netherfield` becomes `ignored`, its card leaves the review queue, and the card is kept

#### Scenario: Another device
- **WHEN** that reader's other device synchronises
- **THEN** the word is ignored there too and its card no longer comes due

#### Scenario: Changing one's mind
- **WHEN** the reader later puts that word back to learn (French UI copy: « Remettre à apprendre »)
- **THEN** the word is highlighted again, as any ignored word put back to learn is

### Requirement: A session ends with what it did
The end of a session SHALL say how many cards were reviewed, how many missed cards were recovered, and how many cards are now scheduled more than a month away, and SHALL offer to continue (French UI copy: « Encore 10 ») only when cards remain due.

#### Scenario: End of a session
- **WHEN** a reader finishes a session of 10 cards in which 2 missed cards were recovered
- **THEN** the end says 10 reviewed and 2 recovered, and how many cards now hold for more than a month

#### Scenario: Nothing left to review
- **WHEN** a session ends and no card remains due
- **THEN** the end does not offer to continue

### Requirement: Practice when nothing is due
When no card is due in the language under review, the review SHALL offer a practice session (French UI copy: « S'entraîner quand même »), both where it says nothing is due and at the end of a session that leaves nothing due, provided the reader has at least one reviewed card that is not retired in that language. A practice session SHALL hold at most 10 reviewed, non-retired cards, the lowest predicted recall first, whether or not they are due, and SHALL include no never-reviewed card. Its answers SHALL leave every card's schedule unchanged — due date, memory and history — while a missed card SHALL still come back in the session as in a review; marking a word known or hiding it SHALL apply as in a review. Each card SHALL say it is practice in place of the count of cards remaining (French UI copy: « Entraînement · N carte(s) »), and each card's first answer SHALL count as a review in the day's statistics.

#### Scenario: Nothing due
- **WHEN** a reader with 10 reviewed English cards, none due, opens the review in English
- **THEN** the review says nothing is due and offers to practise anyway

#### Scenario: A practice session
- **WHEN** the reader chooses to practise
- **THEN** a session holds those cards, the one they are most likely to have forgotten first, each saying it is practice

#### Scenario: The schedule stays as it was
- **WHEN** the reader answers a practice card as recalled, or as missed
- **THEN** the card's next due date, memory and review history are what they were before the session

#### Scenario: Counted in the day
- **WHEN** the reader answers five practice cards
- **THEN** the day's count of reviews grows by five

#### Scenario: Nothing to practise
- **WHEN** the reader has no reviewed card in that language, only never-reviewed or retired ones
- **THEN** the review offers no practice

#### Scenario: Cards due
- **WHEN** cards are due in the language under review
- **THEN** the review offers the review of those cards, not practice

