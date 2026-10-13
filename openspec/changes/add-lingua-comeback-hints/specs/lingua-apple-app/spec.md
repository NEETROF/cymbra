## ADDED Requirements

### Requirement: The app asks for a rating only through the App Store's own prompt
The Cymbra Lingua app SHALL ask for an App Store rating only through the system's review request (StoreKit), never through a prompt, card or dialog of its own: at the end of a review session that says it is complete (French UI copy: « Séance terminée »), from the fifth such session finished in the app, at most once per version of the app, a short moment after the end is shown. Whether the system shows its prompt SHALL be left to the system. The app SHALL make no network request of its own for it, and the count of sessions it keeps SHALL stay in the app, outside the container it shares with the extension. The review page the app shows SHALL carry none of the extension's rating request.

#### Scenario: The fifth session in the app
- **WHEN** a reader finishes their fifth review session in the app on their iPhone
- **THEN** the app asks StoreKit for a review a moment after « Séance terminée » shows, and the system decides whether its prompt appears

#### Scenario: Once per version
- **WHEN** the reader finishes more sessions in the same version of the app
- **THEN** the app does not ask StoreKit again until a new version is installed

#### Scenario: No prompt of its own
- **WHEN** the end of a session shows in the app
- **THEN** it shows the session's summary and actions only, with no rating card, button or dialog drawn by the app

#### Scenario: On a Mac
- **WHEN** the reader finishes their fifth session in the app on their Mac
- **THEN** the app asks StoreKit for a review from its window, as on the iPhone
