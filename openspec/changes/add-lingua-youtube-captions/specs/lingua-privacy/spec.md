## ADDED Requirements

### Requirement: The caption request carries nothing of the reader's
The caption mode SHALL make requests only to YouTube, only for the video being watched, and they SHALL carry nothing from Lingua: no page text, no Lingua data, no Cymbra identifier.
A request SHALL be the player's own caption request replayed, changed only in the track it names,
its format, and — for YouTube's translation — the target language. The Lingua annex of the privacy
policy SHALL state that on YouTube the extension fetches the captions of the video being watched
from YouTube, and that without a translation engine the translation shown there is YouTube's.

#### Scenario: Inspecting the caption mode's requests
- **WHEN** the requests made by the caption mode during a video are inspected
- **THEN** every one goes to YouTube for that video, differs from the player's own request only in track, format and target language, and carries no page text, Lingua data or Cymbra identifier

#### Scenario: Reading the policy
- **WHEN** a reader opens the Lingua annex of the privacy policy
- **THEN** it says the extension fetches the video's captions from YouTube, and whose translation is shown there when no engine is chosen
