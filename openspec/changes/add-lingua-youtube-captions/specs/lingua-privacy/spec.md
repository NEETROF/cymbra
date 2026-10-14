## ADDED Requirements

### Requirement: The caption request carries nothing of the reader's
The caption mode SHALL make requests only to YouTube, only for the video being watched, and they SHALL carry nothing from Lingua: no page text, no Lingua data, no Cymbra identifier.
A request SHALL be the player's own caption request replayed, changed only in the track it names
and its format. Reading the list of a video's tracks SHALL make no request. The Lingua annex of the
privacy policy, in each of its languages, SHALL state that on YouTube, once the reader starts the
caption mode, the extension fetches the captions of the video being watched from YouTube, and that
nothing of the reader's is sent with them.

#### Scenario: Inspecting the caption mode's requests
- **WHEN** the requests made by the caption mode during a video are inspected
- **THEN** every one goes to YouTube for that video, differs from the player's own request only in track and format, and carries no page text, Lingua data or Cymbra identifier

#### Scenario: Reading the policy
- **WHEN** a reader opens the Lingua annex of the privacy policy, in French, English or Spanish
- **THEN** it says the extension fetches the video's captions from YouTube once the caption mode is started, and sends nothing of the reader's with them
