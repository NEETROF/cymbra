## MODIFIED Requirements

### Requirement: The app signs in with an email and a password through the extension's own flow
The app's sign-in sheet SHALL offer, after Apple and Google, a way to sign in with an email (French UI copy: « Continuer avec un email »), and choosing it SHALL show, in the app's page, the extension's own email sign-in — the same form, steps, copy and error wording as the extension's account page — for an existing account only.
The email sign-in SHALL offer the email and password form, the verification-code step when the
address is not yet verified (signing in once the code is accepted, without asking for the password
again while the app stays open), and the forgotten-password steps (French UI copy: « Mot de passe
oublié ? »), with the emailed code and a new password. It SHALL NOT offer to create an account.
Every failure SHALL be shown in the interface language (French, English, Spanish or Italian) as the
extension words it — wrong email or password, too many attempts, Cymbra unreachable, a wrong or
expired code — and SHALL NOT show a raw technical error. The fields SHALL carry the platform's
autofill hints for a username, a current password, a new password and a one-time code. The password
SHALL be held only in the page's memory and only until the sign-in or the verification step ends;
it SHALL NOT be stored, logged or passed to the native app. Coming back to the app while the email
sign-in is open SHALL keep its current step. Where the app used to say that online translation
needed Apple or Google, it SHALL offer to sign in instead (French UI copy: « Se connecter pour la
traduction en ligne »). Once signed in, whatever the method (Apple, Google or an email), the app's
line SHALL name the account — its handle, else its email address — (French UI copy: « Connecté en
tant que @handle pour la traduction en ligne »), read once after the sign-in and on opening with a
session, kept in memory only; when neither can be read it SHALL keep the plain signed-in line. The
same SHALL hold on iPhone, iPad and Mac.

#### Scenario: An email account signs in
- **WHEN** a reader whose account was created with an email picks « Continuer avec un email » in the app's sheet and enters their email and password
- **THEN** the app is signed in, the online translation that asked for the sign-in is requested, and nothing but the refresh token is kept

#### Scenario: Wrong password
- **WHEN** the reader enters a wrong password
- **THEN** the form stays open with « Email ou mot de passe incorrect. » (French UI copy) in the interface language, and nothing is stored

#### Scenario: Too many attempts
- **WHEN** Cymbra refuses because of too many failed attempts for that address
- **THEN** the form says « Trop de tentatives. Réessaie dans quelques minutes. » (French UI copy) in the interface language, with no technical detail

#### Scenario: An address not yet verified
- **WHEN** the reader signs in with an address whose verification was never completed
- **THEN** the app shows the code step, the reader can ask for a new code, and a valid code signs them in without typing the password again

#### Scenario: Back from the mailbox
- **WHEN** the reader leaves the app on the code step to read the code in Mail and comes back
- **THEN** the code step is still shown, with the address it was sent to

#### Scenario: A forgotten password
- **WHEN** the reader picks « Mot de passe oublié ? », enters their address, then the emailed code and a new password
- **THEN** the code is announced with the same wording whether or not the address has an account, the password is changed, and the reader is back at the sign-in form, told so

#### Scenario: No account creation
- **WHEN** a reader looks at the app's email sign-in
- **THEN** it offers no way to create an account

#### Scenario: Before any sign-in
- **WHEN** online translation is offered in the app's review and the app has never been signed in
- **THEN** the app shows « Se connecter pour la traduction en ligne » (French UI copy) and no note restricting sign-in to Apple or Google

#### Scenario: The account signed in is named
- **WHEN** the app is signed in, with Apple, Google or an email, or opens with the session its Keychain holds
- **THEN** its line says « Connecté en tant que @handle pour la traduction en ligne — Se déconnecter » (French UI copy) with the account's handle, else its email address, else « Connecté à Cymbra pour la traduction en ligne » when neither can be read, and nothing of it is stored
