## ADDED Requirements

### Requirement: Content added after the first paint is analysed
Text that reaches the page after the first paint SHALL be analysed and highlighted once it is on screen, whether it lands in a block already painted or in a container the reader has never scanned.
Text added off screen SHALL be analysed when it scrolls into view. A change that leaves every
block's text as it was SHALL NOT cause the page to be analysed again.

#### Scenario: A sentence added to a highlighted paragraph
- **WHEN** a page appends "Unprecedented circumstances demanded extraordinary measures." to a paragraph the reader has already highlighted
- **THEN** the new words are highlighted like the rest of the paragraph

#### Scenario: A paragraph added to a painted article
- **WHEN** a page appends a new paragraph to an article already on screen
- **THEN** the new paragraph is highlighted without the whole page being walked again

#### Scenario: A section loaded into an empty container
- **WHEN** a site loads its comments on scroll into a container that held no text when the page was first painted
- **THEN** the comments are highlighted once they are on screen

#### Scenario: A change with no new text
- **WHEN** a page updates a clock that shows only digits, every second
- **THEN** the page is not analysed again

### Requirement: A block the reader sees counts as read from the first paint
A block that stays on screen for the reading dwell SHALL be recorded as read from the first paint onward, without the reader making any gesture.
What is recorded, and for which words, SHALL follow `lingua-knowledge-model`'s **Exposure
counters**; a block SHALL be recorded at most once per page load.

#### Scenario: Reading without touching anything
- **WHEN** a reader who has declared a level opens a page and reads its first screen for longer than the dwell, without clicking
- **THEN** the words of the blocks on that screen are recorded as read

### Requirement: The page reader never reads a video player's caption line
On YouTube the page reader SHALL exclude the player's caption area from scanning, highlighting and exposure, whether or not any caption mode is on.
The rest of a watch page — title, description, comments — SHALL be read as any page is. The
player rewrites its caption line every one or two seconds, replacing the whole caption window each
time; reading it as page text would paint text already gone and count every line as a new block.

#### Scenario: Watching with captions on
- **WHEN** the page reader runs on a watch page whose native captions are showing
- **THEN** no word of the caption line is highlighted and no exposure is recorded from it, while the comments are highlighted as usual
