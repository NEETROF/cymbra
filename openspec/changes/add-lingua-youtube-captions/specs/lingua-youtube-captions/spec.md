## ADDED Requirements

### Requirement: The caption mode is offered where it can read captions in the studied language
The extension SHALL offer the caption mode only on a desktop YouTube watch page (`/watch`) whose video has a caption track in the studied language, and SHALL NOT offer it anywhere else.
Shorts, live streams and premieres in progress, and players embedded in other sites SHALL NOT be
offered it. A video with no such track SHALL leave the player untouched, and the popup SHALL say
that the video has no captions in the studied language. Moving to another video inside YouTube
SHALL end the mode for the previous video and start it afresh for the next, with nothing carried
over.

#### Scenario: A video without captions in the studied language
- **WHEN** the caption mode is asked for on a video with no English caption track while English is studied
- **THEN** the player is left as it was, nothing is fetched, and the popup says there are no English captions

#### Scenario: A live stream
- **WHEN** the reader opens a live stream in progress
- **THEN** the caption mode is not offered

#### Scenario: Next video in the same tab
- **WHEN** the reader moves to another video without reloading the page
- **THEN** the lines, percentage and translations of the previous video are gone, and the next video's track is read

### Requirement: The caption mode never takes over the player unasked
The caption mode SHALL start only when the reader asks for it on the video, or on every watch page when the reader has turned on a per-device "start automatically on YouTube" preference, which SHALL be off by default.
Having granted the extension every site SHALL NOT by itself start the mode: the mode changes the
player (it may turn its captions on), which reading a page never does.

#### Scenario: A reader who granted every site
- **WHEN** a reader who granted every site opens a watch page without having turned on the preference
- **THEN** the page is read as usual, the mode is offered, and the player is not changed

#### Scenario: The preference on
- **WHEN** the reader has turned on "start automatically on YouTube" and opens a watch page with English captions
- **THEN** the caption mode starts for that video

### Requirement: The track is read through the player's own request
The extension SHALL read the caption track only by replaying a request the YouTube player has itself made for the same video, and SHALL NOT generate a token, keep one beyond its video, or call the caption endpoint before it has seen such a request.
When the player has not requested captions, the extension SHALL cause that request by turning the
player's captions on. A token SHALL be held in memory only and discarded when its video ends.

#### Scenario: The player has not requested captions yet
- **WHEN** the caption mode starts and the player has made no caption request for this video
- **THEN** the extension turns the player's captions on, and reads the track once the player's request is seen

#### Scenario: A token from the previous video
- **WHEN** the reader moves to another video
- **THEN** the token seen for the previous video is not used for any request

### Requirement: Human captions are preferred to automatic ones
When a video has both, the extension SHALL read the track written by a person rather than the automatic one.
It SHALL read the automatic track only when no written one exists in the studied language, and the
popup SHALL say which of the two the mode is reading.

#### Scenario: Both tracks exist
- **WHEN** a video offers a written English track and an automatic English track
- **THEN** the written track is read, and the popup names it as such

### Requirement: The video's percentage is known before it is played
While the caption mode is on, the badge SHALL show the percentage of known words of the whole caption track as soon as the track is read, before and independently of playback.
The popup SHALL break the video's percentage down as it does for a page, and SHALL show the rest of
the watch page's percentage apart from it. Annotations such as `[Music]` and fillers such as "um"
SHALL NOT count as words.

#### Scenario: Opening a video
- **WHEN** a video's track has been read and the video has not been played
- **THEN** the badge shows the percentage known of the whole track

#### Scenario: Annotations in the track
- **WHEN** the track contains `[Music]` and "uh"
- **THEN** neither changes the percentage

### Requirement: Lingua's caption line follows the video
While the mode is on, the extension SHALL show the caption line for the video's current time in its own UI host, with unknown and learning words highlighted as on a page, and SHALL hide the player's native caption line.
The line SHALL stay in time through play, pause, seek and a change of playback rate, in the
default, theatre and full-screen layouts, and SHALL NOT repaint a line that has not changed. A
status change made anywhere SHALL repaint the line immediately. While an advertisement plays, the
line SHALL be hidden.

#### Scenario: Seeking
- **WHEN** the reader seeks to the middle of a caption line
- **THEN** that line is shown at once, highlighted

#### Scenario: A word marked known in another tab
- **WHEN** the reader marks a word known in another tab while it is on the caption line
- **THEN** the word's highlight disappears from the caption line immediately

#### Scenario: An advertisement
- **WHEN** an advertisement starts during the video
- **THEN** the caption line is hidden until the video resumes

### Requirement: The caption line is captured from as a page is
A click on a highlighted word of the caption line SHALL open the extension's word popup for that word, and a selection of several words of the line SHALL be captured as a phrase exactly as on a page; either SHALL pause the video if it was playing.
The caption sentence SHALL be the source sentence. Closing the popup SHALL resume playback only if
the popup paused it. The popup's actions SHALL behave as on a page. A card made from a caption SHALL
keep, on the device only, the watch page address at the time of its line, so that opening it
returns to that moment.

#### Scenario: Adding a word to the deck from a video
- **WHEN** the reader clicks a highlighted word on the caption line of a playing video and chooses "+ Deck"
- **THEN** the video pauses, a card is created with the caption sentence as source, and the word switches to the learning highlight

#### Scenario: Capturing a phrase from a video
- **WHEN** the reader selects "figure it out" on the caption line
- **THEN** the video pauses and the phrase is offered for capture with the caption sentence as source

#### Scenario: Closing the popup
- **WHEN** the reader closes a popup that paused the video
- **THEN** the video resumes; a video the reader had paused stays paused

#### Scenario: Returning to the moment
- **WHEN** the reader opens the source of a card captured at 3 min 12 s of a video
- **THEN** the watch page opens at that time

### Requirement: Only a caption line played through is an exposure
A caption line SHALL be recorded as an exposure only when playback has covered most of the line's own time span while the video was playing, and SHALL be recorded at most once per video session.
The threshold SHALL be measured in video time, so that a change of playback rate neither prevents
nor multiplies it. Reading the track, analysing it, a line jumped over by a seek, a line shown
while paused, and annotations and fillers SHALL NOT be recorded. What is recorded SHALL go through
the same exposure path as page reading, under the same conditions. There is no audio aspect: what
the reader hears is not counted.

#### Scenario: A track read but not watched
- **WHEN** a video's track has been read and the reader leaves without playing it
- **THEN** no exposure is recorded

#### Scenario: Seeking past lines
- **WHEN** the reader seeks from the first minute to the tenth
- **THEN** the lines between them are not recorded

#### Scenario: Watching a line twice
- **WHEN** the reader rewinds and watches the same line again
- **THEN** it is recorded once

#### Scenario: Watching at double speed
- **WHEN** the reader watches a line through at playback rate 2
- **THEN** it is recorded, once

### Requirement: The translation shown depends on whether an engine is chosen
The caption line's translation SHALL come from the translation engine when the reader has one available and chosen, and otherwise SHALL come only from YouTube's own automatic translation of the line.
With an engine, the translation of a caption sentence SHALL be the engine's, computed ahead of the
line being shown without delaying an answer the reader asked for, and a word opened from the line
SHALL show the sentence translation with that word marked by the same rule as on a page. Without
an engine, the translation SHALL be YouTube's automatic translation of the line into the language
of the reader's glosses, SHALL be labelled as YouTube's automatic translation, SHALL NOT be
presented as aligned word by word, SHALL NOT mark a word, and SHALL NOT be offered outside YouTube
or for a track YouTube does not translate. The translation line SHALL be hidden until the reader
shows it, and that choice SHALL be kept per device. No translation, from either source, SHALL be
stored on a card or sent anywhere.

#### Scenario: No engine
- **WHEN** a reader with no engine shows the translation line on a video
- **THEN** the line shows YouTube's automatic translation, labelled as such, and no model is downloaded

#### Scenario: An engine chosen
- **WHEN** a reader with an engine chosen opens a word from the caption line
- **THEN** the popup shows the engine's translation of the caption sentence with that word marked

#### Scenario: A card from a translated caption
- **WHEN** the reader creates a card from a caption whose translation was shown
- **THEN** the card holds the caption sentence and dictionary data, and no translation

### Requirement: A track that cannot be read degrades, and says so
When the track cannot be read, the extension SHALL fall back to the caption lines the player writes into the page, and SHALL say in the popup that the video is in degraded mode.
In degraded mode the line SHALL be highlighted and captured from as in the full mode, the
percentage SHALL be that of the lines seen so far and labelled so, YouTube's translation SHALL NOT
be shown, and an engine SHALL translate only the line on screen when the reader opens a word. The
failure SHALL be recorded locally by kind, holding no caption text, so that a breakage is visible in
diagnostics.

#### Scenario: The caption request returns nothing
- **WHEN** the replayed caption request answers with an empty body
- **THEN** the mode reads the player's own caption lines, the popup says it is degraded, and the failure kind is recorded

### Requirement: The player is left as the reader left it
Ending the caption mode SHALL restore the player's native caption line and the captions state the reader had before the mode started.
Captions turned on by the extension to obtain the track SHALL be turned off again when the mode
ends; captions the reader had turned on SHALL stay on. The mode SHALL end when the reader turns it
off, leaves the watch page, or moves to a video it does not start on.

#### Scenario: Captions turned on by the extension
- **WHEN** the reader had captions off, the mode turned them on, and the reader turns the mode off
- **THEN** the player's captions are off again

#### Scenario: Captions the reader wanted
- **WHEN** the reader had captions on before the mode started, and turns the mode off
- **THEN** the player's native captions are shown again
