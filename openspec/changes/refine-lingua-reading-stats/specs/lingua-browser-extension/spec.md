## ADDED Requirements

### Requirement: Daily reading statistics count what was read
The extension SHALL keep, per UTC day and studied language, a count of the **words read** and of the **new words seen**, and SHALL count them only from the text blocks the reader actually saw — a block counts once it has stayed on screen for the reading dwell used by exposure tracking, never merely because its page was opened.
- **Words read** SHALL be the word occurrences of a seen block that enter the page's known-word percentage.
- **New words seen** SHALL be, among those occurrences, the ones classed unknown or being learned — the words the reader sees highlighted. Words presumed known under the declared level, ignored words and known words SHALL NOT count as new.
- Each block SHALL be counted at most once per document shown; a block that appears later (a feed that loads more, a section turned to) SHALL be counted when it is seen, like the first ones. Showing the document again (a reload, a navigation back) counts it again.
- For a signed-in reader, both counters SHALL be pushed with the other daily statistics, the new-words-seen counter being set for every day pushed, zero included.
- The stats screen SHALL show words read as « Mots lus » and SHALL NOT show new words seen: that counter serves the back office only.
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
