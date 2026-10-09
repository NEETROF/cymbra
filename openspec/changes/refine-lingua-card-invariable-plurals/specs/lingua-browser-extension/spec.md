## MODIFIED Requirements

### Requirement: The word card says what the form is
The word card SHALL say what the form the reader met is, in words of the interface language and never as a grammatical code: its readings as the card's dictionary form, the other dictionary forms it may also be, and, for a word the analyser split, the pieces it was split into.
- **What the form is.** Its readings as the card's dictionary form, beside the form as seen,
  SHALL be joined into one statement. A form spelled like its dictionary form SHALL show this
  statement only when the pack gives it a reading other than the dictionary form itself and other
  than a plural spelled like it, and SHALL present that reading as a possibility: on that form's own
  card, a noun's, proper noun's, adjective's, determiner's or pronoun's plural reading without a
  degree, whatever its gender, SHALL give no line, in every studied language and every interface
  language, while the form's other readings and other dictionary forms SHALL be said as before.
- **Other dictionary forms.** Each SHALL be named, with its reading, as another possibility. It
  SHALL be text only: it SHALL NOT open a card, and it SHALL NOT change the key of this card,
  its status or its actions.
- **Pieces.** A word the analyser split SHALL show as its pieces joined.
- **Nothing to say.** A form with no reading, no other dictionary form and no pieces SHALL show
  the card as it is shown today.

These lines SHALL sit below the controls that read the word aloud and above the actions, so that a
card completing its answer moves nothing the reader can press.

The names of parts of speech, gender, number and person SHALL be the interface's own. A name that
depends on the studied language, such as the name of a tense, SHALL be provided for English.

#### Scenario: An irregular form
- **WHEN** the reader opens the card of `went`
- **THEN** the card headed `go` says that `went` is the prétérit of `go`

#### Scenario: A form with two readings
- **WHEN** the reader opens the card of `walked`
- **THEN** the card headed `walk` says, in one statement, that `walked` is its prétérit and its participe passé

#### Scenario: A form spelled like its dictionary form
- **WHEN** the reader opens the card of `put`
- **THEN** the card headed `put` says that it may also be its prétérit and its participe passé

#### Scenario: A form of two dictionary forms
- **WHEN** the reader opens the card of `leaves`
- **THEN** the card headed `leave` says that `leaves` may also be the plural of `leaf`, and its actions act on `leave`

#### Scenario: A contraction
- **WHEN** the reader opens the card of the `do` in `don't`
- **THEN** the card headed `do` shows `don't` as `do` + `not`

#### Scenario: A pack without grammar tables
- **WHEN** the reader opens the card of `went` and the pack carries no grammar tables
- **THEN** the card shows the form seen as it does today, and nothing about its reading

#### Scenario: No code reaches the reader
- **WHEN** any word card is shown
- **THEN** no Universal Dependencies code (such as `VERB` or `Tense=Past`) and no occurrence of "lemma" or "lemme" appears in it

#### Scenario: A plural spelled like its dictionary form
- **WHEN** the reader opens the card of `police`, or of the Spanish `crisis` or `nadie`, in any interface language
- **THEN** no line says that it may be the plural of itself, while the card of `leaves` still says that it may be the plural of `leaf`, and the card of the Spanish `paso` still says that it may be a form of `pasar`

#### Scenario: Nothing else on the card moves
- **WHEN** every form of the en-fr, es-fr, es-en and en-es packs is rendered in each interface language
- **THEN** only the cards that said their form may be the plural of their own dictionary form change, each losing that line and nothing else
