## MODIFIED Requirements

### Requirement: Review present right next to the reading
Review SHALL be reachable without leaving the browser: the count of cards to review SHALL be visible in the icon popup on every tab, whether its page was analysed or not, and in the side panel, and a review session SHALL be launchable from the side panel or the injected panel, and from the icon popup on every tab. That count SHALL be the number of cards a review would hold today in the language it covers: the reviewed cards due, and the never-reviewed cards due within that language's allowance of new words left today. Where the injected panel cannot open on the active tab, the popup SHALL open the review in a tab of its own. The front of a card SHALL show the sentence in which its word was met, with the word marked, when the card has one; a card without a sentence SHALL show its word alone. The card's answer — its gloss — SHALL stay hidden until an explicit reveal action.

#### Scenario: Micro-session from the side panel
- **WHEN** the user opens the side panel and starts a review with 3 cards due (French UI copy: « Réviser maintenant »)
- **THEN** the cards come one by one, each showing its sentence with the gloss hidden, then revealed, and the due count goes down

#### Scenario: A word met in a book
- **WHEN** a card captured in a book comes up
- **THEN** its front shows the book's sentence with the word marked, and its gloss appears only once the reader reveals the answer

#### Scenario: A card without a sentence
- **WHEN** a card seeded from a level, which has no sentence, comes up
- **THEN** its front shows the word alone, and its gloss appears only once the reader reveals the answer

#### Scenario: A tab that was not analysed
- **WHEN** the reader opens the icon popup on a tab whose page was not analysed, with 7 cards to review
- **THEN** the popup shows the deck and « Réviser (7) », and pressing it starts the review

#### Scenario: New words beyond today's allowance
- **WHEN** 2 reviewed cards are due and 30 never-reviewed cards wait in English, with an allowance of 10 not yet spent today
- **THEN** the popup and the review count 12 cards to review in English, not 32

#### Scenario: A Firefox tab without the reader
- **WHEN** a reader on Firefox presses « Réviser » in the popup on a browser page where the reader cannot run
- **THEN** the review opens in a tab of its own

## ADDED Requirements

### Requirement: A one-time request to rate the extension at the end of a session
At the end of a session that says it is complete (French UI copy: « Séance terminée »), the review in the side panel and in the injected panel of the Chromium and Firefox builds SHALL ask the reader to rate the extension, from the reader's fifth finished session on this device and no earlier than the third distinct day on which the reader finished a session: one sentence (French UI copy: « Une note sur la boutique de ton navigateur aide d'autres lecteurs à trouver Cymbra Lingua. ») and three actions (French UI copy: « Noter Cymbra Lingua », « Plus tard », « Ne plus demander »), placed below the end's own actions, which stay as they were. A finished session SHALL be one whose end says it is complete, a practice session included; an end that says nothing was due SHALL NOT count and SHALL NOT ask. Once shown, the request SHALL count as made; « Plus tard », or leaving without answering, SHALL ask again after ten more finished sessions; the request SHALL be shown at most twice on a device, ever; « Noter Cymbra Lingua » and « Ne plus demander » SHALL end it. Rating SHALL open, in a new tab, the review page of the store the build is distributed by, through the extension's background, so that no address of a page being read is sent with it; nothing SHALL be sent before the reader chooses to rate. The request SHALL offer nothing in exchange, SHALL select no rating and write no text in advance, SHALL NOT depend on the session's results nor on any question of satisfaction, and SHALL never appear in the popup, the word card, the page bar or the book reader. What it remembers — sessions finished, days, showings, the answer — SHALL stay on the device: it SHALL NOT be synchronised and SHALL NOT be part of the backup file. The Safari extension SHALL show no such request.

#### Scenario: The fifth session, on the third day
- **WHEN** a reader on Chrome finishes their fifth session, having finished sessions on two earlier days
- **THEN** the end shows « Séance terminée », its summary and « Encore 10 », then the request with its three actions

#### Scenario: Five sessions in one evening
- **WHEN** a reader finishes five sessions on the first day they review
- **THEN** no request is shown, and it comes at the end of a session on their third day of review

#### Scenario: Rating
- **WHEN** the reader presses « Noter Cymbra Lingua » on Firefox
- **THEN** the Cymbra Lingua listing on Firefox Add-ons opens in a new tab, and the request never shows again

#### Scenario: Later
- **WHEN** the reader presses « Plus tard », or starts « Encore 10 » without answering
- **THEN** the request does not show again before ten more finished sessions, and after its second showing it never shows again

#### Scenario: Don't ask again
- **WHEN** the reader presses « Ne plus demander »
- **THEN** the request never shows again on that device

#### Scenario: A hard session
- **WHEN** the qualifying session ended with most cards missed
- **THEN** the request is shown all the same

#### Scenario: Nothing due
- **WHEN** the review opens and says nothing is due
- **THEN** no session is counted and no request is shown

#### Scenario: Another device
- **WHEN** the same reader, signed in, reviews on a second browser
- **THEN** that browser counts its own sessions and asks for its own store, and nothing of the first browser's answer reaches it

#### Scenario: No network before the choice
- **WHEN** the request is shown and the reader answers « Plus tard »
- **THEN** the extension makes no network request

#### Scenario: In Safari
- **WHEN** a reader of the Safari extension finishes their fifth session
- **THEN** the drawer's end shows no request
