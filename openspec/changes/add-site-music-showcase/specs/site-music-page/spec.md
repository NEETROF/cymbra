# Spec Delta

## Purpose

The public Cymbra Music page on cymbra.app shows the app as it really is — screenshots and a short video in the page's language, plus a share preview — so the page a promotion links to can convince a visitor.

## ADDED Requirements

### Requirement: The Music page shows real screenshots in its language
The Music page SHALL show a gallery of screenshots of the app in each language the page exists in (French, English, Spanish), and each screenshot's interface SHALL be in that page's language. Every screenshot SHALL be a copy of a store capture produced by the capture harness, never an image edited or taken by hand. Each screenshot SHALL carry alternative text, in the page's language, that says what it shows. Screenshots below the first screen SHALL not delay the page's first render.

#### Scenario: Spanish page
- **WHEN** a visitor opens `/es/music`
- **THEN** the gallery shows the app with its interface in Spanish, each image described in Spanish

#### Scenario: Captures regenerated, site not updated
- **WHEN** the store captures are regenerated and a screenshot on the site no longer matches its source capture
- **THEN** the site check fails, naming the image that drifted

#### Scenario: Missing alternative text
- **WHEN** a screenshot is added to the gallery without alternative text in one of the page's languages
- **THEN** the site check fails

### Requirement: The Music page shows a short video of the app being played
The Music page SHALL show a muted video of 30 seconds or less, recorded from a scripted run of the real app, that shows a piece being played with the score following the player. The video SHALL loop without sound and SHALL show a still poster image before it plays. It SHALL start playing on its own only when the visitor has not asked their system to reduce motion; otherwise it SHALL stay on its poster with controls to play it. The video SHALL not be downloaded before the page's first render, and each video file SHALL stay within a size budget of 4 MB.

#### Scenario: Default visitor
- **WHEN** a visitor without a reduced-motion preference scrolls the video into view
- **THEN** it plays muted and loops

#### Scenario: Reduced motion
- **WHEN** a visitor whose system asks to reduce motion opens the page
- **THEN** the video shows its poster and does not play until they start it

#### Scenario: Over budget
- **WHEN** a video file on the site exceeds 4 MB or 30 seconds
- **THEN** the site check fails

### Requirement: Every Music page declares a share preview
Every Music page SHALL declare a share preview: a title, a description, an absolute page URL and an absolute URL to a 1200×630 image whose text is in the page's language. A link to the page pasted into a service that reads these previews SHALL therefore render with that image.

#### Scenario: French link shared
- **WHEN** the built `/music` page is inspected
- **THEN** it declares a preview image at an absolute `https://cymbra.app/` URL, the file exists in the build at 1200×630, and its text is French

#### Scenario: Page without preview
- **WHEN** a Music page in any language is built without a share preview
- **THEN** the post-build check fails, naming the page
