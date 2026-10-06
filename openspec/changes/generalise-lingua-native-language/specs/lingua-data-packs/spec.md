## ADDED Requirements

### Requirement: A pack names its native language
The core SHALL read the native language a pack's glosses are written in from the pack's metadata (`native`, an ISO 639-1 code), SHALL refuse to load a pack whose native language it does not know or is the pack's own studied language, and SHALL read a pack's pair, `<studied>-<native>`, from its metadata alone, as it reads its studied language. The pack builder SHALL refuse to build such a pack, and the extension's build SHALL refuse a listed pack whose native language is not the native side of its pair's name, or a pair whose two sides are the same language. Reading the native language SHALL leave every existing pack loadable and byte-identical.

#### Scenario: The en-fr pack names French
- **WHEN** the en-fr pack is loaded, or only its metadata is read
- **THEN** the core reports its native language as French and its pair as `en-fr`

#### Scenario: A native language the core does not know
- **WHEN** a pack whose metadata names `de` as its native language is loaded
- **THEN** loading fails with an explicit error naming the language, and no partial analysis is produced

#### Scenario: A pack glossed in the language it studies
- **WHEN** a pack studying English names `en` as its native language
- **THEN** the builder refuses to build it, and the core refuses to load it

#### Scenario: A pack whose native language is not its pair's
- **WHEN** the list names en-fr and that pack's metadata names `es` as its native language
- **THEN** the extension's build fails with a message naming the pack, its native language and the command that rebuilds it

#### Scenario: The shipped packs do not change
- **WHEN** the en-fr and es-fr packs are built from their committed tables after this change
- **THEN** each sha256 is the one its pin records
