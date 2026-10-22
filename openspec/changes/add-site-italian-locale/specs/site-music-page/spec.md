## MODIFIED Requirements

### Requirement: The Music page shows real screenshots in its language
The Music page SHALL show a gallery of screenshots of the app in each language the page exists in (French, English, Spanish, Italian), and each screenshot's interface SHALL be in that page's language. Every screenshot SHALL be a copy of a store capture produced by the capture harness, never an image edited or taken by hand. Each screenshot SHALL carry alternative text, in the page's language, that says what it shows. Screenshots below the first screen SHALL not delay the page's first render.

#### Scenario: Spanish page
- **WHEN** a visitor opens `/es/music`
- **THEN** the gallery shows the app with its interface in Spanish, each image described in Spanish

#### Scenario: Captures regenerated, site not updated
- **WHEN** the store captures are regenerated and a screenshot on the site no longer matches its source capture
- **THEN** the site check fails, naming the image that drifted

#### Scenario: Missing alternative text
- **WHEN** a screenshot is added to the gallery without alternative text in one of the page's languages
- **THEN** the site check fails
