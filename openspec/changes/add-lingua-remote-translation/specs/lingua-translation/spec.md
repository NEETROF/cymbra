## ADDED Requirements

### Requirement: The reader chooses where the engine runs
The reader SHALL choose, per device, between no extended translation, an engine on the device, and a remote engine, and that choice SHALL NOT change by itself.
The default SHALL be no extended translation. Neither host SHALL be reached without the reader
having chosen it: a device engine that fails SHALL NOT send the sentence to a remote one, and a
remote engine that fails SHALL NOT download a model. When the chosen host answers nothing, the card
SHALL behave exactly as it does with no engine at all.

#### Scenario: The device engine fails while the reader chose the device
- **WHEN** the reader has chosen the engine on the device and it does not answer
- **THEN** no sentence is sent anywhere, and the card shows the answer it would show with no engine

#### Scenario: The remote engine fails while the reader chose remote
- **WHEN** the reader has chosen the remote engine and the request fails
- **THEN** nothing is downloaded, and the card shows the answer it would show with no engine

#### Scenario: A reader who chose nothing
- **WHEN** a reader has never opened the setting
- **THEN** no engine runs, nothing is downloaded, and no sentence leaves the device

### Requirement: The remote engine answers what the device engine would
The remote engine SHALL run the same model as the engine on the device and SHALL answer the same translated sentence with the same marked spans.
Where a mark lands SHALL be settled by the same rule in both cases, so a card SHALL NOT differ
according to where the translation was computed.

#### Scenario: The same selection through either host
- **WHEN** the same sentence and selection are translated on the device and remotely
- **THEN** the translated sentence and the marked spans are the same

### Requirement: The remote engine keeps nothing it is given
A sentence sent for remote translation SHALL NOT be written to any log, store, metric or trace, and SHALL NOT be retained after the answer is returned.
Measurements of the service SHALL count requests and their timing only.

#### Scenario: A translated sentence afterwards
- **WHEN** a sentence has been translated remotely and the answer returned
- **THEN** no copy of that sentence, or of its translation, remains anywhere on the server

#### Scenario: A request that fails
- **WHEN** a remote translation fails
- **THEN** the error recorded names the failure, and holds neither the sentence nor any part of it

### Requirement: Remote translation is for a signed-in reader
Remote translation SHALL be available only to a signed-in reader, and SHALL be limited per account.
Translation on the device SHALL NOT require an account.

#### Scenario: A signed-out reader chooses remote
- **WHEN** a reader who is not signed in chooses remote translation
- **THEN** they are told an account is needed, and no sentence is sent

#### Scenario: A reader past their limit
- **WHEN** a signed-in reader has reached the account's limit
- **THEN** they are told so in their own language, and the card shows the answer it would show with no engine
