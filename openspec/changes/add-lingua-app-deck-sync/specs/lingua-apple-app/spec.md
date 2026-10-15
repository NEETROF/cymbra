## MODIFIED Requirements

### Requirement: The app reviews the deck the extension keeps
The Cymbra Lingua app SHALL open on the review of the deck the extension keeps on the same device: the extension's own Révision page — its language filter and due count, its sessions, answers, read-aloud, machine translations and end summary — laid out for the screen it is on. The app SHALL obtain the deck through the container it shares with the extension and, when the reader is signed in inside the app to the account the extension is signed in to, SHALL also synchronise it with Cymbra's server under the app's own session, merging the three — the extension's shared state, the app's changes not yet taken by the extension, and the server's — by last-write-wins per card and per word. Until the extension has shared its state, the app SHALL say how to turn the extension on.
The only network requests the app SHALL make are the download of translation models the reader
accepted, from Cymbra's model host; the reader's sign-in — for an email account, with its
verification code and password reset — and the reading of the signed-in account's access and name;
the sentence sent when the reader presses « Traduction en ligne »; and, while signed in inside the app
to the extension's account, the synchronisation of the reader's statuses and cards with Cymbra's
server. No other request SHALL carry the deck.

#### Scenario: A review on the train
- **WHEN** a reader who captured words in Safari opens the app on their iPhone
- **THEN** the app shows the due count and starts the same review the Safari drawer would, its answers at the bottom of the screen

#### Scenario: On a Mac
- **WHEN** the reader opens the app on their Mac
- **THEN** the review is shown in the app's window, as a centred column

#### Scenario: Before the extension
- **WHEN** the reader opens the app before turning the extension on
- **THEN** the app explains how to turn it on, and shows no review

#### Scenario: Reviewed on the Mac since Safari's last snapshot
- **WHEN** a reader signed in inside the app reviewed ten cards on their Mac this morning, and Safari on their iPhone has not run since
- **THEN** the app on the iPhone opens on a review where those ten cards are no longer due

#### Scenario: Not signed in inside the app
- **WHEN** a reader who has not signed in inside the app opens it
- **THEN** the app reviews the extension's shared state alone and sends no request carrying the deck

### Requirement: No learning state in the app
The container app SHALL NOT keep a deck of its own across openings: each opening SHALL rebuild the review from the extension's shared state, the app's changes not yet taken by the extension and, when it synchronises, the server's state. The Safari extension SHALL keep its statuses, decks and review state in its own extension storage, under the same versioned schema as the other variants, and SHALL synchronise on its own like them. Beyond that, the app SHALL keep only its own session (as `lingua-sync` specifies it), its pending changes in the shared container, and the bookkeeping its synchronisation needs — its device identifier, its pull positions and the erasure mark it acted on — whose loss SHALL cost no more than pulling again.

#### Scenario: Reading without opening the app again
- **WHEN** the user has enabled the extension and never reopens the app
- **THEN** highlighting, decks and review all work from Safari alone

#### Scenario: The app's storage cleared
- **WHEN** the system clears the app page's storage between two openings of a signed-in reader
- **THEN** the next opening pulls the reader's deck again from the start and loses no review

## ADDED Requirements

### Requirement: The app synchronises the reviews made in it
While the reader is signed in inside the app to the extension's account, the app SHALL exchange with Cymbra's server: on opening and on returning to the foreground, before the review starts, waiting at most a few seconds and otherwise opening on the extension's shared state and applying the server's answer between sessions; after answers, debounced; at the end of a session; and when the app goes to the background. It SHALL push only the changes the app's openings made — the same changes it hands to the extension — and SHALL keep handing them to the extension through the shared container. A change the app could not push SHALL stay in the shared container and SHALL be pushed at a later opening. Exchanges SHALL be throttled like a Lingua surface the reader opens in the extension, SHALL never block the review, and SHALL report a failure in one quiet line, by category, never as a raw message.

#### Scenario: Reviewed in the app, seen on another device without opening Safari
- **WHEN** a signed-in reader reviews ten cards in the app on their iPhone, closes it, and opens the side panel on their Mac without having opened Safari on the iPhone
- **THEN** the Mac's due count no longer holds those ten cards

#### Scenario: Offline on the train
- **WHEN** the reader reviews in the app with no network and opens it again later with a network
- **THEN** the reviews made offline reach the server at that opening, and none is lost

#### Scenario: Both paths carry the same review
- **WHEN** the app has pushed a review and the extension later collects the same change from the shared container and synchronises
- **THEN** the card's state on every device is the one the review produced, applied once, and a change the extension made to that card afterwards still wins

#### Scenario: A slow network at opening
- **WHEN** the server does not answer within the wait at opening
- **THEN** the review opens on the extension's shared state, and the server's answer is applied once no session is under way

### Requirement: The app synchronises only on the extension's account
The app SHALL synchronise only when the extension on the same device is signed in to the same Cymbra account as the app. The extension SHALL share, with its state, a digest of its account's identifier — never a token — or that it is signed out. When the accounts differ, when the extension is signed out, or when the extension shares no account, the app SHALL NOT synchronise, SHALL say why in its account line, and SHALL keep handing its changes to the extension as before; online translation SHALL be unaffected.

#### Scenario: Another account in Safari
- **WHEN** Safari's extension is signed in to account A and the reader signs in inside the app to account B
- **THEN** the app sends no deck to the server, its account line says (French UI copy) « Safari est connecté à un autre compte : tes révisions passeront par Safari. », and its reviews still reach the extension

#### Scenario: Safari signed out
- **WHEN** the extension is signed out and the reader is signed in inside the app
- **THEN** the app does not synchronise and its line says (French UI copy) « Connecte aussi Safari à ce compte pour synchroniser depuis l'app. »

#### Scenario: An extension that predates the account digest
- **WHEN** the extension's shared state carries no account at all
- **THEN** the app does not synchronise and its line says (French UI copy) « Mets à jour l'extension Safari pour synchroniser depuis l'app. »

### Requirement: The app's synchronisation respects an erasure
The extension SHALL share, with its state, the erasure mark its store last acted on. When the account's erasure mark is newer than that, the app SHALL drop the shared state and its pending changes, SHALL leave no pending change for the extension to collect, and SHALL review what the server holds. The app SHALL NOT push a change made on a state that predates the erasure.

#### Scenario: Erased on the Mac, Safari not run since
- **WHEN** the reader erases their Lingua data on their Mac, then opens the app on their iPhone, signed in, before Safari has run
- **THEN** the app shows no card from before the erasure, pushes nothing from that state, and the extension finds no change of the app to apply

### Requirement: A review in the app is counted once in the account's statistics
The app SHALL NOT send daily statistics itself. Its review counts SHALL keep reaching the extension through the shared container, which SHALL count each opening's reviews and words learned once, per studied language, and send them as its own device's. The day's allowance of new words SHALL be derived from the merged deck, so cards introduced today on another device that the app has pulled count against it.

#### Scenario: Ten reviews in the app
- **WHEN** the reader reviews ten cards in the app, signed in, and Safari later synchronises
- **THEN** the account's statistics for that day and language count ten reviews from the app, not twenty

#### Scenario: New words introduced on the Mac
- **WHEN** the allowance is ten new words a day, the reader introduced six on their Mac this morning, and opens the app signed in
- **THEN** the app's review introduces at most four more today

### Requirement: The app offers its sign-in on opening
When the extension is signed in and the app is not, the app SHALL offer its sign-in in a quiet line as soon as it opens (French UI copy: « Connecte-toi pour synchroniser tes révisions et utiliser la traduction en ligne. » with « Se connecter »), with no dialog, and SHALL let the reader set it aside for a while (French UI copy: « Plus tard »). Signed in and synchronising, the line SHALL say so, naming the account when it is known as the signed-in line does (French UI copy: « Connecté en tant que @handle : tes révisions se synchronisent », else « Connecté à Cymbra : tes révisions se synchronisent ») with « Se déconnecter ». « Se connecter » opens the app's sign-in sheet, with every way it offers to sign in, email and password included. The sign-in sheet SHALL say that the session serves synchronisation and online translation and asks for the account used in Safari (French UI copy: « Connecte-toi pour synchroniser les révisions faites dans l'app et utiliser la traduction en ligne. Utilise le même compte que dans Safari. »). The copy SHALL exist in every interface language (French, English, Spanish).

#### Scenario: First opening after this change
- **WHEN** a reader signed in to Safari's extension opens the app, not signed in inside it
- **THEN** the review opens at once, with the sign-in line under it, and nothing else asks for attention

#### Scenario: Set aside
- **WHEN** the reader chooses « Plus tard »
- **THEN** the line is not shown at the next openings for a while, and the first press of « Traduction en ligne » still opens the sign-in sheet
