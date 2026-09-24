## ADDED Requirements

### Requirement: The card reads the selection and its sentence aloud
The word card and the selection card SHALL offer a listen row with one button that reads the
selection as it appears on the page and one that reads the whole sentence it was taken from,
the sentence button being left out when the selection already is that sentence. Speech SHALL
start from the press itself, with no wait for anything else. A button that is speaking SHALL
act as its own stop button, and pressing the other button SHALL stop the current reading and
start the other. A pending card SHALL offer the listen row, and its completion SHALL NOT
interrupt a reading already started. Every label SHALL be text, and no user-visible string
SHALL hold the word "lemma".

#### Scenario: Hearing a word as seen
- **WHEN** the reader opens the card of `ran` and presses the selection's listen button
- **THEN** `ran` is spoken, as it appears on the page, not its dictionary form

#### Scenario: Hearing the selected words
- **WHEN** the reader selects several words and presses the selection's listen button
- **THEN** the selected words are spoken as one utterance

#### Scenario: Hearing the sentence
- **WHEN** the reader presses the sentence's listen button on a card
- **THEN** the whole sentence the selection was taken from is spoken

#### Scenario: A selection that is its whole sentence
- **WHEN** the reader selects a whole sentence
- **THEN** the card offers the selection's listen button and no sentence button

#### Scenario: Stopping
- **WHEN** the reader presses the button that is speaking
- **THEN** the speech stops and the button offers to listen again

#### Scenario: Switching
- **WHEN** the sentence is being spoken and the reader presses the selection's listen button
- **THEN** the sentence stops and the selection is spoken

#### Scenario: The answer arrives while speaking
- **WHEN** the reader presses a listen button on a pending card and the card then completes with its answer
- **THEN** the speech goes on, and the completed card shows that button as speaking

### Requirement: Closing the card silences it
Speech started from a card SHALL stop when that card is hidden — by its close button, Escape, a
click off it, a gesture, or the scroll that dismisses it — when another card opens for a
different text, when the reader switches the reader off, and when the page or its tab is hidden.
No speech SHALL outlive the control that stops it.

#### Scenario: Closing while speaking
- **WHEN** the sentence is being spoken and the reader closes the card
- **THEN** the speech stops

#### Scenario: Another word
- **WHEN** a word is being spoken and the reader opens the card of another word
- **THEN** the speech stops

#### Scenario: Leaving the tab
- **WHEN** the sentence is being spoken and the reader switches to another tab
- **THEN** the speech stops

### Requirement: Read-aloud never leaves the device
The extension SHALL speak only with a voice that the browser reports as running on the device
and whose language is the studied language, and SHALL never pass page text to a voice that
synthesises remotely, even when that voice is the browser's default or the only one available.
With no eligible voice the card SHALL show no listen row. Read-aloud SHALL add no permission
and no network request of the extension's own, and SHALL work offline. The language SHALL be
recognised in its two- and three-letter forms (`en`, `eng`).

#### Scenario: Three-letter language codes
- **WHEN** a browser lists a voice as `eng-GBR-default`
- **THEN** it is a voice of the studied language, from the United Kingdom

#### Scenario: A remote voice is the default
- **WHEN** the browser's default voice for the studied language synthesises remotely and an on-device voice for it exists
- **THEN** the on-device voice speaks, and the remote voice never receives the text

#### Scenario: Only remote voices
- **WHEN** every voice the browser offers for the studied language synthesises remotely
- **THEN** the card shows no listen row

#### Scenario: Offline
- **WHEN** the reader has no network connection and an on-device voice exists
- **THEN** the selection and the sentence are spoken

#### Scenario: Voices announced late
- **WHEN** the browser lists its voices only after the card has opened
- **THEN** the listen row appears on that card once an eligible voice is listed, without the reader reopening it

### Requirement: Android's own voices speak only on the reader's say-so
Android's own voices SHALL speak only once the reader has allowed them in Réglages, never by
default: Firefox for Android reports every voice of Android's engine as not local, because it
cannot tell where that engine synthesises. Réglages SHALL offer that switch only where the
browser lists such voices, SHALL say beside it that the text read may then leave the device
depending on the engine chosen in Android, and SHALL no longer claim that the voices stay on the
device while it is on. The choice SHALL be kept on the device and never synchronised.

#### Scenario: By default
- **WHEN** a reader on Firefox for Android opens a card
- **THEN** the card shows no listen row, and Réglages offers "Utiliser la voix d'Android", off, with the note saying why

#### Scenario: Allowed
- **WHEN** the reader switches "Utiliser la voix d'Android" on
- **THEN** the cards offer the listen row, speaking with Android's English voices, and Réglages lists those voices

#### Scenario: Elsewhere
- **WHEN** the browser lists no voice of Android's engine
- **THEN** Réglages shows no such switch

### Requirement: One Réglages on every surface
Réglages SHALL be built by a single implementation that every surface renders — the side
panel, the in-page drawer and the toolbar popup — so that a setting added to Réglages appears
on all of them at once. No surface SHALL keep a Réglages block of its own.

#### Scenario: The toolbar popup
- **WHEN** the reader opens Réglages from the toolbar popup
- **THEN** it shows the same blocks as the side panel and the in-page drawer, the read-aloud block included

#### Scenario: A block added later
- **WHEN** a block is added to Réglages
- **THEN** the side panel, the in-page drawer and the toolbar popup all show it, with no change to any of them

### Requirement: The reader chooses the voice
Réglages SHALL show a read-aloud block listing the eligible voices, with an automatic choice
selected by default and a way to hear each voice, and SHALL leave the block out when there is no
eligible voice. The choice SHALL be kept on the device as a preference. The automatic choice
SHALL prefer the eligible voice the browser marks as default when it is the only voice so
marked, and otherwise SHALL never pick a novelty voice while an ordinary one exists. The block
SHALL list the ordinary voices first and the novelty voices after them, in a group of their own,
whatever language the device names them in. A voice the device offers in several qualities
SHALL be listed once, in its best quality.
A chosen voice that is no longer listed SHALL fall back to the automatic choice.

#### Scenario: The automatic choice on macOS
- **WHEN** the eligible voices are listed with novelty voices such as "Albert" and "Bubbles" before an ordinary voice such as "Samantha"
- **THEN** the automatic choice is the ordinary voice

#### Scenario: Every voice marked default
- **WHEN** the browser marks every voice as default and lists novelty voices before an ordinary one
- **THEN** the automatic choice is the ordinary voice, not the first voice listed

#### Scenario: Novelty voices out of the way
- **WHEN** the eligible voices include novelty voices
- **THEN** Réglages lists the ordinary voices first and the novelty voices in an "Autres voix" group at the bottom

#### Scenario: Novelty voices named in the reader's language
- **WHEN** an iPhone set to French names its novelty voices "Bulles", "Murmure" or "Cloches"
- **THEN** they are listed under "Autres voix" and the automatic choice is still an ordinary voice

#### Scenario: One voice in two qualities
- **WHEN** the device lists the same voice twice, in a compact and a super-compact quality
- **THEN** Réglages lists it once, and the compact one is the voice that speaks when chosen

#### Scenario: A chosen voice
- **WHEN** the reader chooses a voice in Réglages and then listens from a card
- **THEN** that voice speaks, on every page of that browser

#### Scenario: A chosen voice removed
- **WHEN** the voice the reader chose is no longer installed
- **THEN** the card speaks with the automatic choice, and Réglages shows the automatic choice selected
