## MODIFIED Requirements

### Requirement: Review present right next to the reading
Review SHALL be reachable without leaving the browser: the due-card count SHALL be visible in the icon popup and in the side panel, and a review session SHALL be launchable from the side panel or the injected panel. The front of a card SHALL show the sentence in which its word was met, with the word marked, when the card has one; a card without a sentence SHALL show its word alone. The card's answer — its gloss — SHALL stay hidden until an explicit reveal action.

#### Scenario: Micro-session from the side panel
- **WHEN** the user opens the side panel and starts a review with 3 cards due (French UI copy: « Réviser maintenant »)
- **THEN** the cards come one by one, each showing its sentence with the gloss hidden, then revealed, and the due count goes down

#### Scenario: A word met in a book
- **WHEN** a card captured in a book comes up
- **THEN** its front shows the book's sentence with the word marked, and its gloss appears only after « Afficher la réponse »

#### Scenario: A card without a sentence
- **WHEN** a card seeded from a level, which has no sentence, comes up
- **THEN** its front shows the word alone, and its gloss appears only after « Afficher la réponse »

### Requirement: One review queue across languages, with a language filter
A review session SHALL take the due cards of the one studied language the review is in, as *A review is in the language being read* chooses it, ordered as *A session is short and starts with the most fragile cards* orders them. When the reader studies several languages, the review page SHALL offer each of them, one at a time, and its counts SHALL follow the choice.

#### Scenario: A reader of two languages
- **WHEN** a reader of English and Spanish starts a review in Spanish with cards due in both languages
- **THEN** the session holds only Spanish cards, the most fragile first

#### Scenario: One language chosen
- **WHEN** the same reader chooses English in the review page
- **THEN** the counts and the next session hold only English cards, ordered the same way

#### Scenario: A reader of one language
- **WHEN** a reader studies one language
- **THEN** no filter is shown, and the session follows the same order

## ADDED Requirements

### Requirement: A session is short and starts with the most fragile cards
A review session SHALL hold at most 10 cards and SHALL start with the due cards whose predicted recall is lowest. Never-reviewed cards SHALL be interleaved, one after every three reviewed cards, within the day's allowance of new words; when no reviewed card is due, never-reviewed cards SHALL fill the session up to that allowance. Continuing (French UI copy: « Encore 10 ») SHALL start a new session over the cards still due.

#### Scenario: Forty cards due
- **WHEN** a reader with 40 reviewed cards due starts a review
- **THEN** the session holds 10 cards, the one whose predicted recall is lowest first

#### Scenario: Continuing
- **WHEN** the reader finishes those 10 cards and chooses « Encore 10 »
- **THEN** a new session holds the next 10 cards still due, ordered the same way

#### Scenario: New words among reviews
- **WHEN** 12 reviewed cards and 5 never-reviewed cards are due, and the day's allowance is not spent
- **THEN** the session holds three reviewed cards, one new card, three reviewed cards, one new card, then two reviewed cards

### Requirement: New words enter review at a daily pace
The review SHALL bring at most a daily allowance of never-reviewed cards of each studied language into review per local day — 10 by default, 5 or 20 when the reader chooses so in Réglages (« Nouveaux mots par jour ») — and SHALL leave the others due for the following days. A never-reviewed card SHALL count against its own language's allowance, on the day on which it is first answered. Capturing a word SHALL stay unlimited and SHALL NOT be affected by the allowance.

#### Scenario: A heavy reading day
- **WHEN** a reader captures 40 words in one day and reviews several times that day
- **THEN** 10 of those words enter review that day and the other 30 wait

#### Scenario: The next day
- **WHEN** the same reader reviews the next day
- **THEN** up to 10 of the waiting words enter review

#### Scenario: A larger allowance
- **WHEN** the reader sets « Nouveaux mots par jour » to 20
- **THEN** up to 20 never-reviewed cards enter review per day

#### Scenario: Two languages
- **WHEN** a reader of English and Spanish lets 10 new English words into review in the morning, then reviews in Spanish that evening
- **THEN** up to 10 new Spanish words still enter review that day

### Requirement: A missed card comes back in the same session
A card answered « Pas su » SHALL come back in the same session after three other cards, or at the end when fewer remain, until it is answered « Su » or has been asked three times in that session. Only a card's first answer in a session SHALL update its FSRS state and count as a review; a later answer in the same session SHALL only keep the card in the session or take it out.

#### Scenario: A word recovered
- **WHEN** the reader answers « Pas su » on `seldom`, then « Su » when it comes back
- **THEN** `seldom` is graded `again` once, leaves the session, and counts as recovered

#### Scenario: Missed three times
- **WHEN** the reader answers « Pas su » on the same card three times in one session
- **THEN** the card leaves the session and waits for its next due date

#### Scenario: One review counted
- **WHEN** a missed card comes back and is answered again
- **THEN** the day's count of reviews holds one review for that card, not two

### Requirement: Two answers on the card
The review SHALL offer two answers on a revealed card — « Pas su », graded `again`, and « Su », graded `good` — together with « Je connais » and « Ne plus me le montrer ». The core SHALL keep accepting the four FSRS ratings from its other callers.

#### Scenario: Remembered
- **WHEN** the reader reveals a card and answers « Su »
- **THEN** the card is graded `good` and the next card comes

#### Scenario: Forgotten
- **WHEN** the reader reveals a card and answers « Pas su »
- **THEN** the card is graded `again` and will come back in the session

#### Scenario: Another caller
- **WHEN** a caller other than the review page grades a card `hard` or `easy`
- **THEN** the core applies that rating as it always has

### Requirement: Hiding a word from review
Answering « Ne plus me le montrer » SHALL mark the word `ignored`, stamped for synchronisation, and SHALL retire its card from review without deleting the card or its history. The word SHALL be put back to learn the way any ignored word is.

#### Scenario: A name the reader does not want to learn
- **WHEN** the reader answers « Ne plus me le montrer » on `Netherfield`
- **THEN** `Netherfield` becomes `ignored`, its card leaves the review queue, and the card is kept

#### Scenario: Another device
- **WHEN** that reader's other device synchronises
- **THEN** the word is ignored there too and its card no longer comes due

#### Scenario: Changing one's mind
- **WHEN** the reader later chooses « Remettre à apprendre » on that word
- **THEN** the word is highlighted again, as any ignored word put back to learn is

### Requirement: A session ends with what it did
The end of a session SHALL say how many cards were reviewed, how many missed cards were recovered, and how many cards are now scheduled more than a month away, and SHALL offer « Encore 10 » only when cards remain due.

#### Scenario: End of a session
- **WHEN** a reader finishes a session of 10 cards in which 2 missed cards were recovered
- **THEN** the end says 10 reviewed and 2 recovered, and how many cards now hold for more than a month

#### Scenario: Nothing left to review
- **WHEN** a session ends and no card remains due
- **THEN** the end does not offer « Encore 10 »
