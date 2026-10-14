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
  the player's own caption request replayed for the track the mode reads, and, while the
  translation line is shown with no engine ready, YouTube's translation of that track — and
  nothing else, only while the mode is on. Reading the list of a video's caption tracks makes no
  request.

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

#### Scenario: The translation line hidden
- **WHEN** a reader with no engine watches a video in the caption mode without showing the translation line
- **THEN** the only request the mode made is the replayed caption request for the track it reads
