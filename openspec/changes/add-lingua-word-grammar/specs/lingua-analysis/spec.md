## ADDED Requirements

### Requirement: A word's grammar, from the pack
The core SHALL answer, for a word as written and the dictionary form its card is keyed by, that word's grammar from the pack: the readings of the form as that dictionary form, the other dictionary forms the form is also a reading of, the pieces the studied language's pre-pass split the written word into, and the dictionary form's gloss with its senses grouped by part of speech.
It SHALL read the written word through the tokeniser and pre-pass the page analysis uses, and SHALL
take its readings from the piece whose dictionary form is the one asked about. It SHALL report
pieces only when the pre-pass split the written word into more than one, and SHALL then name no
other dictionary form: the split has already settled what the piece is. With a pack that carries
no grammar tables it SHALL still answer:
- the gloss, as one group of senses with no part of speech;
- the pieces, which come from the pre-pass and not from the pack;
- no reading and no other dictionary form.

The answer SHALL be deterministic at an equal `analyzer_version` and an equal pack, and identical
byte for byte between the native and the WASM targets over the fixture corpus. Answering it SHALL
leave the page analysis's output unchanged.

#### Scenario: An irregular form
- **WHEN** the core is asked about `went` for the dictionary form `go`
- **THEN** it answers one reading, a verb in the past tense, finite, and no pieces

#### Scenario: A form with two readings
- **WHEN** the core is asked about `walked` for `walk`
- **THEN** it answers two readings: past tense, finite, and past participle

#### Scenario: A contraction
- **WHEN** the core is asked about `doesn't` for `do`
- **THEN** it answers the pieces `does` and `not`, the reading of `does` as a verb in the present tense, third person singular, and no other dictionary form, although `does` alone may be the plural of `doe`

#### Scenario: A form of another dictionary form too
- **WHEN** the core is asked about `leaves` for `leave`, and the pack holds `leaf`
- **THEN** it answers the reading of `leaves` as `leave`, and names `leaf` as another dictionary form, read as a plural noun

#### Scenario: Senses grouped by part of speech
- **WHEN** the core is asked about `can` for `can`
- **THEN** its gloss comes back as a noun group holding « Boîte de conserve » followed by a verb group holding the verb senses

#### Scenario: A pack without grammar tables
- **WHEN** the core is asked about `went` for `go` with a pack that carries no grammar tables
- **THEN** it answers the gloss of `go` as one group with no part of speech, and no reading

#### Scenario: The page analysis does not move
- **WHEN** the fixture corpus is analysed as a page with a pack carrying grammar tables
- **THEN** the output is byte-for-byte the one produced before this change, at `analyzer_version` `1.1.0`

#### Scenario: Native and WASM agree
- **WHEN** the fixture words are answered by the native build and by the WASM build
- **THEN** the two answers are byte-for-byte identical
