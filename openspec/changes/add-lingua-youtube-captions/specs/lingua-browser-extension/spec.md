## MODIFIED Requirements

### Requirement: No network requests
The extension SHALL issue no network request unless the reader has signed in, turned on extended translation, or started the YouTube caption mode, and SHALL then issue only the requests those choices need.
- **Signed in:** the requests of signing in, and those to the Cymbra backend for the account and
  the synchronisation of what `lingua-privacy` lists as synced.
- **Signed in, « Traduction en ligne » ticked:** for each card on which the reader presses the
  button, one request to the Cymbra backend carrying the sentence that holds the selection, and
  nothing before the press.
- **Extended translation on:** the download of the model from Cymbra's static host, once per model
  version, and nothing else.
- **YouTube caption mode on:** requests to YouTube for the captions of the video being watched —
  the player's own caption request replayed for the track the mode reads — and nothing else, only
  while the mode is on. Reading the list of a video's caption tracks makes no request.

Page text SHALL NOT be part of any request, except the sentence the reader sends by pressing
« Traduction en ligne ». The pack, the glosses and the engine are part of the extension, so
reading, highlighting, the word popup and — once the model is on the device — translation SHALL
work with no network connection.

#### Scenario: Working offline
- **WHEN** the user reads an already-loaded page with no network connection
- **THEN** highlighting and the word popup (gloss included) work in full

#### Scenario: No account and no extended translation
- **WHEN** a reader who is not signed in and has not turned on extended translation reads pages for a day
- **THEN** the extension has made no network request

#### Scenario: Translating offline
- **WHEN** a reader whose model is on the device selects a phrase with no network connection
- **THEN** the sentence is translated

#### Scenario: Online translation not pressed
- **WHEN** a reader who ticked « Traduction en ligne » selects several words and closes the card without pressing the button
- **THEN** no request has carried the sentence

#### Scenario: Watching YouTube with the caption mode off
- **WHEN** a reader who is not signed in and has not turned on extended translation watches YouTube videos without starting the caption mode
- **THEN** the extension has made no network request, although it read each video's list of caption tracks

### Requirement: Daily reading statistics count what was read
The extension SHALL keep, per UTC day and studied language, a count of the **words read** and of the **new words seen**, and SHALL count them only from the text blocks the reader actually saw — a block counts once it has stayed on screen for the reading dwell used by exposure tracking, never merely because its page was opened — and from the caption lines the reader watched in the YouTube caption mode, a line counting once it was played through as `lingua-youtube-captions` defines it.
- **Words read** SHALL be the word occurrences of a seen block, or of a caption line played through, that enter the known-word percentage of its page or video.
- **New words seen** SHALL be, among those occurrences, the ones classed unknown or being learned — the words the reader sees highlighted. Words presumed known under the declared level, ignored words and known words SHALL NOT count as new.
- Each block SHALL be counted at most once per document shown; a block that appears later (a feed that loads more, a section turned to) SHALL be counted when it is seen, like the first ones. Showing the document again (a reload, a navigation back) counts it again.
- A caption line SHALL be counted in English, at most once per video session, under the same rule as its exposure: a line skipped by a seek, a line shown only while paused or during an advertisement, annotations such as `[Music]` and fillers SHALL NOT be counted. Watching the video again in a new session counts its lines again.
- The words read and new words seen that come from caption lines SHALL also be kept apart, per UTC day and studied language, under their own origin (video), so that the statistics can tell them from those read on pages and in books. They SHALL stay part of the two counters above.
- For a signed-in reader, both counters SHALL be pushed with the other daily statistics, the new-words-seen counter being set for every day pushed, zero included. The share that comes from video SHALL stay on the device.
- The stats screen SHALL show words read as « Mots lus », caption lines included, and SHALL NOT show new words seen: that counter serves the back office only.
- These counters SHALL NOT change the per-word exposure counters or exposure-confirmed promotion.

#### Scenario: A long page read only at the top
- **WHEN** the reader opens a page of 40 paragraphs, reads the first 3 for longer than the dwell and closes it
- **THEN** the day's words read grow by the counted occurrences of those 3 paragraphs only, not of the 40

#### Scenario: Scrolling past without stopping
- **WHEN** a paragraph crosses the screen during a fast scroll and leaves it before the dwell
- **THEN** neither counter changes for that paragraph

#### Scenario: New words are the highlighted ones
- **WHEN** a seen paragraph holds 50 counted occurrences, of which 4 are unknown, 1 is being learned and the rest are known or presumed known under the declared level
- **THEN** words read grow by 50 and new words seen grow by 5

#### Scenario: Content that arrives later
- **WHEN** a page loads more paragraphs as the reader scrolls and the reader reads them
- **THEN** those paragraphs are counted when they are seen, although they were not part of the first analysis

#### Scenario: Re-analysis does not recount
- **WHEN** the reader marks a word known and the page is re-analysed while the same paragraphs stay on screen
- **THEN** no paragraph already counted for this document is counted again

#### Scenario: The stats screen shows words read only
- **WHEN** a reader opens the stats view
- **THEN** it shows « Mots lus », « Mots appris » and « Révisions », with no new-words-seen card and no occurrence of "exposition" or "lemma"

#### Scenario: Caption lines watched
- **WHEN** the reader watches, in the caption mode, three caption lines through, of 8 counted occurrences each, one of which is unknown
- **THEN** the day's words read in English grow by 24, new words seen grow by 3, and the share from video grows by the same amounts

#### Scenario: Caption lines skipped
- **WHEN** the reader seeks past ten caption lines and watches the next one through
- **THEN** only that line's occurrences are counted

#### Scenario: « Mots lus » after a video
- **WHEN** a reader who read 300 words on pages and watched 200 words of captions today opens the stats view
- **THEN** « Mots lus » shows 500 for today
