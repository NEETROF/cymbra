## ADDED Requirements

### Requirement: The caption mode is offered only where it can read English captions
The extension SHALL offer the caption mode only to a reader who studies English, on a YouTube watch page (`/watch`) opened in a desktop browser — Chromium, Firefox or Safari on macOS — whose video has an English caption track, and SHALL NOT offer it anywhere else.
Whether a video has an English caption track SHALL be known when its watch page loads, from the
list of tracks the page already holds (see **Only the list of tracks is read before the mode
starts**), so that the offer — a control near the player (French interface: « Sous-titres
Lingua ») and a line in the popup — appears only on a video where the mode works.
English means a written or an automatic English track. Spanish, French and Italian captions are
out of this change's scope (the Spanish programme's decision D7, `docs/lingua/spanish-programme.md`,
carried by the language matrix programme, `docs/lingua/language-matrix-programme.md`, and by the
Italian programme, `docs/lingua/italian-programme.md`): a reader who studies Spanish, French or
Italian as well as English is offered the mode on English captions only, and a reader who does not
study English (an English speaker reading Spanish, French or Italian, or a French or Spanish speaker
reading Italian alone) is not offered it, whatever their native language.
Shorts, live streams and premieres in progress, players embedded in other sites, YouTube's mobile
site, and Safari on iPhone and iPad SHALL NOT be offered it. Where the mode is not offered, the
player SHALL be left untouched, and the popup SHALL say why when the page is a watch page
(French interface: « Cette vidéo n'a pas de sous-titres en anglais. » or « Lingua lit les
sous-titres en anglais seulement. »). Moving to another video inside YouTube SHALL end the mode
for the previous video and decide the offer afresh for the next, with nothing carried over.

#### Scenario: A video without English captions
- **WHEN** a reader who studies English opens a watch page whose video has no English caption track
- **THEN** no « Sous-titres Lingua » control appears near the player, nothing is fetched, the player is left as it was, and the popup says the video has no English captions

#### Scenario: A video with automatic English captions only
- **WHEN** a reader who studies English opens a watch page whose only English track is YouTube's automatic one
- **THEN** the control appears near the player, and the mode, once started, says it reads automatic captions

#### Scenario: A Spanish video for a reader of Spanish
- **WHEN** a reader who studies English and Spanish opens a video whose only captions are Spanish
- **THEN** the caption mode is not offered on it, and the popup says it reads English captions only

#### Scenario: A French video for a reader of French
- **WHEN** a reader who studies English and French opens a video whose only captions are French
- **THEN** the caption mode is not offered on it, and the popup says it reads English captions only

#### Scenario: An Italian video for a reader of Italian
- **WHEN** a reader who studies English and Italian opens a video whose only captions are Italian
- **THEN** the caption mode is not offered on it, and the popup says it reads English captions only

#### Scenario: A Spanish-speaking reader of English
- **WHEN** a reader whose native language is Spanish and who studies English opens a video with an English caption track
- **THEN** the caption mode is offered, as it is to a French-speaking reader of English

#### Scenario: An Italian-speaking reader of English
- **WHEN** a reader whose native language is Italian and who studies English opens a video with an English caption track
- **THEN** the caption mode is offered, as it is to a French-speaking reader of English, its control and popup lines in the Italian interface

#### Scenario: An English-speaking reader
- **WHEN** a reader whose native language is English, who studies Spanish, French or Italian and not English, opens a video with an English caption track
- **THEN** the caption mode is not offered, and the popup says it reads English captions for a reader who studies English

#### Scenario: A live stream
- **WHEN** the reader opens a live stream in progress
- **THEN** the caption mode is not offered

#### Scenario: Safari on an iPad
- **WHEN** a reader opens a watch page with English captions in Safari on an iPad, which loads YouTube's desktop site
- **THEN** the caption mode is not offered, and the page is read as any page is

#### Scenario: Next video in the same tab
- **WHEN** the reader moves to another video without reloading the page
- **THEN** the lines and percentage of the previous video are gone, and the offer is decided from the next video's own list of tracks

### Requirement: Only the list of tracks is read before the mode starts
Until the caption mode starts on a video, the extension SHALL read nothing of that video's captions but the list of its caption tracks — each track's language, whether it is automatic, and whether YouTube translates it — and the facts that decide the offer: the video's identifier and whether it is a Short, a live stream or a premiere in progress.
That list SHALL be read from what the watch page already holds, by a script running in the
page's own context that makes no network request, sends nothing anywhere, and changes nothing
in the player; it SHALL be read again when YouTube moves to another video inside the same page.
That script SHALL be placed only on a watch page where the reader has Lingua switched on and
studies English, in a desktop browser; a reader who does not study English, or who switched Lingua
off, SHALL get no script of the caption mode in the page. No caption text SHALL be read, and no
caption request SHALL be made or replayed, before the mode starts.

#### Scenario: Opening a watch page with the mode not started
- **WHEN** a reader opens a watch page with English captions and does not start the mode
- **THEN** the extension knows the video's tracks, has made no request, has read no caption text, and the player's captions are as the reader left them

#### Scenario: A reader who does not study English
- **WHEN** a reader who studies Spanish only opens a watch page
- **THEN** no script of the caption mode is placed in the page, and the page is read as any page is

#### Scenario: A reader of Italian alone
- **WHEN** a reader whose native language is French and who studies Italian only opens a watch page
- **THEN** no script of the caption mode is placed in the page, and the page is read as any page is

### Requirement: The caption mode never takes over the player unasked
The caption mode SHALL start only when the reader asks for it on the video, or on every watch page where it is offered when the reader has turned on a per-device preference (French interface: « Démarrer automatiquement sur YouTube »), which SHALL be off by default.
Having granted the extension every site SHALL NOT by itself start the mode: the mode changes the
player (it may turn its captions on), which reading a page never does.

#### Scenario: A reader who granted every site
- **WHEN** a reader who granted every site opens a watch page with English captions without having turned on the preference
- **THEN** the page is read as usual, the mode is offered, and the player is not changed

#### Scenario: The preference on
- **WHEN** the reader has turned on « Démarrer automatiquement sur YouTube » and opens a watch page with English captions
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
When a video has both, the extension SHALL read the English track written by a person rather than the automatic one.
It SHALL read the automatic track only when no written English one exists, and the popup SHALL
say which of the two the mode is reading (French interface: « Sous-titres écrits » or
« Sous-titres automatiques »).

#### Scenario: Both tracks exist
- **WHEN** a video offers a written English track and an automatic English track
- **THEN** the written track is read, and the popup names it « Sous-titres écrits »

### Requirement: Captions are read as English, whatever the page's language
The caption track SHALL be analysed, highlighted, looked up, captured, translated and counted as English, whatever language the watch page is detected in.
The rest of the watch page — title, description, comments — SHALL be read as any page is, in the
language detected for it.

#### Scenario: A watch page in French around English captions
- **WHEN** a reader who studies English and French watches, in the caption mode, a video with English captions whose description and comments are in French
- **THEN** the words of the caption line are looked up in English and their cards are English cards, while the description is read as French

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
While the mode is on, the extension SHALL show the caption line for the video's current time in its own element, with unknown and learning words highlighted as on a page, and SHALL hide the player's native caption line.
The line SHALL stay in time through play, pause, seek and a change of playback rate, in the
default, theatre and full-screen layouts, and SHALL NOT repaint a line that has not changed. A
status change made anywhere SHALL repaint the line immediately. While an advertisement plays, the
line SHALL be hidden. The line SHALL change nothing of the page's own markup but the hiding of
the native line.

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
A click on a highlighted word of the caption line SHALL open the extension's word card for that word, and a selection of several words of the line SHALL open the selection card exactly as on a page; either SHALL pause the video if it was playing.
The caption sentence SHALL be the source sentence. The card SHALL answer as it does on a page: the
pack's answer first, the translation engine's translation of the caption sentence when the reader
has one ready for English and a page would ask it (a selection of several words, or a word the
pack does not gloss), and, on a card for several words, the « Traduction en
ligne » button when the reader ticked it, sending the caption sentence only when pressed. The card
SHALL be shown over the player in every layout, full screen included. Closing the card SHALL
resume playback only if the card paused it. The card's actions SHALL behave as on a page, and a
card added to the deck SHALL keep what a card captured on a page keeps. A card made from a caption
SHALL keep, on the device only, the watch page address at the time of its line, so that opening
its source returns to that moment.

#### Scenario: Adding a word to the deck from a video
- **WHEN** the reader clicks a highlighted word on the caption line of a playing video and adds it to the deck
- **THEN** the video pauses, a card is created with the caption sentence as source, and the word switches to the learning highlight

#### Scenario: Capturing a phrase from a video
- **WHEN** the reader selects "figure it out" on the caption line
- **THEN** the video pauses and the selection card opens with the caption sentence as source

#### Scenario: A phrase with the engine ready
- **WHEN** a reader whose translation engine is ready for English selects several words on the caption line
- **THEN** the selection card shows the pack's answer, then the engine's translation of the caption sentence with the selection marked, as on a page

#### Scenario: Online translation from a video
- **WHEN** a reader who ticked « Traduction en ligne » selects several words on the caption line
- **THEN** the card shows the « Traduction en ligne » button, and the caption sentence leaves the device only when the reader presses it

#### Scenario: A word card in full screen
- **WHEN** the reader clicks a highlighted word while the video is full screen
- **THEN** the word card is shown over the full-screen player

#### Scenario: Closing the card
- **WHEN** the reader closes a card that paused the video
- **THEN** the video resumes; a video the reader had paused stays paused

#### Scenario: Returning to the moment
- **WHEN** the reader opens the source of a card captured at 3 min 12 s of a video
- **THEN** the watch page opens at that time

### Requirement: Only a caption line played through is an exposure
A caption line SHALL be recorded as an exposure only when playback has covered most of the line's own time span while the video was playing, and SHALL be recorded at most once per video session.
The threshold SHALL be measured in video time, so that a change of playback rate neither prevents
nor multiplies it. Reading the track, analysing it, a line jumped over by a seek, a line shown
while paused or during an advertisement, and annotations and fillers SHALL NOT be recorded. What
is recorded SHALL go through the same exposure path as page reading, under the same conditions,
tagged with its own source (a video, not a page). There is no audio aspect: what the reader hears
is not counted.

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

### Requirement: A track that cannot be read degrades, and says so
When the track cannot be read, the extension SHALL fall back to the caption lines the player writes into the page, and SHALL say in the popup that the video is in degraded mode (French interface: « Mode dégradé : les sous-titres sont lus à mesure qu'ils s'affichent »).
In degraded mode the line SHALL be highlighted and captured from as in the full mode, the line on
screen SHALL be the source sentence, and the percentage SHALL be that of the lines seen so far and
labelled so (French interface: « Lignes vues jusqu'ici »). The failure SHALL be recorded locally by
kind, holding no caption text, so that a breakage is visible in diagnostics.

#### Scenario: The caption request returns nothing
- **WHEN** the replayed caption request answers with an empty body
- **THEN** the mode reads the player's own caption lines, the popup says it is degraded, and the failure kind is recorded with no caption text

### Requirement: The player is left as the reader left it
Ending the caption mode SHALL restore the player's native caption line and the captions state the reader had before the mode started.
Captions turned on by the extension to obtain the track SHALL be turned off again when the mode
ends; captions the reader had turned on SHALL stay on. The mode SHALL end when the reader turns it
off, leaves the watch page, moves to a video it does not start on, or switches Lingua off.

#### Scenario: Captions turned on by the extension
- **WHEN** the reader had captions off, the mode turned them on, and the reader turns the mode off
- **THEN** the player's captions are off again

#### Scenario: Captions the reader wanted
- **WHEN** the reader had captions on before the mode started, and turns the mode off
- **THEN** the player's native captions are shown again
