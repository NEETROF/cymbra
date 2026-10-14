## ADDED Requirements

### Requirement: The Lingua page shows the extension at work
The site's Lingua page SHALL show, in each site language it is built in, captures of the real extension made by the capture command of `lingua-browser-extension`, whose interface is in the page's language and which read a language that the page's readers study: in the hero, a highlighted page with its percentage pill and a word card open; further down, a demo video, a page in the readers' second studied language, a review card, the statistics ladder and the EPUB reader. Every image SHALL declare its width and height and carry an alt text in the page's language that quotes no figure; every image below the first screen SHALL load lazily. The video SHALL have a poster, SHALL carry no audio track, SHALL show its controls, SHALL download nothing before it plays, and SHALL NOT start on its own for a visitor who prefers reduced motion, who saves data, or whose browser runs no script. The page SHALL load nothing from a third party for these media.

#### Scenario: The Spanish page
- **WHEN** a visitor opens `/es/lingua/`
- **THEN** the hero shows the Spanish interface on an English page with a word card open, and every image's alt text is in Spanish

#### Scenario: Reduced motion
- **WHEN** a visitor whose system asks for reduced motion scrolls the video into view
- **THEN** the video shows its poster and does not start until they press play

#### Scenario: Motion welcome
- **WHEN** a visitor who does not ask for reduced motion and does not save data scrolls the video into view
- **THEN** it plays without sound, and pauses when it leaves the screen

#### Scenario: No script
- **WHEN** a visitor's browser runs no script
- **THEN** the video shows its poster and controls, and nothing of it is downloaded until they press play

#### Scenario: A phone
- **WHEN** the page is shown 375 pixels wide
- **THEN** the hero shows a close-up of the word card that can be read at that width, and the page does not scroll sideways

### Requirement: The Lingua page points each visitor to the store of their browser
The site's Lingua page SHALL show the three store buttons — Chrome Web Store, Firefox Add-ons and App Store — and SHALL name Firefox for Android among the platforms in its hero note and on the Firefox button. Where the visitor's browser runs scripts, the page SHALL mark as primary the button of the store that serves that browser: the App Store for Safari on a Mac and for any browser on an iPhone or an iPad; Firefox Add-ons for Firefox on a computer; Firefox Add-ons' Android listing for any browser on Android; the Chrome Web Store for Chromium browsers on a computer; none otherwise. It SHALL add a line saying that Lingua runs in Safari on iPhone and iPad, or in Firefox on Android, when the visitor uses another browser there. The choice SHALL be made in the visitor's browser and SHALL NOT be stored or sent anywhere; without scripts, the three buttons SHALL be shown alike, in the same order.

#### Scenario: Safari on a Mac
- **WHEN** a visitor opens the page in Safari on a Mac
- **THEN** the App Store button is primary and the other two stay visible

#### Scenario: Firefox on Android
- **WHEN** a visitor opens the page in Firefox for Android
- **THEN** the Firefox button is primary and opens the add-on's Android listing

#### Scenario: Chrome on Android
- **WHEN** a visitor opens the page in Chrome on Android
- **THEN** the Firefox button is primary, opens the Android listing, and a line says that on Android Lingua runs in Firefox

#### Scenario: Edge on Windows
- **WHEN** a visitor opens the page in Edge on Windows
- **THEN** the Chrome Web Store button is primary

#### Scenario: Chrome on an iPad
- **WHEN** a visitor opens the page in Chrome on an iPad, which reports itself as a Mac with a touch screen
- **THEN** the App Store button is primary, and a line says that on iPhone and iPad Lingua runs in Safari

#### Scenario: No script
- **WHEN** a visitor's browser runs no script
- **THEN** the three buttons are shown alike, in the same order as with scripts, and the hero note names Firefox on computers and on Android

### Requirement: The Lingua page's media stay within a weight budget
The site's build checks SHALL fail when a served image variant of the Lingua page exceeds 250 KB, when the hero's largest variant exceeds 200 KB, when a share image exceeds 300 KB, when a video poster exceeds 150 KB, when a demo video exceeds 4 MB, or when the site's images, posters and videos together exceed 30 MB, naming the file and its size. They SHALL also fail when any built file reaches 25 MiB or the build holds 20,000 files or more, the limits of the site's host.

#### Scenario: A video re-encoded too heavy
- **WHEN** a regenerated demo video weighs 5 MB
- **THEN** the site's post-build check fails, naming that video and its size

#### Scenario: Today's media
- **WHEN** the site is built with the captures of this change
- **THEN** every media file is within its budget and the check passes
